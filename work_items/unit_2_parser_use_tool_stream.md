# Unit 2: Parser + AST for `use tool` and `use stream`

Add two new `use` statement forms to the parser and AST. Syntactic additions only -- no interpreter evaluation logic.

## Prerequisites

None. This unit has no dependencies on other units.

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify

## Current State

- `UseStmt` is defined in `crates/lx/src/ast/types.rs` (lines 7-18) with `path: Vec<Sym>` and `kind: UseKind`
- `UseKind` has three variants: `Whole`, `Alias(Sym)`, `Selective(Vec<Sym>)`
- `use_parser` is in `crates/lx/src/parser/stmt.rs` (lines 86-117), takes only `arena: ArenaRef`
- Lowercase `tool` lexes as `Ident(intern("tool"))`, not `ToolKw`. The parser matches `Ident` where the sym equals `intern("tool")`.
- `TokenKind::Use` is on line 72 of token.rs
- `TokenKind::As` is on line 86 of token.rs
- `stream` is a regular identifier `TokenKind::Ident(intern("stream"))`
- The `Stmt::Use(UseStmt)` variant has `#[walk(skip)]` in `crates/lx/src/ast/mod.rs` (line 46), meaning the walker does not recurse into it
- String literals parse as `StrStart` + `StrChunk(content)` + `StrEnd` for quoted strings, or `RawStr(String)` for backtick strings

## Files to Modify

1. `crates/lx/src/ast/types.rs` -- add `UseKind::Tool` and `UseKind::Stream` variants
2. `crates/lx/src/parser/stmt.rs` -- extend `use_parser` to parse new forms, change signature to accept expr parser
3. `crates/lx/src/interpreter/modules.rs` -- add placeholder match arms for new `UseKind` variants (lines 43-59)
4. `crates/lx/src/formatter/emit_stmt.rs` -- add formatting for new `UseKind` variants (lines 254-279)
5. `crates/lx/src/linter/rules/unused_import.rs` -- add match arms for new `UseKind` variants (lines 44-48)
6. `crates/lx/src/checker/visit_stmt.rs` -- add match arms for new `UseKind` variants (lines 115-168)

## Step 1: Add UseKind variants to AST

File: `crates/lx/src/ast/types.rs`

### 1a: Add ExprId to the import on line 4

Current line 4:
```rust
use super::{ExprId, Literal, PatternId, TypeExprId};
```

`ExprId` is already imported. No change needed.

### 1b: Add two new variants to UseKind enum

Current (lines 13-18):
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
  Whole,
  Alias(Sym),
  Selective(Vec<Sym>),
}
```

Change to:
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

`Tool { command, alias }`: `command` is the string from `use tool "command-name" as Alias` (e.g. `"agent-browser"`). `alias` is the bound module name (e.g. `Browser`).

`Stream(ExprId)`: holds the config record expression that will be evaluated at runtime (e.g. `{backend: "jsonl", path: ".lx/events.jsonl"}`).

## Step 2: Modify use_parser to handle new forms

File: `crates/lx/src/parser/stmt.rs`

### 2a: Change use_parser signature

The `use_parser` function (line 86) currently takes only `arena: ArenaRef`. It needs the `expr` parser to parse the config record for `use stream`.

Change the signature from:
```rust
fn use_parser<'a, I>(arena: ArenaRef) -> impl Parser<'a, I, StmtId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
```
to:
```rust
fn use_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, StmtId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
```

### 2b: Update the call site

Line 44 currently reads:
```rust
let use_stmt = use_parser(arena.clone());
```

Change to:
```rust
let use_stmt = use_parser(expr.clone(), arena.clone());
```

### 2c: Add imports

At line 4, the existing import is:
```rust
use super::expr::{ident, name_or_type, type_name};
```

`use crate::sym::{Sym, intern};` is already at line 11. No new imports needed.

### 2d: Restructure use_parser body

Replace the entire function body of `use_parser` (lines 86-117). The new body consumes `Use` once, then branches on the next token:

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

  let plain_str = just(TokenKind::StrStart)
    .ignore_then(select! { TokenKind::StrChunk(s) => s })
    .then_ignore(just(TokenKind::StrEnd));
  let raw_str = select! { TokenKind::RawStr(s) => s };
  let command_str = plain_str.or(raw_str);

  let a_tool = arena.clone();
  let tool_branch = just(TokenKind::Ident(tool_sym))
    .ignore_then(command_str)
    .then_ignore(just(TokenKind::As))
    .then(type_name())
    .map_with(move |(command, alias), e| {
      a_tool.borrow_mut().alloc_stmt(
        Stmt::Use(UseStmt { path: vec![], kind: UseKind::Tool { command, alias } }),
        ss(e.span()),
      )
    });

  let a_stream = arena.clone();
  let stream_branch = just(TokenKind::Ident(stream_sym))
    .ignore_then(expr)
    .map_with(move |config_expr, e| {
      a_stream.borrow_mut().alloc_stmt(
        Stmt::Use(UseStmt { path: vec![], kind: UseKind::Stream(config_expr) }),
        ss(e.span()),
      )
    });

  let path_seg = super::expr::ident_or_keyword();
  let dotdot_prefix = just(TokenKind::DotDot).then_ignore(just(TokenKind::Slash)).to(intern(".."));
  let dot_prefix = just(TokenKind::Dot).then_ignore(just(TokenKind::Slash)).to(intern("."));
  let prefix_parts = dotdot_prefix.repeated().collect::<Vec<_>>().then(dot_prefix.or_not()).map(|(mut dd, dot)| {
    if let Some(d) = dot {
      dd.push(d);
    }
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

  let path_branch = prefix_parts.then(segments).then(kind).map_with(move |((mut prefix, segs), kind), e| {
    prefix.extend(segs);
    arena.borrow_mut().alloc_stmt(Stmt::Use(UseStmt { path: prefix, kind }), ss(e.span()))
  });

  just(TokenKind::Use).ignore_then(
    tool_branch
      .or(stream_branch)
      .or(path_branch)
  )
}
```

