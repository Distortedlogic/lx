# Unit 1: Parser — `use tool` and `use stream` syntax

## Goal

Add two new `use` statement forms to the parser and AST:
- `use tool "command-name" as ModuleName` — binds an MCP tool server as a module
- `use stream {backend: "jsonl", path: ".lx/events.jsonl"}` — configures the runtime event stream

These are syntactic additions only. No interpreter evaluation logic in this unit.

## Preconditions

- `crates/lx/src/ast/types.rs` defines `UseStmt { path: Vec<Sym>, kind: UseKind }` with `UseKind { Whole, Alias(Sym), Selective(Vec<Sym>) }` (lines 7-18)
- `crates/lx/src/parser/stmt.rs` defines `use_parser` (lines 86-117) which parses path segments separated by `/`
- `crates/lx/src/lexer/token.rs` has `TokenKind::Use`, `TokenKind::As`, `TokenKind::ToolKw` (line 89, uppercase "Tool")
- `crates/lx/src/lexer/helpers.rs` has `ident_or_keyword()` (lines 16-33) for lowercase keywords and `type_name_or_keyword()` (lines 35-52) for uppercase
- String literals are parsed via `TokenKind::RawStr(String)` (backtick-delimited) and `TokenKind::StrStart`/`TokenKind::StrEnd` (quote-delimited)
- Record literals are parsed in `crates/lx/src/parser/expr_helpers.rs:30-55` (`block_or_record_parser`)

## Step 1: Add UseKind variants to AST

File: `crates/lx/src/ast/types.rs`

Add two new variants to the `UseKind` enum:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
  Whole,
  Alias(Sym),
  Selective(Vec<Sym>),
  Tool { command: String, alias: Sym },
  Stream(ExprId),
}
```

`Tool.command` holds the string literal (e.g. `"agent-browser"`). `Tool.alias` holds the bound name (e.g. `Browser`).

`Stream(ExprId)` holds the config record expression. The expression will be evaluated at runtime by the interpreter (Unit 5).

This requires adding `ExprId` to the import at line 4: add `ExprId` to the `use super::{ExprId, ...}` import.

## Step 2: Context-sensitive keywords (no lexer changes)

Do NOT add new keyword tokens to the lexer. `tool` and `stream` remain regular identifiers (`Ident(Sym)`) and only have special meaning after `Use` in the parser. This avoids reserving these words globally.

No changes to `crates/lx/src/lexer/helpers.rs`.

Note on alias syntax: the existing `use` alias syntax is `use path : alias` (using `TokenKind::Colon`). The new `use tool` syntax uses `as` (`TokenKind::As`) instead. This is intentional — `use tool "cmd" as Name` reads more naturally and matches the architecture doc's design. Both forms coexist.

## Step 3: Modify use_parser to handle new forms

File: `crates/lx/src/parser/stmt.rs`

The `use_parser` function (line 86) currently takes only `arena: ArenaRef`. It needs to also take the `expr` parser to parse the config record for `use stream`.

### Step 3a: Change use_parser signature

Change the signature from:
```rust
fn use_parser<'a, I>(arena: ArenaRef) -> impl Parser<...>
```
to:
```rust
fn use_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<...>
```

Update the call site at line 44:
```rust
let use_stmt = use_parser(expr.clone(), arena.clone());
```

### Step 3b: Restructure use_parser to branch after Use token

The key insight: consume `Use` once, then branch on the next token. This avoids backtracking issues with chumsky's `choice`.

```rust
fn use_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, StmtId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let tool_sym = intern("tool");
  let stream_sym = intern("stream");

  // String literal parser (for tool command)
  let plain_str = just(TokenKind::StrStart)
    .ignore_then(select! { TokenKind::StrChunk(s) => s })
    .then_ignore(just(TokenKind::StrEnd));
  let raw_str = select! { TokenKind::RawStr(s) => s };
  let command_str = plain_str.or(raw_str);

  // use tool "cmd" as Name
  let a1 = arena.clone();
  let tool_branch = just(TokenKind::Ident(tool_sym))
    .ignore_then(command_str)
    .then_ignore(just(TokenKind::As))
    .then(type_name())
    .map_with(move |(command, alias), e| {
      a1.borrow_mut().alloc_stmt(
        Stmt::Use(UseStmt { path: vec![], kind: UseKind::Tool { command, alias } }),
        ss(e.span()),
      )
    });

  // use stream {config}
  let a2 = arena.clone();
  let stream_branch = just(TokenKind::Ident(stream_sym))
    .ignore_then(expr)
    .map_with(move |config_expr, e| {
      a2.borrow_mut().alloc_stmt(
        Stmt::Use(UseStmt { path: vec![], kind: UseKind::Stream(config_expr) }),
        ss(e.span()),
      )
    });

  // Existing path-based use (unchanged logic from current lines 90-116)
  let path_seg = super::expr::ident_or_keyword();
  let dotdot_prefix = just(TokenKind::DotDot).then_ignore(just(TokenKind::Slash)).to(intern(".."));
  let dot_prefix = just(TokenKind::Dot).then_ignore(just(TokenKind::Slash)).to(intern("."));
  let prefix_parts = dotdot_prefix.repeated().collect::<Vec<_>>().then(dot_prefix.or_not()).map(|(mut dd, dot)| {
    if let Some(d) = dot { dd.push(d); }
    dd
  });
  let segments = path_seg.separated_by(just(TokenKind::Slash)).at_least(1).collect::<Vec<_>>();
  let alias = just(TokenKind::Colon).ignore_then(ident()).map(UseKind::Alias);
  let selective = name_or_type()
    .separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)).or_not())
    .collect::<Vec<_>>()
    .delimited_by(just(TokenKind::LBrace), just(TokenKind::RBrace))
    .map(UseKind::Selective);
  let kind = alias.or(selective).or_not().map(|k| k.unwrap_or(UseKind::Whole));
  let a3 = arena.clone();
  let path_branch = prefix_parts.then(segments).then(kind).map_with(move |((mut prefix, segs), kind), e| {
    prefix.extend(segs);
    a3.borrow_mut().alloc_stmt(Stmt::Use(UseStmt { path: prefix, kind }), ss(e.span()))
  });

  // Consume Use once, then branch
  just(TokenKind::Use).ignore_then(
    tool_branch.or(stream_branch).or(path_branch)
  )
}
```

This replaces the entire `use_parser` function body. The three branches after `Use` are tried in order: `tool_branch` checks for `Ident("tool")`, `stream_branch` checks for `Ident("stream")`, and `path_branch` handles the existing path-based syntax. Since each branch starts with a different token, there's no ambiguity.

## Step 4: Handle new UseKind variants in existing code

### 4a: Interpreter — eval_use placeholder

File: `crates/lx/src/interpreter/modules.rs:43-59`

In the match on `&use_stmt.kind`, add placeholder arms before the existing arms:

```rust
UseKind::Tool { .. } => {
  return Err(LxError::runtime("use tool not yet implemented", span));
},
UseKind::Stream(_) => {
  return Err(LxError::runtime("use stream not yet implemented", span));
},
```

These placeholders will be replaced in Units 3 and 5.

### 4b: Desugarer — walk the Stream config expression

File: `crates/lx/src/ast/mod.rs:45-46`

`Stmt::Use(UseStmt)` has `#[walk(skip)]`, which means the `AstWalk` derive macro's `recurse_children` does NOT recurse into `UseStmt`. This means expressions inside `UseKind::Stream(ExprId)` will NOT be desugared (e.g., string interpolation in the config record won't be transformed).

