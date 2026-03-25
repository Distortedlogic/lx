# Goal

Add MCP and CLI keyword desugaring with generated method bodies. `MCP Filesystem = { command: "npx", args: ["-y", "@anthropic/mcp-filesystem"] }` desugars to a `Class Filesystem : [Connector]` with auto-generated `connect()`, `disconnect()`, `call()`, and `tools()` methods that wrap `std/mcp`. `CLI GitTool = { command: "git", tool_defs: [...] }` desugars similarly with CLI-specific method bodies.

# Why

- Every MCP connector in the codebase (`pkg/connectors/mcp.lx`) repeats ~30 lines of identical session lifecycle boilerplate. The only things that vary are `command` and `args`.
- Every CLI connector (`pkg/connectors/cli.lx`) repeats ~20 lines of identical arg-building and subprocess execution. The only things that vary are `command` and `tool_defs`.
- Unlike the simple keywords (Unit 2) which just inject a trait name, MCP and CLI generate actual method body AST expressions. This is the most complex desugaring in the pipeline.

# What Changes

**Desugar — `crates/lx/src/folder/desugar.rs`:**

In `transform_stmts`, handle `KeywordDecl { keyword: Mcp, ... }`:

1. Create two `Stmt::Use` statements:
   - `use pkg/core/connector {Connector}`
   - `use std/mcp`
2. Inject `session: None` field if not present in user's fields.
3. Generate four method AST expressions, each allocated in the arena:

   `connect = () { s = mcp.connect self.command self.args ^; self.session <- s; Ok () }`
   - Allocate: Ident `mcp`, FieldAccess `.connect`, Apply with `self.command`, Apply with `self.args`, Propagate `^`, Binding `s`, FieldUpdate `self.session <- s`, Apply `Ok` with `()`

   `disconnect = () { mcp.disconnect self.session; Ok () }`
   - Allocate: Ident `mcp`, FieldAccess `.disconnect`, Apply with `self.session`, Apply `Ok` with `()`

   `call = (req) { mcp.call self.session req.tool req.args }`
   - Allocate: chain of Applies

   `tools = () { mcp.tools self.session }`
   - Allocate: chain of Applies

4. User-provided methods with the same names override generated ones (check before injecting).
5. Create `Stmt::ClassDecl(ClassDeclData { name, traits: [intern("Connector")], fields: user_fields + session, methods: user_methods + generated, exported })`.

Handle `KeywordDecl { keyword: Cli, ... }`:

1. Create `Stmt::Use` for `pkg/core/connector {Connector}`.
2. Inject `tool_defs: []` and `env: {}` fields if not present.
3. Generate four method AST expressions:

   `connect = () { Ok () }`

   `disconnect = () { Ok () }`

   `call = (req) { ... }` — look up req.tool in self.tool_defs, build CLI args, execute via bash builtin, return result. Simplified: `bash (self.command ++ " " ++ req.tool ++ " " ++ (req.args | to_str))`

   `tools = () { self.tool_defs }`

4. User-provided methods override generated ones.
5. Create `Stmt::ClassDecl` with Connector trait.

**Helper — `crates/lx/src/folder/desugar.rs` or new `crates/lx/src/folder/gen_ast.rs`:**

Extract AST generation helpers since building expression trees by hand is verbose. Helper functions:
- `gen_ident(sym, span, arena) -> ExprId` — allocates `Expr::Ident(sym)`
- `gen_apply(func, arg, span, arena) -> ExprId` — allocates `Expr::Apply`
- `gen_field_access(expr, field, span, arena) -> ExprId` — allocates `Expr::FieldAccess`
- `gen_block(stmts, span, arena) -> ExprId` — allocates `Expr::Block`
- `gen_func(params, body, span, arena) -> ExprId` — allocates `Expr::Func`
- `gen_method(name, func_expr) -> AgentMethod` — wraps as method

These keep the desugaring logic readable.

**Validate — `crates/lx/src/folder/validate_core.rs`:**

Remove the temporary pass-through for Mcp and Cli keywords.

# Files Affected

- `crates/lx/src/folder/desugar.rs` — Add MCP and CLI desugaring branches + AST generation helpers
- `crates/lx/src/folder/gen_ast.rs` — New file: AST generation helper functions (optional, can inline)
- `crates/lx/src/folder/mod.rs` — Add mod gen_ast if separate file
- `crates/lx/src/folder/validate_core.rs` — Remove Mcp/Cli pass-through
- `tests/keyword_mcp.lx` — New test file
- `tests/keyword_cli.lx` — New test file

# Task List

### Task 1: Create AST generation helpers

**Subject:** Write helper functions for building expression ASTs in the desugarer

**Description:** Create `crates/lx/src/folder/gen_ast.rs` (or add to desugar.rs if it stays under 300 lines). Write helper functions that allocate AST nodes in the arena:

