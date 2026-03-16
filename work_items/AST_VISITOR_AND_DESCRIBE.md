# Generic AST Visitor Trait + stdlib/describe Module

## Goal

Add a generic tree-visitor trait to `ast.rs` that any AST consumer can implement, then use it to (a) rewrite the existing `diag_walk` module and (b) build a new `stdlib/describe` module that converts lx programs into structured human-readable summaries.

## Why

Every AST traversal in the codebase re-implements the same recursive match logic independently — `diag_walk` for mermaid graphs, the interpreter for evaluation, the checker for type synthesis. When a new `Expr` or `Stmt` variant is added, every walker must be updated by hand. A generic visitor trait provides:

- One place to define the recursion structure of the AST
- Default `walk_*` implementations that recurse into children automatically
- Consumers only override the nodes they care about — everything else auto-walks
- Compile-time enforcement when new variants are added (trait method signature changes)
- Foundation for future tools: linters, formatters, dead-code analysis, refactoring passes, the dx-backend program-view highlighting

The `describe` module is the first new consumer: given an lx program, it produces a structured record describing what the program does — agents, message flows, control structure, resources used — in a form suitable for rendering in UIs or passing to LLMs as context.

## What Changes

**Visitor trait (Tasks 1-3):** Add `visitor.rs` to `crates/lx/src/` with an `AstVisitor` trait. Default implementations walk children. A `walk` free-function module provides the default recursion so visitors can call `walk::walk_expr(self, expr)` from their overrides. The trait is generic over a return type `R` with a `Default` bound so leaf nodes return `R::default()`.

**Migrate diag_walk (Tasks 4-5):** Rewrite `diag_walk.rs` and `diag_walk_expr.rs` to implement `AstVisitor` for `Walker` instead of hand-matching. The `Walker` struct and `Graph`/`DiagNode`/`DiagEdge` types stay the same — only the traversal changes. `diag.rs` public API is unchanged.

**Describe module (Tasks 6-9):** New `stdlib/describe.rs` implementing `AstVisitor` for a `Describer` struct that builds a `ProgramDescription` record. Register as `std/describe` in `stdlib/mod.rs`. Add lx test suite.

## How It Works

The visitor trait uses Rust's default-method pattern. Each `visit_*` method has a default that calls the corresponding `walk_*` function, which recurses into children. Implementors override specific `visit_*` methods and decide whether to call the walk function for recursion.

```
AstVisitor trait
├── visit_program(program)     → default: walk each stmt
├── visit_stmt(stmt, span)     → default: dispatch to visit_binding, visit_agent_decl, etc.
├── visit_binding(binding, span)
├── visit_type_def(...)
├── visit_protocol(...)
├── visit_protocol_union(...)
├── visit_mcp_decl(...)
├── visit_trait_decl(...)
├── visit_agent_decl(...)
├── visit_field_update(...)
├── visit_use(...)
├── visit_expr(expr, span)     → default: dispatch to visit_literal, visit_binary, etc.
├── visit_literal(literal, span)
├── visit_binary(op, left, right, span)
├── visit_pipe(left, right, span)
├── visit_apply(func, arg, span)
├── visit_agent_send(target, msg, span)
├── visit_agent_ask(target, msg, span)
├── visit_par(stmts, span)
├── visit_sel(arms, span)
├── visit_match(scrutinee, arms, span)
├── visit_refine(initial, grade, revise, threshold, max_rounds, on_round, span)
├── visit_func(params, ret_type, body, span)
├── visit_emit(value, span)
├── visit_yield(value, span)
├── visit_with(name, value, body, mutable, span)
├── visit_with_resource(resources, body, span)
├── visit_loop(stmts, span)
├── visit_block(stmts, span)
├── ... (remaining expr variants)
├── visit_pattern(pattern, span) → default: dispatch to sub-patterns
└── visit_type_expr(type_expr, span)
```

The `Describer` implements only the agent-related, control-flow, and communication visitors. It accumulates into:

```
ProgramDescription {
    imports: [UseStmt summaries],
    agents: [{name, traits, methods, spawned_by}],
    messages: [{from, to, style, label}],
    control_flow: [{kind, children}],    -- par, sel, match, refine, loop
    resources: [{kind, name, source}],   -- mcp connections, with-resource blocks
    ai_calls: [{context, options}],      -- ai.prompt sites
    exports: [exported binding names],
}
```

Exposed to lx programs as:

