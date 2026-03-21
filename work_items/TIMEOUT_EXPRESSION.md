# Goal

Add `timeout` as a first-class expression wrapper so any expression can be time-bounded without wrapping it in a `sel` block. Currently `sel { expr -> it; timeout 5000 -> fallback }` is the only way to add a timeout — verbose for the common case of "run this one thing with a time limit."

# Why

- Every agentic workflow has operations that can hang — LLM calls, HTTP requests, agent asks. Timeout is the most common `sel` usage pattern but requires 3 lines of boilerplate for one expression.
- The research in `research/concurrency/design-patterns.md` covers structured concurrency (Trio, Kotlin) where timeout wraps expressions directly.
- `timeout` already exists as a `sel` arm keyword — promoting it to an expression doesn't add new syntax, just broadens where it can appear.

# What Changes

## New expression: `timeout MILLIS EXPR`

When used outside a `sel` block, `timeout millis expr` evaluates `expr` with a time limit. If `expr` completes within `millis`, returns `Ok result`. If it exceeds the limit, returns `Err {kind: :timeout, ms: millis}`.

```
result = timeout 5000 (http.get url)
result ? {
  Ok data -> process data
  Err {kind: :timeout ..} -> fallback
}
```

This desugars internally to the existing `sel` machinery — wrapping the expression and a timeout arm.

## Parser changes

In `crates/lx/src/parser/prefix.rs`, when `timeout` is encountered as a prefix token (not inside a `sel` arm context), parse it as: `Expr::Timeout { ms: SExpr, body: SExpr }`. The `ms` expression is parsed first, then the body.

## AST changes

Add `Timeout { ms: Box<SExpr>, body: Box<SExpr> }` variant to the `Expr` enum in `crates/lx/src/ast/mod.rs`.

## Interpreter changes

In the interpreter, `Timeout` evaluates as: spawn `body` as a future, race it against `tokio::time::sleep(Duration::from_millis(ms))`. If body wins, return `Ok(result)`. If timer wins, return `Err({kind: :timeout, ms: ms_val})`.

## Checker changes

Add a match arm for `Timeout` in the checker that synthesizes `Result { ok: body_type, err: Record }`.

# Files Affected

**Modified files:**
- `crates/lx/src/ast/mod.rs` — add `Timeout` variant to `Expr`
- `crates/lx/src/parser/prefix.rs` — parse `timeout MILLIS EXPR` as expression
- `crates/lx/src/interpreter/eval.rs` — evaluate `Timeout` via tokio select
- `crates/lx/src/checker/synth.rs` — synthesize type for `Timeout`
- `crates/lx/src/visitor/walk/mod.rs` — walk `Timeout` children

**New files:**
- `tests/80_timeout_expr.lx` — tests for timeout as expression

# Task List

### Task 1: Add Timeout AST variant and parser support

**Subject:** Parse `timeout MILLIS EXPR` as a standalone expression

**Description:** Add `Timeout { ms: Box<SExpr>, body: Box<SExpr> }` to the `Expr` enum in `crates/lx/src/ast/mod.rs`.

In `crates/lx/src/parser/prefix.rs`, add a match arm for the `timeout` keyword when it appears as a prefix expression (not inside a `sel` block). Parse: consume `timeout` keyword, parse the milliseconds expression at high binding power (e.g., BP 100 to get a single literal or ident), parse the body expression in parentheses or at standard BP.

Update the visitor walker in `crates/lx/src/visitor/walk/mod.rs` to walk both `ms` and `body` children.

Run `just diagnose`.

**ActiveForm:** Adding Timeout AST variant and parser

---

### Task 2: Implement Timeout evaluation in interpreter

**Subject:** Evaluate timeout expression via tokio select

**Description:** In `crates/lx/src/interpreter/eval.rs` (or whichever file handles eval dispatch), add a match arm for `Expr::Timeout { ms, body }`:

1. Evaluate `ms` to get the timeout duration in milliseconds (must be Int or Float).
2. Spawn `body` evaluation as a future.
3. Use `tokio::select!` to race the body future against `tokio::time::sleep(Duration::from_millis(ms_val))`.
4. If body completes first: return `Ok(LxVal::ok(result))`.
5. If timer fires first: return `Ok(LxVal::err(LxVal::Record(indexmap!{"kind" => LxVal::str(":timeout"), "ms" => LxVal::Int(ms_val)})))`.

Run `just diagnose`.

**ActiveForm:** Implementing timeout evaluation

---

### Task 3: Add checker support and write tests

**Subject:** Type-check Timeout expression and create test suite

**Description:** In `crates/lx/src/checker/synth.rs`, add a match arm for `Expr::Timeout`: synthesize `ms` (expect Int or Float), synthesize `body`, return `Type::Result { ok: body_type, err: Type::Record(...) }`.

Create `tests/80_timeout_expr.lx` with tests:
1. `timeout 1000 (42)` — should return `Ok 42` (instant completion).
2. `timeout 1 (loop {})` — should return `Err` with kind `:timeout` (loop never completes).
3. Verify the timeout error record has `kind` and `ms` fields.
4. Compose with `?` operator: `timeout 1000 (42) ?` should unwrap to 42.

Run `just diagnose` and `just test`.

**ActiveForm:** Adding checker support and writing timeout tests

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
mcp__workflow__load_work_item({ path: "work_items/TIMEOUT_EXPRESSION.md" })
```

Then call `next_task` to begin.
