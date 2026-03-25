# Goal

Add the 12 keyword tokens to the lexer, a single `KeywordDecl` AST node, a unified keyword parser, and the desugaring pass that converts 8 simple keywords (Agent, Tool, Prompt, Connector, Store, Session, Guard, Workflow) into `Class : [Trait]` + auto-injected `use` import. After this unit, writing `Agent MyAgent = { ... }` produces identical runtime behavior to `use pkg/agent {Agent}` followed by `Class MyAgent : [Agent] = { ... }`.

# Why

- This is the core compiler pipeline that makes all 12 keywords parse and the 8 simple ones fully functional.
- The simple keywords are pure syntactic sugar — they inject a trait name and an import. No method body generation. This keeps the desugaring logic minimal and testable.
- Schema (Unit 3), MCP/CLI (Unit 4), and HTTP (Unit 5) build on this infrastructure.

# What Changes

**Lexer — `crates/lx/src/lexer/token.rs`:**

Add 12 variants to `TokenKind`: `AgentKw`, `ToolKw`, `PromptKw`, `ConnectorKw`, `StoreKw`, `SessionKw`, `GuardKw`, `WorkflowKw`, `SchemaKw`, `McpKw`, `CliKw`, `HttpKw`.

**Lexer — `crates/lx/src/lexer/helpers.rs`:**

Add 12 match arms to `type_name_or_keyword()` mapping capitalized strings to the new token variants.

**AST — `crates/lx/src/ast/types.rs`:**

Add `KeywordKind` enum with 12 variants: `Agent`, `Tool`, `Prompt`, `Connector`, `Store`, `Session`, `Guard`, `Workflow`, `Schema`, `Mcp`, `Cli`, `Http`.

Add `KeywordDeclData` struct: `keyword: KeywordKind`, `name: Sym`, `type_params: Vec<Sym>`, `fields: Vec<ClassField>`, `methods: Vec<AgentMethod>`, `exported: bool`.

**AST — `crates/lx/src/ast/mod.rs`:**

Add `#[walk(skip)] KeywordDecl(KeywordDeclData)` variant to `Stmt` enum.

**Parser — new file `crates/lx/src/parser/stmt_keyword.rs`:**

Single `keyword_parser()` function that: matches any of the 12 keyword tokens, reads optional Export prefix, reads TypeName (declaration name), reads optional type params, reads `=`, delegates to class body parser (fields + methods), produces `Stmt::KeywordDecl(KeywordDeclData { ... })`.

**Parser — `crates/lx/src/parser/stmt.rs`:**

Add `keyword_parser()` to the `choice()` list in `stmt_parser()`, before `class_parser` and `trait_parser`.

**Desugar — `crates/lx/src/folder/desugar.rs`:**

Override `transform_stmts()` on `Desugarer`. For each `Stmt::KeywordDecl` with a simple keyword kind (Agent, Tool, Prompt, Connector, Store, Session, Guard, Workflow):

1. Create a `Stmt::Use(UseStmt { path, kind: Selective([trait_sym]) })` importing the trait from its package location.
2. Create a `Stmt::ClassDecl(ClassDeclData { name, traits: [trait_sym], fields, methods, exported, type_params })`.
3. For Store specifically, inject an `entries: Store()` field if the user didn't provide one.
4. Return both statements (Use + ClassDecl) in place of the single KeywordDecl.

For Schema, Mcp, Cli, Http keywords: pass through unchanged (handled by later units). They remain as KeywordDecl in the Surface AST. The validate_core pass must NOT assert on them yet until those units land.

Import path mapping:
- Agent → `pkg/agent` import `Agent`
- Tool → `pkg/core/tool` import `Tool`
- Prompt → `pkg/core/prompt_trait` import `Prompt`
- Connector → `pkg/core/connector` import `Connector`
- Store → `pkg/core/collection` import `Collection`
- Session → `pkg/core/session` import `Session`
- Guard → `pkg/core/guard` import `Guard`
- Workflow → `pkg/core/workflow` import `Workflow`

**Validate — `crates/lx/src/folder/validate_core.rs`:**

Add check that no `Stmt::KeywordDecl` with simple keyword kinds survives into Core AST. Schema/Mcp/Cli/Http are temporarily allowed until their units land.

# Files Affected

