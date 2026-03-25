# Goal

Add 12 keyword tokens to the lexer, a `KeywordDecl` AST node, a unified keyword parser, and desugaring for 8 simple keywords (Agent, Tool, Prompt, Connector, Store, Session, Guard, Workflow) into `Class : [Trait]` with auto-injected `use` import. Also update all 6 exhaustive `match stmt` sites to handle the new variant.

# Why

This is the core compiler pipeline. After this unit, `Agent MyAgent = { ... }` produces identical runtime behavior to writing `use pkg/agent {Agent}` then `Class MyAgent : [Agent] = { ... }`.

# What Changes

**Lexer:** 12 new `TokenKind` variants. 12 match arms in `type_name_or_keyword()`.

**AST:** `KeywordKind` enum (12 variants), `KeywordDeclData` struct, `Stmt::KeywordDecl` variant with `#[walk(skip)]`.

**Parser:** New `stmt_keyword.rs`. The class body parser is currently **inlined** in `class_parser()` at `stmt_class.rs:24-51` — it must be extracted into a standalone `class_body()` helper first. The trait body parser already exists as `trait_body()` at `stmt.rs:222`. The keyword parser delegates to `class_body()` for all keywords except Schema (which uses `trait_body()`).

**Desugar:** Override `transform_stmts` on `Desugarer`. Each simple keyword becomes a `Stmt::Use` + `Stmt::ClassDecl`.

**Match sites:** Adding `KeywordDecl` to `Stmt` will cause compile errors in 6 exhaustive match locations. Since KeywordDecl is always desugared before any of these run, each gets `Stmt::KeywordDecl(_) => unreachable!("desugared")`:

1. `formatter/emit_stmt.rs:8` — formatter
2. `visitor/walk/mod.rs:99` — AST visitor walk_stmt
3. `checker/visit_stmt.rs:27` — type checker check_stmt
4. `checker/check_expr.rs:159` — type checker block tail typing
5. `checker/module_graph.rs:24` — module signature extraction
6. `interpreter/exec_stmt.rs:27` — interpreter eval_stmt

The `#[derive(AstWalk)]` on Stmt handles the new variant automatically via `#[walk(skip)]`.

# Files Affected

- `crates/lx/src/lexer/token.rs` — Add 12 TokenKind variants
- `crates/lx/src/lexer/helpers.rs` — Add 12 keyword match arms
- `crates/lx/src/ast/types.rs` — Add KeywordKind, KeywordDeclData
- `crates/lx/src/ast/mod.rs` — Add Stmt::KeywordDecl
- `crates/lx/src/parser/stmt_class.rs` — Extract class_body() helper
- `crates/lx/src/parser/stmt_keyword.rs` — New file
- `crates/lx/src/parser/mod.rs` — Add mod stmt_keyword
- `crates/lx/src/parser/stmt.rs` — Add keyword_parser to choice, before class_stmt and trait_stmt
- `crates/lx/src/folder/desugar.rs` — Add transform_stmts
- `crates/lx/src/folder/validate_core.rs` — Assert simple keywords desugared
- `crates/lx/src/formatter/emit_stmt.rs` — Add unreachable arm
- `crates/lx/src/visitor/walk/mod.rs` — Add unreachable arm
- `crates/lx/src/checker/visit_stmt.rs` — Add unreachable arm
- `crates/lx/src/checker/check_expr.rs` — Add KeywordDecl to unit-typed arm
- `crates/lx/src/checker/module_graph.rs` — Add to catch-all arm
- `crates/lx/src/interpreter/exec_stmt.rs` — Add unreachable arm
- `tests/keyword_agent.lx` — New test
- `tests/keyword_tool.lx` — New test
- `tests/keyword_store.lx` — New test
- `tests/keyword_guard.lx` — New test

# Task List

### Task 1: Add keyword tokens to lexer

**Subject:** Add 12 keyword TokenKind variants and match arms

**Description:** Edit `crates/lx/src/lexer/token.rs`. Add to the `TokenKind` enum (in the keywords section near lines 78-84):

```rust
AgentKw, ToolKw, PromptKw, ConnectorKw, StoreKw,
SessionKw, GuardKw, WorkflowKw, SchemaKw,
McpKw, CliKw, HttpKw,
```

Edit `crates/lx/src/lexer/helpers.rs`. In `type_name_or_keyword()` (lines 35-41), add match arms:

