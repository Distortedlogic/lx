# Work Item: `use tool` Parser and AST

## Goal

Add parser support for `use tool "command" as Name` syntax. This is a parse-only change: the new `UseKind::Tool` variant is recognized, stored in the AST, handled by the formatter, and returns a stub error in `eval_use`. No runtime behavior.

## Preconditions

- `crates/lx/src/ast/types.rs` exists with `UseStmt` (fields: `path: Vec<Sym>`, `kind: UseKind`) and `UseKind` enum (variants: `Whole`, `Alias(Sym)`, `Selective(Vec<Sym>)`)
- `crates/lx/src/parser/stmt.rs` exists with `use_parser()` function (lines 86-117)
- `crates/lx/src/interpreter/modules.rs` exists with `eval_use()` method (line 20)
- `crates/lx/src/formatter/emit_stmt.rs` exists with `emit_use()` method (lines 254-279)
- `crates/lx/src/lexer/token.rs` exists with `TokenKind::ToolKw` (line 89) and `TokenKind::As` (line 86)
- Token `ToolKw` is the keyword `tool` (lowercase) used in keyword declarations like `Tool Foo = {...}`. It is distinct from `use std/tool` which is parsed as path segments.

## Files to Modify

### 1. `crates/lx/src/ast/types.rs`

**Change the `UseKind` enum** (lines 15-20). Add a new variant:

Current:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
  Whole,
  Alias(Sym),
  Selective(Vec<Sym>),
}
```

New:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
  Whole,
  Alias(Sym),
  Selective(Vec<Sym>),
  Tool { command: Sym, alias: Sym },
}
```

The `command` field holds the string literal value (e.g., `"agent-browser"`). The `alias` field holds the identifier after `as` (e.g., `Browser`).

### 2. `crates/lx/src/parser/stmt.rs`

**Modify `use_parser()`** (lines 86-117). Add a new branch that matches `use tool "string" as Ident` BEFORE the existing path-based parsing.

The `use tool` form is distinguished from `use std/tool` by the token sequence: `Use` followed by `ToolKw` followed by a string literal (not a path segment).

**Current `use_parser` returns a single parser.** Replace it with a `choice` of two parsers: the new `use tool` parser and the existing path-based parser.

**New `use tool` branch:**

```rust
let use_tool = just(TokenKind::Use)
    .ignore_then(just(TokenKind::ToolKw))
    .ignore_then(raw_string())
    .then_ignore(just(TokenKind::As))
    .then(name_or_type())
    .map_with(move |(command_str, alias), e| {
        let command = intern(&command_str);
        arena_tool.borrow_mut().alloc_stmt(
            Stmt::Use(UseStmt {
                path: vec![],
                kind: UseKind::Tool { command, alias },
            }),
            ss(e.span()),
        )
    });
```

Where `raw_string()` parses a simple string literal token. Looking at the lexer tokens, strings are parsed as `StrStart`, `StrChunk(String)`, `StrEnd`. So the string parser is:

```rust
let raw_string = just(TokenKind::StrStart)
    .ignore_then(select! { TokenKind::StrChunk(s) => s })
    .then_ignore(just(TokenKind::StrEnd));
```

This handles simple non-interpolated strings like `"agent-browser"`.

**Implementation steps for `use_parser`:**

1. Clone `arena` one more time at the top: `let arena_tool = arena.clone();`

2. Define `raw_string` parser inside `use_parser`.

3. Define the `use_tool` parser as shown above. The `path` field is `vec![]` because tool imports don't have a module path. The `UseStmt` needs the `path` field to exist but it's empty for tool imports.

4. Wrap the return in `choice((use_tool, existing_path_parser))`. The existing path parser is everything from `just(TokenKind::Use).ignore_then(prefix_parts)...` through the final `.map_with(...)`.

**Full revised `use_parser`:**