```lx
use std/describe

desc = describe.extract source_code ^
desc = describe.extract_file "path.lx" ^
text = describe.render desc ^           -- human-readable text
```

## Files Affected

**New files:**
- `crates/lx/src/visitor.rs` — AstVisitor trait + walk module
- `crates/lx/src/stdlib/describe.rs` — Describer implementation + stdlib bindings
- `tests/70_describe.lx` — Test suite

**Modified files:**
- `crates/lx/src/lib.rs` — add `pub mod visitor`
- `crates/lx/src/stdlib/mod.rs` — register `describe` module
- `crates/lx/src/stdlib/diag_walk.rs` — rewrite to use AstVisitor
- `crates/lx/src/stdlib/diag_walk_expr.rs` — rewrite to use AstVisitor

## Task List

### Task 1: Define AstVisitor trait with Stmt visitors

**Subject:** Create visitor.rs with the AstVisitor trait covering all Stmt variants

**Description:** Create `crates/lx/src/visitor.rs`. Define `pub trait AstVisitor` with these methods, all with default implementations that recurse via walk functions:

- `fn visit_program(&mut self, program: &Program)` — iterates `program.stmts` calling `visit_stmt`
- `fn visit_stmt(&mut self, stmt: &Stmt, span: Span)` — matches on Stmt and dispatches to specific visitor
- `fn visit_binding(&mut self, binding: &Binding, span: Span)` — walks `binding.value`
- `fn visit_type_def(&mut self, name: &str, variants: &[(String, usize)], exported: bool, span: Span)` — no-op default
- `fn visit_protocol(&mut self, name: &str, entries: &[ProtocolEntry], exported: bool, span: Span)` — walks default exprs and constraint exprs in entries
- `fn visit_protocol_union(&mut self, def: &ProtocolUnionDef, span: Span)` — no-op default
- `fn visit_mcp_decl(&mut self, name: &str, tools: &[McpToolDecl], exported: bool, span: Span)` — no-op default
- `fn visit_trait_decl(&mut self, name: &str, handles: &[String], provides: &[String], requires: &[String], exported: bool, span: Span)` — no-op default
- `fn visit_agent_decl(&mut self, name: &str, traits: &[String], uses: &[(String, String)], init: Option<&SExpr>, on: Option<&SExpr>, methods: &[AgentMethod], exported: bool, span: Span)` — walks init, on, and method handler exprs
- `fn visit_field_update(&mut self, name: &str, fields: &[String], value: &SExpr, span: Span)` — walks value
- `fn visit_use(&mut self, stmt: &UseStmt, span: Span)` — no-op default

Add `pub mod visitor;` to `lib.rs`. Run `just diagnose`.

**ActiveForm:** Defining AstVisitor trait with Stmt visitors

---

### Task 2: Add Expr visitors to AstVisitor trait

**Subject:** Extend AstVisitor with visit methods for all 34 Expr variants

**Description:** In `crates/lx/src/visitor.rs`, add these methods to `AstVisitor` with default walk implementations:

