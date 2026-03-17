# AstAnalyzer Trait: Generic Framework for AST Analysis Passes

## Goal

Extract the shared lex→parse→walk→Value pipeline from `diag` and `describe` into a generic `AstAnalyzer` trait, so adding a new AST analysis pass (complexity metrics, security audits, dead-code detection, etc.) requires only implementing the trait — no boilerplate for lexing, parsing, file I/O, or Value conversion wiring.

## Why

`diag` and `describe` follow an identical pipeline:

1. Take source string (or file path → read → source string)
2. Lex → parse → get `Program` AST
3. Walk the AST with a custom `AstVisitor` implementation
4. Collect results into a domain-specific output type
5. Convert output ↔ lx `Value` (for use in lx programs)
6. Render output to a string representation

Every step except 3-6 is copy-pasted between the two modules. The lex/parse glue, the `bi_extract` function shape, the `bi_extract_file` function shape, and the `bi_render` function shape are structurally identical — only the visitor, output type, and conversion functions differ.

Adding a third analyzer today means copying ~40 lines of boilerplate from either module. The `AstAnalyzer` trait eliminates this by making the varying parts explicit (associated types + methods) and providing the shared pipeline as generic functions.

## What Changes

**AstAnalyzer trait (Task 1):** Define the trait in `crates/lx/src/ast_analyzer.rs` with associated types for the visitor and output, plus methods for construction, finishing, and Value conversion. Add generic helper functions (`analyze_source`, `analyze_program`, `bi_extract`, `bi_extract_file`, `bi_render`) that work with any implementor. The generic `bi_*` functions have the `BuiltinFn` signature so they can be used directly with `mk()` via monomorphization (e.g. `bi_extract::<DiagAnalyzer>`).

**DiagAnalyzer (Task 2-3):** Create a unit struct `DiagAnalyzer` implementing `AstAnalyzer` for `Walker`/`Graph`. Migrate `diag.rs` to use the generic helpers, removing the duplicated lex/parse/walk boilerplate. The existing `Walker`, `Graph`, `DiagNode`, `DiagEdge`, conversion functions, and `to_mermaid` renderer stay in their current files — only the glue in `diag.rs` changes. The `extract_mermaid` convenience function becomes a two-line call through the trait.

**DescribeAnalyzer (Task 4-5):** Create a unit struct `DescribeAnalyzer` implementing `AstAnalyzer` for `Describer`/`ProgramDescription`. Migrate `describe/mod.rs` to use the generic helpers, removing the duplicated lex/parse/walk boilerplate. The existing `Describer`, `ProgramDescription`, info structs, conversion functions, and renderer stay — only the glue changes.

## How It Works

The trait uses associated types and static dispatch via monomorphization. No closures, no dynamic dispatch, no runtime cost.

```
trait AstAnalyzer {
    type Visitor: AstVisitor;
    type Output;

    const NAME: &'static str;

    fn new_visitor() -> Self::Visitor;
    fn finish(visitor: Self::Visitor) -> Self::Output;
    fn to_value(output: &Self::Output) -> Value;
    fn from_value(val: &Value, span: Span) -> Result<Self::Output, LxError>;
    fn render(output: &Self::Output) -> String;
}
```

Generic helpers eliminate the per-module boilerplate:

```
analyze_source<A>(src, span)     → lex + parse + walk + finish → A::Output
analyze_program<A>(program)      → walk + finish → A::Output
bi_extract<A>(args, span, ctx)   → args[0] as str → analyze_source → to_value
bi_extract_file<A>(args, span, ctx) → args[0] as path → read → analyze_source → to_value
bi_render<A>(args, span, ctx)    → args[0] as value → from_value → render → Str
```

Each module's `build()` function registers the monomorphized generics:

```rust
pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("extract".into(), mk("diag.extract", 1, bi_extract::<DiagAnalyzer>));
    m.insert("extract_file".into(), mk("diag.extract_file", 1, bi_extract_file::<DiagAnalyzer>));
    m.insert("to_mermaid".into(), mk("diag.to_mermaid", 1, bi_render::<DiagAnalyzer>));
    m
}
```

