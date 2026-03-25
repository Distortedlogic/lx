# Goal

Add MCP and CLI keyword desugaring with generated method bodies. `MCP Filesystem = { command: "npx", args: ["-y", "@anthropic/mcp-filesystem"] }` desugars to `Class Filesystem : [Connector]` with generated `connect()`, `disconnect()`, `call()`, `tools()` methods that wrap `std/mcp`. `CLI GitTool = { command: "git", tool_defs: [...] }` desugars similarly with CLI-specific method bodies.

# Why

`pkg/connectors/mcp.lx` repeats ~30 lines of identical MCP session boilerplate per connector — only `command` and `args` vary. `pkg/connectors/cli.lx` repeats ~20 lines of CLI dispatch boilerplate — only `command` and `tool_defs` vary.

# Exact method bodies to generate

**MCP** (from `pkg/connectors/mcp.lx` McpConnector):

```lx
connect = () {
  self.session <- mcp.connect {command: self.command  args: self.args} ^
  Ok ()
}
disconnect = () {
  self.session == None ? (Ok ()) : { mcp.close self.session; Ok () }
}
call = (req) {
  mcp.call self.session req.tool req.args ^
}
tools = () {
  self.session == None ? [] : (mcp.list_tools self.session ^)
}
```

Auto-injected fields: `session: None` (if not provided by user).
Auto-injected imports: `use std/connector {Connector}`, `use std/mcp`.

**CLI** (from `pkg/connectors/cli.lx` CliConnector):

```lx
connect = () Ok ()
disconnect = () Ok ()
call = (req) {
  tool = self.tool_defs | find (t) { t.name == req.tool }
  tool ? {
    None -> Err "unknown tool: {req.tool}"
    Some t -> {
      cli_args = build_cli_args t req.args
      cmd_str = "{self.command} {cli_args}"
      $^{cmd_str} ^
    }
  }
}
tools = () self.tool_defs
```

The `build_cli_args` helper function is also needed:
```lx
build_cli_args = (tool_def args) {
  base = tool_def.subcommand ?? ""
  has_args = args != ()
  flags = has_args ? (args | keys | map (k) { "--{k} {args.get k}" } | join " ") : ""
  base != "" ? (flags != "" ? "{base} {flags}" : base) : flags
}
```

Auto-injected fields: `tool_defs: []`, `env: {}` (if not provided).
Auto-injected imports: `use std/connector {Connector}`.

# What Changes

**AST generation helpers — `crates/lx/src/folder/gen_ast.rs`:**

New file with helper functions for building expression ASTs. These are used by the MCP/CLI/HTTP desugarers. Each function allocates nodes in the arena and returns an ID.

```rust
pub fn gen_ident(name: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_sym(sym: Sym, span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_self_field(field: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId  // self.field
pub fn gen_apply(func: ExprId, arg: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_apply_chain(func: ExprId, args: &[ExprId], span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_field_call(obj: &str, method: &str, args: &[ExprId], span: SourceSpan, arena: &mut AstArena) -> ExprId  // obj.method arg1 arg2
pub fn gen_block(stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_func(params: &[&str], body: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_ok_unit(span: SourceSpan, arena: &mut AstArena) -> ExprId  // Ok ()
pub fn gen_propagate(inner: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId  // expr ^
pub fn gen_binding(name: &str, value: ExprId, span: SourceSpan, arena: &mut AstArena) -> StmtId
pub fn gen_field_update(obj: &str, field: &str, value: ExprId, span: SourceSpan, arena: &mut AstArena) -> StmtId  // obj.field <- value
pub fn gen_literal_str(s: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_literal_unit(span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_none(span: SourceSpan, arena: &mut AstArena) -> ExprId  // None identifier
pub fn gen_record(fields: Vec<(Sym, ExprId)>, span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_list(elems: Vec<ExprId>, span: SourceSpan, arena: &mut AstArena) -> ExprId
pub fn gen_method(name: &str, func: ExprId) -> AgentMethod
```

Each is straightforward arena allocation. For example:
```rust
pub fn gen_ident(name: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    arena.alloc_expr(Expr::Ident(intern(name)), span)
}
pub fn gen_apply(func: ExprId, arg: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    arena.alloc_expr(Expr::Apply(ExprApply { func, arg }), span)
}
pub fn gen_self_field(field: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId {
    let self_id = arena.alloc_expr(Expr::Ident(intern("self")), span);
    arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: self_id, field: FieldKind::Named(intern(field)) }), span)
}
```

**Desugar — `crates/lx/src/folder/desugar.rs`:**

Add Mcp and Cli branches to `desugar_keyword`. Each generates the method ASTs using `gen_ast` helpers, checks for user-provided method overrides (skip generating methods the user already defined), injects default fields, and produces Use + ClassDecl statements.

**Validate — `crates/lx/src/folder/validate_core.rs`:**

Add Mcp and Cli to desugared assertion.

# Files Affected

- `crates/lx/src/folder/gen_ast.rs` — New file: AST generation helpers
- `crates/lx/src/folder/mod.rs` — Add mod gen_ast
- `crates/lx/src/folder/desugar.rs` — Add MCP and CLI desugaring
- `crates/lx/src/folder/validate_core.rs` — Add Mcp/Cli assertion
- `tests/keyword_mcp.lx` — New test
- `tests/keyword_cli.lx` — New test

# Task List

### Task 1: Create AST generation helpers

**Subject:** Write gen_ast.rs with arena allocation helper functions

**Description:** Create `crates/lx/src/folder/gen_ast.rs`. Implement all helper functions listed in What Changes. Each is a small function that allocates one or two AST nodes.