- `gen_ident(name: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId` — `arena.alloc_expr(Expr::Ident(intern(name)), span)`
- `gen_self_field(field: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId` — `self.field` access
- `gen_apply(func: ExprId, arg: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId` — function application
- `gen_apply_chain(func: ExprId, args: &[ExprId], span: SourceSpan, arena: &mut AstArena) -> ExprId` — curried application chain
- `gen_field_call(obj: &str, method: &str, args: &[ExprId], span: SourceSpan, arena: &mut AstArena) -> ExprId` — `obj.method arg1 arg2`
- `gen_block(stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId`
- `gen_func(param_names: &[&str], body: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId` — lambda with named params
- `gen_ok_unit(span: SourceSpan, arena: &mut AstArena) -> ExprId` — `Ok ()`
- `gen_propagate(inner: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId` — `expr ^`
- `gen_binding(name: &str, value: ExprId, span: SourceSpan, arena: &mut AstArena) -> StmtId` — `let name = value`
- `gen_field_update(name: &str, fields: &[&str], value: ExprId, span: SourceSpan, arena: &mut AstArena) -> StmtId` — `self.field <- value`
- `gen_method(name: &str, func: ExprId) -> AgentMethod`

Add `mod gen_ast;` to `crates/lx/src/folder/mod.rs`. Use `pub(crate)` visibility.

**ActiveForm:** Creating AST generation helpers

---

### Task 2: Implement MCP desugaring

**Subject:** Add MCP keyword desugaring with generated connect/disconnect/call/tools methods

**Description:** Edit `crates/lx/src/folder/desugar.rs`. In `transform_stmts`, add the Mcp branch.

Read `pkg/connectors/mcp.lx` first to understand the exact method bodies that need to be generated. The generated code must be semantically equivalent to McpConnector's methods.

For `connect`: generate a block that calls `mcp.connect self.command self.args`, propagates with `^`, assigns to `s`, then does `self.session <- s`, then returns `Ok ()`. Use the gen_ast helpers.

For `disconnect`: generate `mcp.disconnect self.session` then `Ok ()`.

For `call`: generate `mcp.call self.session req.tool req.args` where `req` is the parameter.

For `tools`: generate `mcp.tools self.session`.

Check user-provided methods: iterate `data.methods` and collect names into a HashSet. Only inject generated methods whose names are NOT in the user's set.

Inject `session: None` field if not in user's fields.

Emit `Stmt::Use` for `pkg/core/connector` (selective: Connector) and `std/mcp` (whole).

Emit `Stmt::ClassDecl` with traits `[intern("Connector")]`, merged fields and methods.

**ActiveForm:** Implementing MCP keyword desugaring

---

### Task 3: Implement CLI desugaring

**Subject:** Add CLI keyword desugaring with generated connect/disconnect/call/tools methods

**Description:** Edit `crates/lx/src/folder/desugar.rs`. In `transform_stmts`, add the Cli branch.

Read `pkg/connectors/cli.lx` first to understand the exact method bodies that need to be generated.

For `connect`: generate `Ok ()`.

For `disconnect`: generate `Ok ()`.

For `call`: generate a body that extracts `req.tool` and `req.args`, looks up the tool in `self.tool_defs`, builds a CLI command string, and executes it. Simplified version: `bash (self.command ++ " " ++ req.tool ++ " " ++ (req.args | to_str))` wrapped in try. Use gen_ast helpers.

For `tools`: generate `self.tool_defs`.

Same override logic as MCP: user-provided methods take precedence.

Inject `tool_defs: []` and `env: {}` fields if not present.

Emit `Stmt::Use` for `pkg/core/connector` (selective: Connector).

Emit `Stmt::ClassDecl` with traits `[intern("Connector")]`.

**ActiveForm:** Implementing CLI keyword desugaring

---

### Task 4: Update validate_core for MCP and CLI

**Subject:** Remove Mcp/Cli pass-through in validate_core

**Description:** Edit `crates/lx/src/folder/validate_core.rs`. Add Mcp and Cli to the list of keyword kinds that must not survive into Core AST.

**ActiveForm:** Updating validate_core for MCP and CLI

---

### Task 5: Write MCP and CLI keyword tests

**Subject:** Create test files validating MCP and CLI keywords

**Description:** Create `tests/keyword_mcp.lx`:

```
MCP TestServer = {
  command: "echo"
  args: ["test"]
}

s = TestServer {}
-- Verify fields exist
assert s.command == "echo"
assert s.args == ["test"]
-- Verify methods exist (they're generated)
assert (method_of s "connect" | some?)
assert (method_of s "disconnect" | some?)
assert (method_of s "call" | some?)
assert (method_of s "tools" | some?)
```

Create `tests/keyword_cli.lx`:

```
CLI TestCli = {
  command: "echo"
  tool_defs: [{name: "hello", args: ["world"]}]
}

c = TestCli {}
assert c.command == "echo"
assert (c.tool_defs | len) == 1
-- Verify Connector methods exist
assert (method_of c "connect" | some?)
assert (method_of c "tools" | some?)
-- tools() returns tool_defs
assert (c.tools () | len) == 1
```

Run `just test`.

**ActiveForm:** Writing MCP and CLI keyword tests

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
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_4_MCP_CLI.md" })
```

Then call `next_task` to begin.