The `bi_*` generics have the exact `BuiltinFn` signature (`fn(&[Value], Span, &Arc<RuntimeCtx>) -> Result<Value, LxError>`) so `mk()` accepts them as function pointers after monomorphization.

## Files Affected

**New files:**
- `crates/lx/src/ast_analyzer.rs` — trait definition + generic helper functions

**Modified files:**
- `crates/lx/src/lib.rs` — add `pub mod ast_analyzer`
- `crates/lx/src/stdlib/diag.rs` — replace manual lex/parse/walk glue with generic helpers
- `crates/lx/src/stdlib/diag_walk.rs` — make `Walker`, `Graph`, `DiagNode`, `DiagEdge` `pub(crate)` (currently `pub(crate)` via `pub(super)` + re-export, needs direct visibility for the trait impl in diag.rs)
- `crates/lx/src/stdlib/describe/mod.rs` — replace manual lex/parse/walk glue with generic helpers
- `crates/lx/src/stdlib/describe/describe_visitor.rs` — make `Describer`, `ProgramDescription`, info structs `pub(crate)` for trait impl access

## Task List

### Task 1: Define AstAnalyzer trait and generic helper functions

**Subject:** Create ast_analyzer.rs with the AstAnalyzer trait and shared pipeline functions

**Description:** Create `crates/lx/src/ast_analyzer.rs`. Define:

```rust
pub trait AstAnalyzer {
    type Visitor: AstVisitor;
    type Output;

    const NAME: &'static str;

    fn new_visitor() -> Self::Visitor;
    fn finish(visitor: Self::Visitor) -> Self::Output;
    fn to_value(output: &Self::Output) -> Value;
    fn from_value(val: &Value, span: Span) -> Result<Self::Output, LxError>;
    fn render(output: &Self::Output) -> String;
}
```

Add these generic functions:

- `pub fn analyze_source<A: AstAnalyzer>(src: &str, span: Span) -> Result<A::Output, LxError>` — lex, parse, create visitor, visit program, finish. Use `A::NAME` in error messages.
- `pub fn analyze_program<A: AstAnalyzer>(program: &Program) -> A::Output` — create visitor, visit program, finish. No error possible since the program is already parsed.
- `pub fn bi_extract<A: AstAnalyzer>(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>` — get string from `args[0]`, call `analyze_source::<A>`, convert with `A::to_value`.
- `pub fn bi_extract_file<A: AstAnalyzer>(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>` — get path from `args[0]`, read file, call `analyze_source::<A>`, convert with `A::to_value`.
- `pub fn bi_render<A: AstAnalyzer>(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>` — call `A::from_value` on `args[0]`, call `A::render`, wrap in `Value::Str`.

Add `pub mod ast_analyzer;` to `crates/lx/src/lib.rs`. Run `just diagnose`.

**ActiveForm:** Defining AstAnalyzer trait and generic helpers

---

### Task 2: Implement AstAnalyzer for DiagAnalyzer

**Subject:** Create DiagAnalyzer unit struct implementing AstAnalyzer for Walker/Graph

**Description:** In `crates/lx/src/stdlib/diag.rs`, define a unit struct `pub(crate) struct DiagAnalyzer;` and implement `AstAnalyzer` for it:

- `type Visitor = Walker;`
- `type Output = Graph;`
- `const NAME: &'static str = "diag";`
- `fn new_visitor() -> Walker` — call `Walker::new()`
- `fn finish(visitor: Walker) -> Graph` — call `visitor.into_graph()`
- `fn to_value(output: &Graph) -> Value` — delegate to existing `graph_to_value`
- `fn from_value(val: &Value, span: Span) -> Result<Graph, LxError>` — delegate to existing `value_to_graph`
- `fn render(output: &Graph) -> String` — delegate to existing `to_mermaid`