- `fn visit_expr(&mut self, expr: &Expr, span: Span)` — master dispatch that matches on Expr and calls the specific visitor
- `fn visit_literal(&mut self, lit: &Literal, span: Span)` — walks StrPart::Interp exprs in Str literals
- `fn visit_ident(&mut self, name: &str, span: Span)` — no-op
- `fn visit_type_constructor(&mut self, name: &str, span: Span)` — no-op
- `fn visit_binary(&mut self, op: BinOp, left: &SExpr, right: &SExpr, span: Span)` — walks both sides
- `fn visit_unary(&mut self, op: UnaryOp, operand: &SExpr, span: Span)` — walks operand
- `fn visit_pipe(&mut self, left: &SExpr, right: &SExpr, span: Span)` — walks both
- `fn visit_apply(&mut self, func: &SExpr, arg: &SExpr, span: Span)` — walks both
- `fn visit_section(&mut self, section: &Section, span: Span)` — walks operand exprs in Right/Left variants
- `fn visit_field_access(&mut self, expr: &SExpr, field: &FieldKind, span: Span)` — walks expr and Computed field
- `fn visit_block(&mut self, stmts: &[SStmt], span: Span)` — walks each stmt
- `fn visit_tuple(&mut self, elems: &[SExpr], span: Span)` — walks each
- `fn visit_list(&mut self, elems: &[ListElem], span: Span)` — walks each elem's expr
- `fn visit_record(&mut self, fields: &[RecordField], span: Span)` — walks each field's value
- `fn visit_map(&mut self, entries: &[MapEntry], span: Span)` — walks key and value exprs
- `fn visit_func(&mut self, params: &[Param], ret_type: Option<&SType>, body: &SExpr, span: Span)` — walks default exprs and body
- `fn visit_match(&mut self, scrutinee: &SExpr, arms: &[MatchArm], span: Span)` — walks scrutinee, patterns, guards, bodies
- `fn visit_ternary(&mut self, cond: &SExpr, then_: &SExpr, else_: Option<&SExpr>, span: Span)` — walks all branches
- `fn visit_propagate(&mut self, inner: &SExpr, span: Span)` — walks inner
- `fn visit_coalesce(&mut self, expr: &SExpr, default: &SExpr, span: Span)` — walks both
- `fn visit_slice(&mut self, expr: &SExpr, start: Option<&SExpr>, end: Option<&SExpr>, span: Span)` — walks all present
- `fn visit_named_arg(&mut self, name: &str, value: &SExpr, span: Span)` — walks value
- `fn visit_loop(&mut self, stmts: &[SStmt], span: Span)` — walks stmts
- `fn visit_break(&mut self, value: Option<&SExpr>, span: Span)` — walks value if present
- `fn visit_assert(&mut self, expr: &SExpr, msg: Option<&SExpr>, span: Span)` — walks both
- `fn visit_par(&mut self, stmts: &[SStmt], span: Span)` — walks stmts
- `fn visit_sel(&mut self, arms: &[SelArm], span: Span)` — walks expr and handler per arm
- `fn visit_agent_send(&mut self, target: &SExpr, msg: &SExpr, span: Span)` — walks both
- `fn visit_agent_ask(&mut self, target: &SExpr, msg: &SExpr, span: Span)` — walks both
- `fn visit_emit(&mut self, value: &SExpr, span: Span)` — walks value
- `fn visit_yield(&mut self, value: &SExpr, span: Span)` — walks value
- `fn visit_with(&mut self, name: &str, value: &SExpr, body: &[SStmt], mutable: bool, span: Span)` — walks value and body
- `fn visit_with_resource(&mut self, resources: &[(SExpr, String)], body: &[SStmt], span: Span)` — walks resource exprs and body
- `fn visit_refine(&mut self, initial: &SExpr, grade: &SExpr, revise: &SExpr, threshold: &SExpr, max_rounds: &SExpr, on_round: Option<&SExpr>, span: Span)` — walks all
- `fn visit_shell(&mut self, mode: ShellMode, parts: &[StrPart], span: Span)` — walks Interp parts

Run `just diagnose`.

**ActiveForm:** Adding Expr visitors to AstVisitor trait

---

### Task 3: Add Pattern and TypeExpr visitors

**Subject:** Extend AstVisitor with visit methods for Pattern and TypeExpr variants

**Description:** In `crates/lx/src/visitor.rs`, add:

Pattern visitors:
- `fn visit_pattern(&mut self, pattern: &Pattern, span: Span)` — dispatch to specific pattern visitors
- `fn visit_pattern_literal(&mut self, lit: &Literal, span: Span)` — no-op
- `fn visit_pattern_bind(&mut self, name: &str, span: Span)` — no-op
- `fn visit_pattern_wildcard(&mut self, span: Span)` — no-op
- `fn visit_pattern_tuple(&mut self, elems: &[SPattern], span: Span)` — walks each
- `fn visit_pattern_list(&mut self, elems: &[SPattern], rest: Option<&str>, span: Span)` — walks each
- `fn visit_pattern_record(&mut self, fields: &[FieldPattern], rest: Option<&str>, span: Span)` — walks each field's pattern
- `fn visit_pattern_constructor(&mut self, name: &str, args: &[SPattern], span: Span)` — walks args

TypeExpr visitors:
- `fn visit_type_expr(&mut self, type_expr: &TypeExpr, span: Span)` — dispatch to specific visitors
- `fn visit_type_named(&mut self, name: &str, span: Span)` — no-op
- `fn visit_type_var(&mut self, name: &str, span: Span)` — no-op
- `fn visit_type_applied(&mut self, name: &str, args: &[SType], span: Span)` — walks args
- `fn visit_type_list(&mut self, inner: &SType, span: Span)` — walks inner
- `fn visit_type_map(&mut self, key: &SType, value: &SType, span: Span)` — walks both
- `fn visit_type_record(&mut self, fields: &[TypeField], span: Span)` — walks field types
- `fn visit_type_tuple(&mut self, elems: &[SType], span: Span)` — walks each
- `fn visit_type_func(&mut self, param: &SType, ret: &SType, span: Span)` — walks both
- `fn visit_type_fallible(&mut self, ok: &SType, err: &SType, span: Span)` — walks both