```rust
"Agent" => TokenKind::AgentKw,
"Tool" => TokenKind::ToolKw,
"Prompt" => TokenKind::PromptKw,
"Connector" => TokenKind::ConnectorKw,
"Store" => TokenKind::StoreKw,
"Session" => TokenKind::SessionKw,
"Guard" => TokenKind::GuardKw,
"Workflow" => TokenKind::WorkflowKw,
"Schema" => TokenKind::SchemaKw,
"MCP" => TokenKind::McpKw,
"CLI" => TokenKind::CliKw,
"HTTP" => TokenKind::HttpKw,
```

These go in `type_name_or_keyword` (not `ident_or_keyword`) because all 12 keywords are capitalized type-level names, same as `Class` and `Trait`.

**ActiveForm:** Adding keyword tokens

---

### Task 2: Add AST types and Stmt variant

**Subject:** Add KeywordKind, KeywordDeclData, Stmt::KeywordDecl

**Description:** Edit `crates/lx/src/ast/types.rs`. Add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordKind {
    Agent, Tool, Prompt, Connector, Store,
    Session, Guard, Workflow, Schema,
    Mcp, Cli, Http,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeywordDeclData {
    pub keyword: KeywordKind,
    pub name: Sym,
    pub type_params: Vec<Sym>,
    pub fields: Vec<ClassField>,
    pub methods: Vec<AgentMethod>,
    pub trait_entries: Option<Vec<TraitEntry>>,
    pub exported: bool,
}
```

The `trait_entries` field is `Some` only for Schema keyword (trait-body syntax). All other keywords use `fields` + `methods` (class-body syntax) and leave `trait_entries` as `None`.

Edit `crates/lx/src/ast/mod.rs`. Add to the `Stmt` enum:

```rust
#[walk(skip)]
KeywordDecl(KeywordDeclData),
```

Add `KeywordDeclData` and `KeywordKind` to the pub use exports at the top of mod.rs if needed.

**ActiveForm:** Adding AST types

---

### Task 3: Update all 6 exhaustive match sites

**Subject:** Add KeywordDecl arms to prevent compile errors

**Description:** The following 6 files have exhaustive matches on `Stmt` that will fail to compile. Add the appropriate arm to each:

1. `crates/lx/src/formatter/emit_stmt.rs:8` — Add: `Stmt::KeywordDecl(_) => unreachable!("keyword not desugared"),`

2. `crates/lx/src/visitor/walk/mod.rs:99` — In `walk_stmt`, add: `Stmt::KeywordDecl(_) => {},` (no-op, since walk(skip) means we never descend)

3. `crates/lx/src/checker/visit_stmt.rs:27` — In `check_stmt`, add: `Stmt::KeywordDecl(_) => self.type_arena.unit(),`

4. `crates/lx/src/checker/check_expr.rs:159` — Add `Stmt::KeywordDecl(_)` to the existing multi-variant arm that returns unit type (the arm with `Stmt::Binding(_) | Stmt::TypeDef(_) | ...`)

5. `crates/lx/src/checker/module_graph.rs:24` — Add `Stmt::KeywordDecl(_)` to the catch-all arm (around line 43-50)

6. `crates/lx/src/interpreter/exec_stmt.rs:27` — In `eval_stmt`, add: `Stmt::KeywordDecl(_) => unreachable!("keyword not desugared"),`

After these changes, `cargo check` should pass.

**ActiveForm:** Updating match sites

---

### Task 4: Extract class_body helper from stmt_class.rs

**Subject:** Extract the inlined class body parser into a reusable function

**Description:** The class body parser is currently inlined in `class_parser()` at `crates/lx/src/parser/stmt_class.rs:24-51`. The keyword parser needs to reuse it. Extract it.

The body parser is the section that parses `{ field: default, method = handler }` after the `=` token. It currently:
1. Defines `ClassMember` enum (Field, Method) at lines 55-59
2. Defines member parsers: `class_field` (name `:` expr) and `class_method` (name `=` expr) at lines 24-29
3. Parses delimited `{ members }` with skip_semis at lines 37-41
4. Splits members into `(Vec<ClassField>, Vec<AgentMethod>)` at lines 42-50

Extract into a public function:

```rust
pub fn class_body<'a, I>(
    expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, (Vec<ClassField>, Vec<AgentMethod>), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
    I: ValueInput<'a, Token = TokenKind, Span = Span>,
```

Move the `ClassMember` enum inside or alongside this function. The function parses `{ ... }` (including braces) and returns `(Vec<ClassField>, Vec<AgentMethod>)`.

Update `class_parser()` to call `class_body(expr)` instead of inlining the logic.

Verify `cargo check` passes after this refactor.

**ActiveForm:** Extracting class body parser

---

### Task 5: Create keyword parser

**Subject:** Create stmt_keyword.rs

**Description:** Create `crates/lx/src/parser/stmt_keyword.rs`:

```rust
use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::expr::type_name;
use super::stmt_class::class_body;
use super::{ArenaRef, Span, StmtId, ss};
use crate::ast::{KeywordDeclData, KeywordKind, Stmt};
use crate::lexer::token::TokenKind;

pub fn keyword_parser<'a, I>(
    expr: impl Parser<'a, I, crate::ast::ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
    arena: ArenaRef,
) -> impl Parser<'a, I, StmtId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
    I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
    let keyword = choice((
        just(TokenKind::AgentKw).to(KeywordKind::Agent),
        just(TokenKind::ToolKw).to(KeywordKind::Tool),
        just(TokenKind::PromptKw).to(KeywordKind::Prompt),
        just(TokenKind::ConnectorKw).to(KeywordKind::Connector),
        just(TokenKind::StoreKw).to(KeywordKind::Store),
        just(TokenKind::SessionKw).to(KeywordKind::Session),
        just(TokenKind::GuardKw).to(KeywordKind::Guard),
        just(TokenKind::WorkflowKw).to(KeywordKind::Workflow),
        just(TokenKind::SchemaKw).to(KeywordKind::Schema),
        just(TokenKind::McpKw).to(KeywordKind::Mcp),
        just(TokenKind::CliKw).to(KeywordKind::Cli),
        just(TokenKind::HttpKw).to(KeywordKind::Http),
    ));

    keyword
        .then(type_name())
        .then_ignore(just(TokenKind::Assign))
        .then(class_body(expr))
        .map_with(move |((kw, name), (fields, methods)), e| {
            let data = KeywordDeclData {
                keyword: kw,
                name,
                type_params: vec![],
                fields,
                methods,
                trait_entries: None,
                exported: false,
            };
            arena.borrow_mut().alloc_stmt(Stmt::KeywordDecl(data), ss(e.span()))
        })
}
```

Note: Schema keyword also uses `class_body` in this unit. Unit 3 will modify it to use `trait_body` instead. For now, Schema parses with class syntax — Unit 3 fixes this.

Add `pub mod stmt_keyword;` to `crates/lx/src/parser/mod.rs`.

Edit `crates/lx/src/parser/stmt.rs`. In `stmt_parser()`, add to the `choice()` at line 54, **before** the trait_stmt and class_stmt entries:

```rust
exported.clone().then(stmt_keyword::keyword_parser(expr.clone(), a_kw.clone())).map_with(move |(exp, sid), _e| {
    // set exported flag on the KeywordDeclData
    let stmt = a_kw_ref.borrow_mut().stmt_mut(sid);
    if let Stmt::KeywordDecl(ref mut d) = stmt { d.exported = exp; }
    sid
}),
```

You'll need an additional `ArenaRef` clone (`a_kw`) for the keyword parser, following the same pattern used for `a2`, `a3`, etc. in the existing choice arms.

**ActiveForm:** Creating keyword parser

---

### Task 6: Implement simple keyword desugaring

**Subject:** Add transform_stmts to Desugarer for 8 simple keywords

**Description:** Edit `crates/lx/src/folder/desugar.rs`. Add a `transform_stmts` override to the `Desugarer` impl:

```rust
fn transform_stmts(&mut self, stmts: Vec<StmtId>, arena: &mut AstArena) -> Vec<StmtId> {
    let mut result = Vec::new();
    for sid in stmts {
        let span = arena.stmt_span(sid);
        let stmt = arena.stmt(sid).clone();
        match stmt {
            Stmt::KeywordDecl(data) => {
                let desugared = desugar_keyword(data, span, arena);
                result.extend(desugared);
            }
            _ => {
                let transformed = super::walk_transform::walk_transform_stmt(self, sid, arena);
                result.push(transformed);
            }
        }
    }
    result
}
```

Then implement `desugar_keyword`:

```rust
fn desugar_keyword(data: KeywordDeclData, span: SourceSpan, arena: &mut AstArena) -> Vec<StmtId> {
    let (import_path, trait_name) = match data.keyword {
        KeywordKind::Agent => (vec!["pkg", "agent"], "Agent"),
        KeywordKind::Tool => (vec!["pkg", "core", "tool"], "Tool"),
        KeywordKind::Prompt => (vec!["pkg", "core", "prompt_trait"], "Prompt"),
        KeywordKind::Connector => (vec!["pkg", "core", "connector"], "Connector"),
        KeywordKind::Store => (vec!["pkg", "core", "collection"], "Collection"),
        KeywordKind::Session => (vec!["pkg", "core", "session"], "Session"),
        KeywordKind::Guard => (vec!["pkg", "core", "guard"], "Guard"),
        KeywordKind::Workflow => (vec!["pkg", "core", "workflow"], "Workflow"),
        // Schema, Mcp, Cli, Http handled by later units — pass through
        _ => return vec![arena.alloc_stmt(Stmt::KeywordDecl(data), span)],
    };

    let trait_sym = intern(trait_name);
    let path: Vec<Sym> = import_path.iter().map(|s| intern(s)).collect();

    // Generate: use pkg/... {TraitName}
    let use_stmt = arena.alloc_stmt(
        Stmt::Use(UseStmt { path, kind: UseKind::Selective(vec![trait_sym]) }),
        span,
    );

    let mut fields = data.fields;
    let methods = data.methods;

    // Store keyword: inject `entries: Store()` if not present
    if data.keyword == KeywordKind::Store {
        let has_entries = fields.iter().any(|f| f.name == intern("entries"));
        if !has_entries {
            let store_ident = arena.alloc_expr(Expr::Ident(intern("Store")), span);
            let unit = arena.alloc_expr(Expr::Literal(Literal::Unit), span);
            let store_call = arena.alloc_expr(Expr::Apply(ExprApply { func: store_ident, arg: unit }), span);
            fields.insert(0, ClassField { name: intern("entries"), default: store_call });
        }
    }

    // Generate: Class Name : [Trait] = { fields, methods }
    let class_stmt = arena.alloc_stmt(
        Stmt::ClassDecl(ClassDeclData {
            name: data.name,
            type_params: data.type_params,
            traits: vec![trait_sym],
            fields,
            methods,
            exported: data.exported,
        }),
        span,
    );

    vec![use_stmt, class_stmt]
}
```

Add required imports at top of file: `UseStmt`, `UseKind`, `ClassDeclData`, `ClassField`, `KeywordKind`, `KeywordDeclData`, `ExprApply`, `Literal`, `intern`.

**ActiveForm:** Implementing keyword desugaring

---

### Task 7: Update validate_core

**Subject:** Assert simple keywords desugared in Core AST

**Description:** Edit `crates/lx/src/folder/validate_core.rs`. The validator visits all statements. Add a check: if encountering `Stmt::KeywordDecl(data)` where `data.keyword` is Agent, Tool, Prompt, Connector, Store, Session, Guard, or Workflow, panic with `"KeywordDecl({:?}) should have been desugared"`. Allow Schema, Mcp, Cli, Http to pass through for now.

Implementation: Add a `visit_stmt` override (or update the existing visitor) that checks for KeywordDecl. The validate_core pass currently only checks expressions via `visit_expr`. Add statement checking.

**ActiveForm:** Updating validate_core

---

### Task 8: Write keyword tests

**Subject:** Create test files for 4 representative keywords

**Description:** Create four test files that verify end-to-end keyword functionality:

`tests/keyword_agent.lx`:
```lx
Agent TestAgent = {
  perceive = (msg) { {intent: msg} }
}

a = TestAgent {}
result = a.perceive "hello"
assert result.intent == "hello"
assert (methods_of a | any? (== "think"))
assert (methods_of a | any? (== "handle"))
```

`tests/keyword_tool.lx`:
```lx
Tool Echo = {
  description: "echoes"
  params: {text: "Str"}
  run = (args) { Ok args.text }
}

t = Echo {}
assert t.description == "echoes"
r = t.run {text: "hi"}
assert r == Ok "hi"
v = t.validate {text: "hello"}
assert (v | ok?)
bad = t.validate {}
assert (bad | err?)
```

`tests/keyword_store.lx`:
```lx
Store Items = {}

s = Items {}
s.entries.set "a" 1
s.entries.set "b" 2
assert (s.len ()) == 2
assert (s.has "a")
assert (s.get "a") == 1
```

`tests/keyword_guard.lx`:
```lx
Guard TurnLimit = { max_turns: 3 }

g = TurnLimit {}
g.tick ()
g.tick ()
g.tick ()
assert (g.check () | ok?)
g.tick ()
assert (g.is_tripped ())
```

Run `just test`.

**ActiveForm:** Writing keyword tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_2_SIMPLE_PIPELINE.md" })
```