```rust
fn use_parser<'a, I>(arena: ArenaRef) -> impl Parser<'a, I, StmtId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let arena_tool = arena.clone();

  let raw_string = just(TokenKind::StrStart)
      .ignore_then(select! { TokenKind::StrChunk(s) => s })
      .then_ignore(just(TokenKind::StrEnd));

  let use_tool = just(TokenKind::Use)
      .ignore_then(just(TokenKind::ToolKw))
      .ignore_then(raw_string)
      .then_ignore(just(TokenKind::As))
      .then(name_or_type())
      .map_with(move |(command_str, alias), e| {
          let command = intern(&command_str);
          arena_tool.borrow_mut().alloc_stmt(
              Stmt::Use(UseStmt {
                  path: vec![],
                  kind: UseKind::Tool { command, alias },
              }),
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

  let use_path = just(TokenKind::Use).ignore_then(prefix_parts).then(segments).then(kind).map_with(move |((mut prefix, segs), kind), e| {
    prefix.extend(segs);
    arena.borrow_mut().alloc_stmt(Stmt::Use(UseStmt { path: prefix, kind }), ss(e.span()))
  });

  use_tool.or(use_path)
}
```

**Required imports in `stmt.rs`:** `intern` is already imported from `crate::sym`. `name_or_type` is already imported from `super::expr`. The `select!` macro comes from `chumsky::prelude::*` which is already imported. `Stmt` and `UseKind` are already imported. Verify `As` token is accessible — `TokenKind::As` exists at line 86 of `token.rs`.

### 3. `crates/lx/src/interpreter/modules.rs`

**Modify `eval_use()`** (starts at line 20). Add a match arm at the TOP of the method, before the module resolution logic, to handle `UseKind::Tool`:

Insert after `let str_joined = str_path.join("/");` (line 22) and before `let exports = if crate::stdlib::std_module_exists(...)`:

```rust
if let UseKind::Tool { command, alias } = &use_stmt.kind {
    return Err(LxError::runtime(
        format!("tool modules not yet implemented (use tool \"{}\" as {})", command, alias),
        span,
    ));
}
```

This ensures that `use tool "..." as Name` is parsed but immediately returns a clear error at runtime, rather than falling through to path resolution (which would fail confusingly since `path` is empty).

### 4. `crates/lx/src/formatter/emit_stmt.rs`

**Modify `emit_use()`** (lines 254-279). Add a match arm for `UseKind::Tool` in the match on `&u.kind`:

Current code (lines 262-278):
```rust
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
}
```

For `UseKind::Tool`, the entire `emit_use` method needs special handling because the path is empty and the syntax is different. Restructure `emit_use` to handle `Tool` before writing the path:

```rust
fn emit_use(&mut self, u: &UseStmt) {
  if let UseKind::Tool { command, alias } = &u.kind {
    self.write("use tool \"");
    self.write(command.as_str());
    self.write("\" as ");
    self.write(alias.as_str());
    return;
  }
  self.write("use ");
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
    UseKind::Tool { .. } => unreachable!(),
  }
}
```

## Step-by-Step Instructions

1. Add `Tool { command: Sym, alias: Sym }` variant to `UseKind` in `crates/lx/src/ast/types.rs`.

2. Rewrite `use_parser()` in `crates/lx/src/parser/stmt.rs` to add the `use tool "..." as Name` branch before the existing path-based branch, using `choice` or `.or()`.

3. Add the stub error handler for `UseKind::Tool` at the top of `eval_use()` in `crates/lx/src/interpreter/modules.rs`.

4. Rewrite `emit_use()` in `crates/lx/src/formatter/emit_stmt.rs` to handle `UseKind::Tool` with early return, then fall through to the existing logic for other variants.

5. Grep for any exhaustive `match` on `UseKind` elsewhere in the codebase and add `UseKind::Tool { .. }` arms. Known locations:
   - `crates/lx/src/interpreter/modules.rs` `eval_use()` lines 48-64 (the `match &use_stmt.kind` block) — add `UseKind::Tool { .. } => unreachable!()` since the early return above prevents reaching this match.
   - Search for other matches with: `grep -rn "UseKind" crates/lx/src/`

## Deliverable

After this work item:
- `use tool "agent-browser" as Browser` parses successfully and produces `Stmt::Use(UseStmt { path: [], kind: UseKind::Tool { command: "agent-browser", alias: Browser } })`
- The formatter round-trips it as `use tool "agent-browser" as Browser`
- Attempting to execute the statement at runtime produces a clear error: `"tool modules not yet implemented (use tool \"agent-browser\" as Browser)"`
- All existing `use` forms (`use std/foo`, `use ./local`, `use mod : alias`, `use mod { A; B }`) continue to work unchanged
- No new runtime behavior — this is parser/AST/formatter only
