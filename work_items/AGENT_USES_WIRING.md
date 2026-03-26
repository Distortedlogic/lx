# Goal

Add `uses` declarations to Agent keyword bodies so agents can declaratively specify connectors that auto-connect during initialization.

# Why

- `uses` is documented but NOT implemented — `KeywordDeclData` at `ast/types.rs:115-124` has no `uses` field, `stmt_keyword.rs` doesn't parse it, `desugar.rs` doesn't generate any wiring
- Agents must manually create and connect connectors in method bodies

# Syntax

```
+Agent MyAgent = {
  uses MyMcp
  uses MyHttp

  act = (msg) { ... }
}
```

`uses` appears inside the `{ }` body before fields and methods. Each `uses` declaration is the `Use` keyword token followed by a TypeName (the connector type name).

# How it works

The `class_body` function at `stmt_class.rs:10-39` currently parses `{ members }` where each member is a field (name `:` expr) or method (name `=` expr). A third member variant is added: `uses` (the `Use` token followed by `type_name()`). The `class_body` return type changes from `(Vec<ClassField>, Vec<AgentMethod>)` to `(Vec<Sym>, Vec<ClassField>, Vec<AgentMethod>)` where the first vec is the uses list.

Callers: `class_parser` at `stmt_class.rs:61` ignores the uses vec (destructure and discard). `keyword_parser` at `stmt_keyword.rs:52` passes the uses vec to `KeywordDeclData`.

The desugarer at `desugar.rs:201-226` generates for each uses entry: (1) a field `__conn_{name}: None`, (2) init body that creates the connector and calls `.connect()`, (3) a tools method that collects tools from all connectors.

# Files affected

- `crates/lx/src/ast/types.rs` line 121 — Add `pub uses: Vec<Sym>` to `KeywordDeclData`
- `crates/lx/src/parser/stmt_class.rs` lines 10-39 — Add `Uses` variant to `ClassMember`, parse `Use TypeName`, return uses in tuple
- `crates/lx/src/parser/stmt_class.rs` line 62 — Update `class_parser` to destructure and discard uses
- `crates/lx/src/parser/stmt_keyword.rs` line 52-56 — Pass uses to `KeywordDeclData`
- `crates/lx/src/folder/desugar.rs` lines 201-226 — Generate fields, init, and tools from uses

# Task List

### Task 1: Add uses field to KeywordDeclData

In `crates/lx/src/ast/types.rs`, add `pub uses: Vec<Sym>` to `KeywordDeclData` after `exported` at line 122. In `crates/lx/src/parser/stmt_keyword.rs`, update both construction sites at lines 56 and 58 to include `uses: vec![]` temporarily (the parser wiring comes in Task 3).

### Task 2: Add Uses variant to ClassMember and parse it

In `crates/lx/src/parser/stmt_class.rs`, add `Uses(Sym)` to the `ClassMember` enum at line 66. Add a `uses_member` parser: `just(TokenKind::Use).ignore_then(type_name()).map(ClassMember::Uses)`. Add it as the first choice in the `member` parser at line 21: `let member = uses_member.or(class_field).or(class_method)`. `Use` token does not conflict with field/method parsing because fields start with ident/typename + Colon and methods start with ident/typename + Assign — `Use` is a distinct keyword token.

Change `class_body` return type from `(Vec<ClassField>, Vec<AgentMethod>)` to `(Vec<Sym>, Vec<ClassField>, Vec<AgentMethod>)`. In the `.map()` at lines 28-38, add a `uses` vec, collect `ClassMember::Uses(name)` into it, and return `(uses, fields, methods)`.

### Task 3: Update callers of class_body

In `crates/lx/src/parser/stmt_class.rs`, update `class_parser` at line 62. The `.then(class_body(expr))` now returns `(Vec<Sym>, Vec<ClassField>, Vec<AgentMethod>)`. Destructure as `(_, fields, methods)` — discard uses for plain Class declarations.

In `crates/lx/src/parser/stmt_keyword.rs`, update `other_branch` at line 52. The `.then(class_body(expr))` now returns `(Vec<Sym>, Vec<ClassField>, Vec<AgentMethod>)`. Destructure as `(uses, fields, methods)`. Pass `uses` to the `KeywordDeclData` constructor at line 56 instead of `vec![]`.

### Task 4: Generate connector wiring in the desugarer

In `crates/lx/src/folder/desugar.rs`, extend `desugar_keyword` at lines 201-226. After building `fields` and `methods` from `data.fields` and `data.methods`, check `data.uses`. If empty, proceed as before. If non-empty:

For each `uses` entry (a Sym like the interned string "MyMcp"): add a `ClassField` with name `intern("__conn_{lowercase}")` and default set to a `gen_ident` for `None`. Use `gen_ast.rs` helpers: `gen_ident(name, span, arena)` at line 9, `gen_apply(func, arg, span, arena)` at line 18, `gen_field_call(obj, method, args, span, arena)` at line 30.

Generate an init method body that for each connector: creates the instance (`gen_apply(gen_ident(type_name), gen_ident("()"), ...)`) and calls `.connect()` on it (`gen_field_call`). If the user defined an init method in `data.methods`, prepend the generated statements to its body. If not, create a new init method.

Generate or extend the tools method to call `.tools()` on each `__conn_*` field and spread the results into the tools list.

### Task 5: Add tests

Create `tests/agent_uses_wiring.lx`:

Define a Connector: `+Connector TestConn = { connect = () { Ok () }; disconnect = () { Ok () }; call = (req) { Ok "done" }; tools = () { ["test_tool"] } }`.

Define an Agent using it: `+Agent TestAgent = { uses TestConn; act = (msg) { "done" } }`.

Instantiate: `a = TestAgent {}`. Assert `a.tools () | len > 0` (tools collected from connector). Assert `a.tools () | find (t) { t == "test_tool" }` is not None.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