Run `just diagnose`.

**ActiveForm:** Adding Pattern and TypeExpr visitors to AstVisitor

---

### Task 4: Migrate diag_walk.rs to use AstVisitor

**Subject:** Rewrite Walker in diag_walk.rs to implement AstVisitor

**Description:** In `crates/lx/src/stdlib/diag_walk.rs`, replace the hand-written `walk_program` and `walk_stmt` methods with an `impl AstVisitor for Walker` block. Keep the `Walker` struct, `DiagNode`, `DiagEdge`, `Graph` types, and `add_node`/`add_edge` methods unchanged.

Implement these visitor methods (override from defaults):
- `visit_program` — call default walk
- `visit_binding` — check for `extract_agent_spawn` / `extract_mcp_connect`, register in `agent_vars`/`mcp_vars`, else call default walk
- `visit_mcp_decl` — add tool node, register in `mcp_vars`
- `visit_agent_decl` — add agent node, register in `agent_vars`

Remove the old `walk_program`, `walk_stmts`, `walk_stmt` methods. The `into_graph` and `new` methods stay. Update `diag.rs` to call `visitor.visit_program(&program)` instead of `walker.walk_program(&program)`. The public API (`extract`, `extract_file`, `to_mermaid`, `extract_mermaid`) stays identical.

Run `just diagnose` and `just test` to verify diag tests still pass.

**ActiveForm:** Migrating diag_walk.rs to AstVisitor trait

---

### Task 5: Migrate diag_walk_expr.rs to use AstVisitor

**Subject:** Rewrite Walker's expression walking to use AstVisitor overrides

**Description:** In `crates/lx/src/stdlib/diag_walk_expr.rs`, replace the `walk_expr` method with AstVisitor overrides:

- `visit_agent_send` — resolve target, add dashed edge
- `visit_agent_ask` — resolve target, add solid edge
- `visit_par` — add fork node, save/restore context, walk stmts
- `visit_sel` — add decision node, save/restore context, walk arms
- `visit_match` — add decision node with label, save/restore context, walk arm bodies
- `visit_apply` — check for `extract_mcp_call`, if found add edge, else call default walk
- `visit_refine` — walk initial, grade, revise (same as current)

Remove the old `walk_expr` impl block. Keep the helper free functions (`extract_agent_spawn`, `extract_mcp_connect`, `is_field_call`, `extract_spawn_label`, `extract_msg_label`, `extract_str_literal`, `expr_label`, `resolve_target`) — these are diag-specific analysis helpers, not generic walking. `resolve_target` and `extract_mcp_call` move from `impl Walker` methods to free functions taking `&Walker`.

Run `just diagnose` and `just test`.

**ActiveForm:** Migrating diag_walk_expr.rs to AstVisitor overrides

---

### Task 6: Create Describer struct and implement AstVisitor

**Subject:** Build Describer that walks AST to produce ProgramDescription

**Description:** Create `crates/lx/src/stdlib/describe.rs`. Define:

```rust
struct Describer {
    imports: Vec<ImportInfo>,
    agents: Vec<AgentInfo>,
    messages: Vec<MessageInfo>,
    control_flow: Vec<ControlFlowInfo>,
    resources: Vec<ResourceInfo>,
    ai_calls: Vec<AiCallInfo>,
    exports: Vec<String>,
    context_stack: Vec<String>,
}
```

Where the `*Info` structs are:

```rust
struct ImportInfo { path: String, kind: String }
struct AgentInfo { name: String, traits: Vec<String>, methods: Vec<String>, declared: bool, spawned_by: String }
struct MessageInfo { from: String, to: String, style: String, label: String }
struct ControlFlowInfo { kind: String, label: String, depth: usize }
struct ResourceInfo { kind: String, name: String, source: String }
struct AiCallInfo { context: String }
```