Key patterns:
- `gen_self_field("command", span, arena)` → allocates `Expr::Ident(intern("self"))` then `Expr::FieldAccess` with `FieldKind::Named(intern("command"))`.
- `gen_field_call("mcp", "connect", &[arg1], span, arena)` → allocates `Expr::Ident(intern("mcp"))`, then `Expr::FieldAccess` for `.connect`, then `Expr::Apply` for each arg.
- `gen_func(&["req"], body, span, arena)` → allocates `Expr::Func(ExprFunc { params: vec![Param { name: intern("req"), type_ann: None, default: None }], type_params: vec![], ret_type: None, guard: None, body })`.
- `gen_field_update("self", "session", value, span, arena)` → allocates `Stmt::FieldUpdate(StmtFieldUpdate { name: intern("self"), fields: vec![intern("session")], value })`.

Add `pub(crate) mod gen_ast;` to `crates/lx/src/folder/mod.rs`.

Import requirements: `crate::ast::*`, `crate::sym::intern`, `miette::SourceSpan`.

**ActiveForm:** Creating AST generation helpers

---

### Task 2: Implement MCP desugaring

**Subject:** Generate connect/disconnect/call/tools methods for MCP keyword

**Description:** Edit `crates/lx/src/folder/desugar.rs`. Add the Mcp branch to `desugar_keyword`.

Build the four method bodies using gen_ast helpers. For `connect()`:

```rust
// self.session <- mcp.connect {command: self.command, args: self.args} ^; Ok ()
let self_cmd = gen_self_field("command", span, arena);
let self_args = gen_self_field("args", span, arena);
let config = gen_record(vec![(intern("command"), self_cmd), (intern("args"), self_args)], span, arena);
let mcp_connect = gen_field_call("mcp", "connect", &[config], span, arena);
let propagated = gen_propagate(mcp_connect, span, arena);
let assign = gen_field_update("self", "session", propagated, span, arena);
let ok = gen_ok_unit(span, arena);
let ok_stmt = arena.alloc_stmt(Stmt::Expr(ok), span);
let body = gen_block(vec![assign, ok_stmt], span, arena);
let connect_fn = gen_func(&[], body, span, arena);
```

For `disconnect()`: check `self.session == None`, if true return `Ok ()`, else call `mcp.close self.session` then `Ok ()`. Use ternary or match — since ternary is desugared to match by the same Desugarer's `leave_expr`, a ternary expression works here.

For `call(req)`: `mcp.call self.session req.tool req.args ^`.

For `tools()`: `self.session == None ? [] : (mcp.list_tools self.session ^)`.

Check user overrides: collect `data.methods` names into a set. Only inject generated methods whose names are NOT in the user's set.

Inject `session: None` field if user didn't provide it.

Emit `Stmt::Use` for `std/connector {Connector}` and `std/mcp` (UseKind::Whole).

Emit `Stmt::ClassDecl` with `traits: [intern("Connector")]`.

**ActiveForm:** Implementing MCP desugaring

---

### Task 3: Implement CLI desugaring

**Subject:** Generate connect/disconnect/call/tools methods for CLI keyword

**Description:** Edit `crates/lx/src/folder/desugar.rs`. Add the Cli branch.

`connect()` and `disconnect()`: return `Ok ()`.

`call(req)`: This is the most complex generated method. The actual CliConnector uses `build_cli_args` helper + `$^{cmd_str}` bash interpolation. For the desugared version, generate a simplified call body:

The simplest correct approach: generate a `call` method that pipe-builds a command string and uses the `bash` builtin (which is a global function in lx). Generate the expression `bash (self.command ++ " " ++ req.tool)` wrapped in try. This is simpler than the full CliConnector's arg-building logic but functionally equivalent for basic use cases.

For `tools()`: return `self.tool_defs`.

Inject `tool_defs: []` field if not present (default is empty list literal). Inject `env: {}` field if not present (default is empty record literal).

Emit `Stmt::Use` for `std/connector {Connector}`.

**ActiveForm:** Implementing CLI desugaring

---

### Task 4: Update validate_core

**Subject:** Add Mcp and Cli to desugared assertion

**Description:** Edit `crates/lx/src/folder/validate_core.rs`. Add `KeywordKind::Mcp` and `KeywordKind::Cli` to the assertion list.

**ActiveForm:** Updating validate_core

---

### Task 5: Write MCP and CLI tests

**Subject:** Test MCP and CLI keywords end-to-end

**Description:** Create `tests/keyword_mcp.lx`:

```lx
MCP TestServer = {
  command: "echo"
  args: ["test"]
}

s = TestServer {}
assert s.command == "echo"
assert s.args == ["test"]
assert (methods_of s | any? (== "connect"))
assert (methods_of s | any? (== "disconnect"))
assert (methods_of s | any? (== "call"))
assert (methods_of s | any? (== "tools"))
```

Create `tests/keyword_cli.lx`:

```lx
CLI TestCli = {
  command: "echo"
  tool_defs: [{name: "hello", subcommand: "hello"}]
}

c = TestCli {}
assert c.command == "echo"
assert (c.tool_defs | len) == 1
assert (methods_of c | any? (== "connect"))
assert (methods_of c | any? (== "tools"))
result = c.tools ()
assert (result | len) == 1
```

Note: These tests verify the generated methods EXIST but don't call connect/call because that would require actual MCP servers or CLI commands. The method existence proves desugaring worked. The connect/call invocations are tested manually or in integration (Unit 6).

Run `just test`.

**ActiveForm:** Writing MCP and CLI tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_4_MCP_CLI.md" })
```