Adjust visibility on `Walker`, `Graph`, `DiagNode`, `DiagEdge` in `diag_walk.rs` — change from `pub(crate)` to `pub(crate)` if not already (they are currently `pub(crate)` via the `pub(super)` + crate-level reexport pattern — verify they're accessible from `diag.rs`). Also ensure `Walker::new`, `Walker::into_graph` are accessible.

Run `just diagnose`.

**ActiveForm:** Implementing AstAnalyzer for DiagAnalyzer

---

### Task 3: Migrate diag.rs to use generic helpers

**Subject:** Replace manual lex/parse/walk boilerplate in diag.rs with AstAnalyzer generics

**Description:** In `crates/lx/src/stdlib/diag.rs`:

- Replace `bi_extract` body with a call to `ast_analyzer::bi_extract::<DiagAnalyzer>(args, span, ctx)`
- Replace `bi_extract_file` body with a call to `ast_analyzer::bi_extract_file::<DiagAnalyzer>(args, span, ctx)`
- Replace `bi_to_mermaid` body with a call to `ast_analyzer::bi_render::<DiagAnalyzer>(args, span, ctx)`
- Replace `extract_mermaid` body with: `let graph = ast_analyzer::analyze_program::<DiagAnalyzer>(program); to_mermaid(&graph)`
- Remove the now-unused `extract_graph` function (its logic is now in `analyze_source`)

Update `build()` to use the generic function pointers if desired, or keep the thin wrapper functions — either way the lex/parse/walk boilerplate is gone.

Run `just diagnose` and `just test`.

**ActiveForm:** Migrating diag.rs to generic helpers

---

### Task 4: Implement AstAnalyzer for DescribeAnalyzer

**Subject:** Create DescribeAnalyzer unit struct implementing AstAnalyzer for Describer/ProgramDescription

**Description:** In `crates/lx/src/stdlib/describe/mod.rs`, define a unit struct `pub(crate) struct DescribeAnalyzer;` and implement `AstAnalyzer` for it:

- `type Visitor = Describer;`
- `type Output = ProgramDescription;`
- `const NAME: &'static str = "describe";`
- `fn new_visitor() -> Describer` — call `Describer::new()`
- `fn finish(visitor: Describer) -> ProgramDescription` — call `visitor.into_description()`
- `fn to_value(output: &ProgramDescription) -> Value` — delegate to existing `description_to_value`
- `fn from_value(val: &Value, span: Span) -> Result<ProgramDescription, LxError>` — delegate to existing `value_to_description`
- `fn render(output: &ProgramDescription) -> String` — delegate to existing `describe_render::render_description`

Adjust visibility on `Describer`, `ProgramDescription`, and info structs in `describe_visitor.rs` — change from `pub(super)` to `pub(crate)` so they're accessible from the trait impl.

Run `just diagnose`.

**ActiveForm:** Implementing AstAnalyzer for DescribeAnalyzer

---

### Task 5: Migrate describe/mod.rs to use generic helpers

**Subject:** Replace manual lex/parse/walk boilerplate in describe/mod.rs with AstAnalyzer generics

**Description:** In `crates/lx/src/stdlib/describe/mod.rs`:

- Replace `bi_extract` body with a call to `ast_analyzer::bi_extract::<DescribeAnalyzer>(args, span, ctx)`
- Replace `bi_extract_file` body with a call to `ast_analyzer::bi_extract_file::<DescribeAnalyzer>(args, span, ctx)`
- Replace `bi_render` body with a call to `ast_analyzer::bi_render::<DescribeAnalyzer>(args, span, ctx)`
- Remove the now-unused `extract_description` function (its logic is now in `analyze_source`)

Update `build()` to use the generic function pointers if desired, or keep the thin wrapper functions — either way the lex/parse/walk boilerplate is gone.

Run `just diagnose` and `just test`.

**ActiveForm:** Migrating describe/mod.rs to generic helpers

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AST_ANALYZER_TRAIT.md" })
```

Then call `next_task` to begin. After completing each task's implementation, call `complete_task` to format, commit, and run diagnostics. Repeat until all tasks are done.