## Step 3: Handle new UseKind variants in eval_use

File: `crates/lx/src/interpreter/modules.rs`

The `UseKind::Tool` and `UseKind::Stream` variants bypass the module resolution logic since they have `path: vec![]`. Two changes are required:

### 3a: Insert early-return guards before line 16 (before `let str_path: ...`):

```rust
if let UseKind::Tool { ref command, alias } = use_stmt.kind {
    return Err(LxError::runtime("use tool not yet implemented", span));
}
if let UseKind::Stream(_) = use_stmt.kind {
    return Err(LxError::runtime("use stream not yet implemented", span));
}
```

These are replaced by Unit 3 (Tool) and Unit 5 (Stream) respectively.

### 3b: Add exhaustive match arms at line 43

The match on `&use_stmt.kind` at line 43 must cover all variants or the compiler rejects it, even though the guards above guarantee `Tool` and `Stream` never reach it. Add a catch-all after the existing three arms:

```rust
    match &use_stmt.kind {
      UseKind::Whole => { ... },       // existing
      UseKind::Alias(alias) => { ... }, // existing
      UseKind::Selective(names) => { ... }, // existing
      UseKind::Tool { .. } | UseKind::Stream(_) => unreachable!(),
    }
```

## Step 4: Handle new UseKind variants in formatter

File: `crates/lx/src/formatter/emit_stmt.rs`

Restructure the `emit_use` method (lines 254-279). For Tool/Stream, the path is empty, so handle them before path output:

```rust
fn emit_use(&mut self, u: &UseStmt) {
    self.write("use ");
    match &u.kind {
        UseKind::Tool { ref command, alias } => {
            self.write("tool \"");
            self.write(command);
            self.write("\" as ");
            self.write(alias.as_str());
        },
        UseKind::Stream(config_expr) => {
            self.write("stream ");
            self.emit_expr(*config_expr);
        },
        _ => {
            for (i, seg) in u.path.iter().enumerate() {
                if i > 0 {
                    self.write("/");
                }
                self.write(seg.as_str());
            }
            match &u.kind {
                UseKind::Whole => {},
                UseKind::Alias(alias) => {
                    self.write(" : ");
                    self.write(alias.as_str());
                },
                UseKind::Selective(names) => {
                    self.write(" { ");
                    for (i, n) in names.iter().enumerate() {
                        if i > 0 {
                            self.write("; ");
                        }
                        self.write(n.as_str());
                    }
                    self.write(" }");
                },
                UseKind::Tool { .. } | UseKind::Stream(_) => unreachable!(),
            }
        },
    }
}
```

## Step 5: Handle new UseKind variants in linter

File: `crates/lx/src/linter/rules/unused_import.rs`

In the match at lines 44-48, add arms:

```rust
UseKind::Tool { alias, .. } => vec![*alias],
UseKind::Stream(_) => vec![intern("stream")],
```

Add `use crate::sym::intern;` to the imports at the top of the file.

## Step 6: Handle new UseKind variants in checker

File: `crates/lx/src/checker/visit_stmt.rs`

In the match at lines 115-168 (on `&u.kind`), add arms after the `UseKind::Selective` block:

```rust
UseKind::Tool { alias, .. } => {
    let def_id = self.sem.add_definition(*alias, DefKind::Import, span, false);
    self.sem.set_definition_type(def_id, unknown);
},
UseKind::Stream(_) => {
    let stream_name = crate::sym::intern("stream");
    let def_id = self.sem.add_definition(stream_name, DefKind::Import, span, false);
    self.sem.set_definition_type(def_id, unknown);
},
```

The `unknown` type variable is already available in scope at that point in the function.

## Step 7: Handle UseKind::Stream in AST walk

File: `crates/lx/src/ast/mod.rs`

Keep `#[walk(skip)]` on `Stmt::Use` and do NOT change the walker. Config expressions are plain record literals. The walker skips `UseKind::Stream` and `UseKind::Tool`.

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. All existing tests pass unchanged (run `just test`)
3. The new UseKind variants compile and all exhaustive match sites are covered