- `crates/lx/src/lexer/token.rs` — Add 12 TokenKind variants
- `crates/lx/src/lexer/helpers.rs` — Add 12 keyword match arms
- `crates/lx/src/ast/types.rs` — Add KeywordKind enum, KeywordDeclData struct
- `crates/lx/src/ast/mod.rs` — Add KeywordDecl variant to Stmt
- `crates/lx/src/parser/stmt_keyword.rs` — New file: keyword parser
- `crates/lx/src/parser/mod.rs` — Add mod stmt_keyword
- `crates/lx/src/parser/stmt.rs` — Add keyword_parser to choice
- `crates/lx/src/folder/desugar.rs` — Add transform_stmts for keyword desugaring
- `crates/lx/src/folder/validate_core.rs` — Add KeywordDecl validation
- `tests/keyword_agent.lx` — New test file
- `tests/keyword_tool.lx` — New test file
- `tests/keyword_prompt.lx` — New test file
- `tests/keyword_connector.lx` — New test file
- `tests/keyword_store.lx` — New test file
- `tests/keyword_session.lx` — New test file
- `tests/keyword_guard.lx` — New test file
- `tests/keyword_workflow.lx` — New test file

# Task List

### Task 1: Add keyword token variants to lexer

**Subject:** Add 12 keyword token variants to TokenKind and helpers

**Description:** Edit `crates/lx/src/lexer/token.rs`: add `AgentKw`, `ToolKw`, `PromptKw`, `ConnectorKw`, `StoreKw`, `SessionKw`, `GuardKw`, `WorkflowKw`, `SchemaKw`, `McpKw`, `CliKw`, `HttpKw` to the `TokenKind` enum in the keywords section (near lines 70-84).

Edit `crates/lx/src/lexer/helpers.rs`: add match arms in `type_name_or_keyword()` for `"Agent"` → `AgentKw`, `"Tool"` → `ToolKw`, `"Prompt"` → `PromptKw`, `"Connector"` → `ConnectorKw`, `"Store"` → `StoreKw`, `"Session"` → `SessionKw`, `"Guard"` → `GuardKw`, `"Workflow"` → `WorkflowKw`, `"Schema"` → `SchemaKw`, `"MCP"` → `McpKw`, `"CLI"` → `CliKw`, `"HTTP"` → `HttpKw`.

**ActiveForm:** Adding keyword tokens to lexer

---

### Task 2: Add KeywordKind and KeywordDeclData to AST

**Subject:** Add KeywordKind enum, KeywordDeclData struct, and Stmt::KeywordDecl variant

**Description:** Edit `crates/lx/src/ast/types.rs`: add `KeywordKind` enum with 12 variants (Agent, Tool, Prompt, Connector, Store, Session, Guard, Workflow, Schema, Mcp, Cli, Http). Derive `Debug, Clone, Copy, PartialEq`. Add `KeywordDeclData` struct with fields: `pub keyword: KeywordKind`, `pub name: Sym`, `pub type_params: Vec<Sym>`, `pub fields: Vec<ClassField>`, `pub methods: Vec<AgentMethod>`, `pub exported: bool`. Derive `Debug, Clone, PartialEq`.

Edit `crates/lx/src/ast/mod.rs`: add `#[walk(skip)] KeywordDecl(KeywordDeclData)` variant to the `Stmt` enum.

**ActiveForm:** Adding AST types for keyword declarations

---

### Task 3: Create keyword parser

**Subject:** Create stmt_keyword.rs with unified keyword parser

**Description:** Create `crates/lx/src/parser/stmt_keyword.rs`. Write a `keyword_parser()` function that:

1. Matches any of the 12 keyword tokens using `choice()` over `just(TokenKind::AgentKw)`, `just(TokenKind::ToolKw)`, etc. Map each to its `KeywordKind` variant.
2. Reads the declaration name using the existing `name_or_type()` parser (TypeName).
3. Reads optional type parameters (same pattern as class_parser).
4. Reads `=` token.
5. Reads the class body using the existing class body parser from `stmt_class.rs` — the `class_body()` helper that returns `(Vec<ClassField>, Vec<AgentMethod>)`. If this helper doesn't exist as a standalone function, extract it from `class_parser()` in `stmt_class.rs` first.
6. Handles the optional `+` export prefix (the Export token).
7. Returns `Stmt::KeywordDecl(KeywordDeclData { keyword, name, type_params, fields, methods, exported })`.

Add `mod stmt_keyword;` to `crates/lx/src/parser/mod.rs`.

Edit `crates/lx/src/parser/stmt.rs`: add `stmt_keyword::keyword_parser()` to the `choice()` list in `stmt_parser()`, positioned before `class_parser` and `trait_parser` so keywords match first.