Fix: In `crates/lx/src/folder/desugar.rs`, modify the `Desugarer::transform_stmts` method to manually handle `Stmt::Use` with `UseKind::Stream`:

```rust
// In transform_stmts, add a match arm before the generic `_ =>` arm:
Stmt::Use(ref use_stmt) => {
  if let UseKind::Stream(config_expr) = use_stmt.kind {
    // Walk the config expression through the desugarer
    let new_expr = crate::visitor::walk_transform::walk_transform_expr(self, config_expr, arena);
    let new_stmt = Stmt::Use(UseStmt {
      path: use_stmt.path.clone(),
      kind: UseKind::Stream(new_expr),
    });
    let new_sid = arena.alloc_stmt(new_stmt, span);
    result.push(new_sid);
  } else {
    let transformed = crate::visitor::walk_transform::walk_transform_stmt(self, sid, arena);
    result.push(transformed);
  }
}
```

Add `UseKind` and `UseStmt` to the imports at `desugar.rs:8` if not already present (they are already imported).

### 4c: Grep for all UseKind match sites

Run: `rg 'UseKind::' --type rust crates/` to find all locations. Each must handle `Tool` and `Stream`. The known locations are:
- `interpreter/modules.rs:43` — handled in 4a
- Any visitor code — `Stmt::Use` has `#[walk(skip)]` at `ast/mod.rs:45`, no walking needed
- The `collect_exports` function at `interpreter/modules.rs:220` does NOT match on UseKind (it matches on Stmt variants), so no change needed there

## Step 5: Add test programs

Create test `.lx` files that verify parsing:

File: `tests/suite/use_tool_parse.lx`
```lx
-- goal: verify `use tool` parses without error
-- (will fail at runtime since tool module isn't implemented yet,
--  but should parse successfully)
use tool "echo-server" as Echo
```

File: `tests/suite/use_stream_parse.lx`
```lx
-- goal: verify `use stream` parses without error
use stream {backend: "jsonl", path: ".lx/events.jsonl"}
```

These tests verify the parser accepts the new syntax. They will produce runtime errors ("not yet implemented") which is expected — the runtime support comes in Units 3 and 5.

## Verification

Run `just diagnose` — no compiler errors or clippy warnings. The new `UseKind` variants should compile and all existing tests should pass unchanged.
