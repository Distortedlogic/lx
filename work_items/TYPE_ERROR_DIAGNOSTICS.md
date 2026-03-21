# Goal

Improve type error messages from the checker to include source location context, expected vs found types with labels, and actionable help text. Currently the checker produces generic messages like `"type mismatch: expected Int, got Str"` with no indication of which function call, parameter, or expression caused the mismatch.

# Why

- The span infrastructure exists in `LxError::Type { msg, span }` and miette rendering, but the checker's `unify()` method in `checker/types.rs` only returns `Err(String)` — it discards all context about what was being unified and why.
- The research in `research/diagnostics/` covers Rust and Elm error message design — both use structured multi-span errors with labels, expected/found pairs, and help suggestions.
- Type errors are the most common error during development. Better messages directly reduce agent iteration cycles.

# What Changes

## Enrich unification errors with context

Replace `UnificationTable::unify` returning `Err(String)` with a structured error type:

```rust
pub struct TypeError {
    pub expected: Type,
    pub found: Type,
    pub context: TypeContext,
}

pub enum TypeContext {
    FuncArg { func_name: String, param_name: String, param_idx: usize },
    FuncReturn { func_name: String },
    Binding { name: String },
    RecordField { field_name: String },
    ListElement { index: usize },
    MatchArm { arm_idx: usize },
    BinaryOp { op: String },
}
```

## Propagate context through the checker

In `checker/synth.rs` and `checker/stmts.rs`, where `unify` is called, wrap the call with context about what's being checked:

```rust
self.table.unify_with_context(
    &expected, &found,
    TypeContext::FuncArg { func_name: "retry", param_name: "max_attempts", param_idx: 0 }
)?;
```

## Render rich error messages

When constructing `LxError::Type`, format the message with the structured context:

```
type mismatch in argument `max_attempts` of `retry`
  expected: Int
     found: Str
```

Add help text for common cases:
- Int/Str mismatch on a function arg → "did you mean to pass a number?"
- Func/non-Func mismatch → "this value is not callable"
- Record missing field → "record is missing field `name`"

# Files Affected

**Modified files:**
- `crates/lx/src/checker/types.rs` — add `TypeError`, `TypeContext` types; add `unify_with_context` method
- `crates/lx/src/checker/synth.rs` — pass context to unify calls for function args, returns, bindings
- `crates/lx/src/checker/stmts.rs` — pass context for binding type checks
- `crates/lx/src/checker/mod.rs` — convert `TypeError` to `LxError::Type` with rich message
- `crates/lx/src/error.rs` — add `help` field to `LxError::Type`

**Verification:** Existing type error tests in `tests/12_types.lx` must still pass. New error message format is verified by running `just diagnose` on intentionally mistyped programs.

# Task List

### Task 1: Add structured TypeError and TypeContext types

**Subject:** Define TypeError with expected/found/context for rich diagnostics

**Description:** In `crates/lx/src/checker/types.rs`, add:

```rust
pub struct TypeError {
    pub expected: Type,
    pub found: Type,
    pub context: TypeContext,
}

pub enum TypeContext {
    FuncArg { func_name: String, param_name: String, param_idx: usize },
    FuncReturn { func_name: String },
    Binding { name: String },
    RecordField { field_name: String },
    ListElement { index: usize },
    MatchArm { arm_idx: usize },
    BinaryOp { op: String },
    General,
}
```

Add `pub fn unify_with_context(&mut self, a: &Type, b: &Type, ctx: TypeContext) -> Result<Type, TypeError>` that calls `self.unify(a, b)` and wraps the `Err(String)` into a `TypeError` with the provided context. Keep the existing `unify` method unchanged for backward compatibility.

Run `just diagnose`.

**ActiveForm:** Defining TypeError and TypeContext types

---

### Task 2: Add rich message formatting to TypeError

**Subject:** Format TypeError into multi-line diagnostic messages with help text

**Description:** Implement `TypeError::to_message(&self) -> String` that produces context-aware messages:

For `FuncArg`: `"type mismatch in argument '{param_name}' (#{param_idx}) of '{func_name}'\n  expected: {expected}\n     found: {found}"`

For `Binding`: `"type mismatch in binding '{name}'\n  expected: {expected}\n     found: {found}"`

For `RecordField`: `"type mismatch in record field '{field_name}'\n  expected: {expected}\n     found: {found}"`

For `General` (fallback): `"type mismatch\n  expected: {expected}\n     found: {found}"` — identical to current behavior.

Add `TypeError::help(&self) -> Option<String>` returning contextual help:
- Str where Int expected → `Some("did you mean to pass a number?")`
- non-Func where Func expected → `Some("this value is not callable")`
- Otherwise → `None`

Add `help` field (`Option<String>`) to `LxError::Type` variant in `crates/lx/src/error.rs` and wire miette's `#[help]` attribute to it.

Run `just diagnose`.

**ActiveForm:** Formatting rich type error messages

---

### Task 3: Wire TypeContext into checker synth and stmts

**Subject:** Pass context to unify calls in the checker for function args and bindings

**Description:** In `crates/lx/src/checker/synth.rs`, find all calls to `self.table.unify()` and replace with `self.table.unify_with_context()` where context is available:

- Function argument checking: pass `TypeContext::FuncArg` with function name, param name, and index.
- Function return type: pass `TypeContext::FuncReturn` with function name.
- Binary operators: pass `TypeContext::BinaryOp` with the operator string.

In `crates/lx/src/checker/stmts.rs`:
- Binding type annotations: pass `TypeContext::Binding` with the binding name.
- Record field assignments: pass `TypeContext::RecordField`.

Convert `TypeError` to `LxError::Type` at the point where errors are surfaced, using `TypeError::to_message()` for the msg and `TypeError::help()` for the help field.

For `unify` calls where context isn't readily available, use `TypeContext::General` — these can be incrementally enriched later.

Run `just diagnose` and `just test`.

**ActiveForm:** Wiring context into checker unify calls

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/TYPE_ERROR_DIAGNOSTICS.md" })
```

Then call `next_task` to begin.