**ActiveForm:** Creating keyword parser

---

### Task 4: Implement simple keyword desugaring

**Subject:** Add transform_stmts to Desugarer for 8 simple keywords

**Description:** Edit `crates/lx/src/folder/desugar.rs`. Override `transform_stmts()` on `Desugarer`. For each statement in the input list:

If the statement is `Stmt::KeywordDecl(data)` and `data.keyword` is one of the 8 simple keywords (Agent, Tool, Prompt, Connector, Store, Session, Guard, Workflow):

1. Determine the import path and trait name from the mapping table (see What Changes).
2. Allocate a `Stmt::Use(UseStmt { path: vec![intern("pkg"), intern("core"), intern("tool")], kind: UseKind::Selective(vec![intern("Tool")]) })` (adjust path per keyword).
3. For Store keyword: check if `data.fields` already contains a field named `entries`. If not, allocate an `entries` field with default value `Expr::Apply(ExprApply { func: <Store ident>, arg: <Unit literal> })`.
4. Allocate a `Stmt::ClassDecl(ClassDeclData { name: data.name, type_params: data.type_params, traits: vec![trait_sym], fields: data.fields, methods: data.methods, exported: data.exported })`.
5. Return both Use and ClassDecl statements.

If the statement is `Stmt::KeywordDecl` with Schema/Mcp/Cli/Http keyword, pass it through unchanged.

All other statements pass through to the existing walk_transform_stmt.

**ActiveForm:** Implementing keyword desugaring in transform_stmts

---

### Task 5: Update validate_core

**Subject:** Assert simple keyword declarations don't survive into Core AST

**Description:** Edit `crates/lx/src/folder/validate_core.rs`. In the visitor that validates no Surface-only constructs remain, add a check: if any `Stmt::KeywordDecl(data)` exists where `data.keyword` is Agent, Tool, Prompt, Connector, Store, Session, Guard, or Workflow, panic with a message like "KeywordDecl({keyword:?}) should have been desugared". Allow Schema, Mcp, Cli, Http to pass through temporarily — they'll be handled by Units 3-5.

**ActiveForm:** Adding keyword validation to validate_core

---

### Task 6: Write tests for simple keyword desugaring

**Subject:** Create test .lx files for all 8 simple keywords

**Description:** Create 8 test files:

`tests/keyword_agent.lx`: `Agent TestAgent = { perceive = (msg) { msg } }`. Instantiate with `a = TestAgent {}`. Call `a.perceive "hello"`. Assert result is "hello". Call `a.think "test"` to verify default Agent methods are injected.

`tests/keyword_tool.lx`: `Tool Echo = { description: "echoes", params: {text: "Str"}, run = (args) { args.text } }`. Instantiate, call run with `{text: "hi"}`, assert result is "hi". Call schema(), assert it returns params.

`tests/keyword_prompt.lx`: `Prompt Greeter = { system: "You greet" }`. Instantiate. Call `with_section "Name" "Alice"`. Call render(). Assert output contains "You greet" and "Alice".

`tests/keyword_connector.lx`: `Connector Custom = { connect = () { Ok () }, disconnect = () { Ok () }, call = (req) { Ok req }, tools = () { [] } }`. Instantiate. Call connect(), assert Ok. Call tools(), assert empty list.

`tests/keyword_store.lx`: `Store Items = {}`. Instantiate. Verify `entries` field exists (auto-injected). Call `get`, `keys`, `len` from Collection trait. Add items via the Store, verify CRUD.

`tests/keyword_session.lx`: `Session Chat = { max_tokens: 1000 }`. Instantiate. Call add_message three times. Check pressure > 0. Checkpoint. Add more. Resume. Verify state.

`tests/keyword_guard.lx`: `Guard TurnLimit = { max_turns: 3 }`. Instantiate. Tick 3 times, check returns Ok. Tick again, check returns Err. Reset, check returns Ok.

`tests/keyword_workflow.lx`: `Workflow TwoStep = { steps: [{id: "a", run: (ctx) { 1 }, depends: []}, {id: "b", run: (ctx) { 2 }, depends: ["a"]}] }`. Instantiate. Call run({}). Assert both steps completed.

Run `just test` to verify all pass.

**ActiveForm:** Writing keyword desugaring tests

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
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_2_SIMPLE_PIPELINE.md" })
```

Then call `next_task` to begin.