Implement `AstVisitor for Describer` overriding:
- `visit_use` — push to `imports`
- `visit_binding` — check for `agent.spawn`, `mcp.connect`, `ai.prompt` calls; track exported bindings
- `visit_agent_decl` — push to `agents` with name, traits, method names
- `visit_mcp_decl` — push to `resources`
- `visit_trait_decl` — no accumulation needed (traits are metadata on agents)
- `visit_agent_send` — push to `messages` with style "send"
- `visit_agent_ask` — push to `messages` with style "ask"
- `visit_par` — push to `control_flow` with kind "par", recurse
- `visit_sel` — push to `control_flow` with kind "sel", recurse
- `visit_match` — push to `control_flow` with kind "match", recurse
- `visit_refine` — push to `control_flow` with kind "refine", recurse
- `visit_loop` — push to `control_flow` with kind "loop", recurse
- `visit_with_resource` — push to `resources` with kind "scoped"
- `visit_apply` — detect `ai.prompt` / `ai.prompt_with` / `mcp.call` patterns, push to `ai_calls` or `resources`
- `visit_emit` — no accumulation (emit is output, not structure)

Add `fn describe(program: &Program) -> ProgramDescription` that creates a `Describer`, visits, and returns the accumulated result. Add `fn description_to_value(desc: &ProgramDescription) -> Value` that converts to an lx Record.

Run `just diagnose`.

**ActiveForm:** Creating Describer with AstVisitor implementation

---

### Task 7: Add text renderer for ProgramDescription

**Subject:** Implement describe.render that converts ProgramDescription to human-readable text

**Description:** In `crates/lx/src/stdlib/describe.rs`, add `fn render_description(desc: &ProgramDescription) -> String` that produces a structured text summary:

```
Program: path/to/file.lx

Imports:
  std/agent, std/ai, ../lib/grading

Agents:
  - researcher (spawned, traits: none)
  - composer (spawned, traits: none)
  - Router (declared, traits: Reviewer)

Message Flow:
  main → researcher: "research" (ask)
  main → researcher: "breakdown" (ask)
  main → composer: "compose" (ask)
  main → composer: kill

Control Flow:
  match on mode → "audit" | "manual"
  par: 3 branches
  refine: grade/revise loop

Resources:
  MCP: gritql (connect)
  Scoped: grit (with-resource)

AI Calls:
  router.route (in run_manual)

Exports:
  main, run_audit, run_manual
```

The format is intentionally plain — no markdown, no mermaid, just structured text suitable for LLM context or terminal display.

Run `just diagnose`.

**ActiveForm:** Adding text renderer for ProgramDescription

---

### Task 8: Register std/describe in stdlib and wire builtins

**Subject:** Expose describe.extract, describe.extract_file, describe.render as stdlib functions

**Description:** In `crates/lx/src/stdlib/describe.rs`, add:

```rust
pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("extract".into(), mk("describe.extract", 1, bi_extract));
    m.insert("extract_file".into(), mk("describe.extract_file", 1, bi_extract_file));
    m.insert("render".into(), mk("describe.render", 1, bi_render));
    m
}
```

Where:
- `bi_extract(source_str)` — lex + parse + `describe()` → description record
- `bi_extract_file(path)` — read file + lex + parse + `describe()` → description record
- `bi_render(description_record)` — convert record back to `ProgramDescription` + `render_description()` → Str

In `crates/lx/src/stdlib/mod.rs`:
- Add `mod describe;`
- Add `"describe" => describe::build()` to the match in `get_std_module`
- Add `"describe"` to the matches! in `std_module_exists`

Run `just diagnose`.

**ActiveForm:** Registering std/describe in stdlib

---

### Task 9: Add test suite for describe module

**Subject:** Create tests/70_describe.lx exercising extract, extract_file, render

**Description:** Create `tests/70_describe.lx` with tests:

1. **extract basic program** — pass a simple lx source string with a binding and assert, verify description record has expected structure (imports list, agents list, exports list)

2. **extract with agents** — source string containing `agent.spawn` calls and `~>?` messages, verify agents and messages arrays are populated correctly

3. **extract with control flow** — source with `par`, match, loop, verify control_flow array

4. **extract with agent declaration** — source with `Agent` declaration syntax, verify agents array includes declared agents with traits and methods

5. **render produces text** — call `describe.render` on a description record, verify result is a non-empty Str containing expected keywords like "Agents:", "Message Flow:", "Imports:"

6. **extract_file** — write a temp file, call `describe.extract_file`, verify it works (use `std/fs` to write temp, clean up after)

Run `just test` to verify all tests pass.

**ActiveForm:** Adding test suite for describe module
