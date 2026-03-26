# Goal

Make comma (`,`) a proper collection separator token instead of mapping it to semicolon. Commas separate fields/elements in collections (records, lists, tuples, maps, patterns, type annotations, use imports, generic params). Commas do NOT separate statements.

# Why

The lexer maps `Comma` → `Semi` at `crates/lx/src/lexer/mod.rs:143`. This means commas work as statement separators (`a = 1, b = 2` is valid), which is wrong. Commas should only work in collections. LLM agents writing lx naturally use commas in records (`{a: Str, b: Int}`).

# What Changes

## Lexer

**`crates/lx/src/lexer/token.rs`** — Add `Comma` variant after `Semi` (line 101):
```rust
Semi,
Comma,
Eof,
```

**`crates/lx/src/lexer/mod.rs:143`** — Split Comma from Semi:
```rust
// Before:
RawToken::Semi | RawToken::Comma => self.emit(Token::new(TokenKind::Semi, span)),
// After:
RawToken::Semi => self.emit(Token::new(TokenKind::Semi, span)),
RawToken::Comma => self.emit(Token::new(TokenKind::Comma, span)),
```

Note: the `is_semi` dedup at lines 37-40 only collapses consecutive `Semi` tokens. `Comma` won't be affected since it's a separate token kind. No change needed to the dedup logic.

## Parser — new helpers

**`crates/lx/src/parser/expr.rs`** — Add two new helpers after `semi_sep()` (after line 82):
```rust
pub(super) fn item_sep<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  just(TokenKind::Semi).or(just(TokenKind::Comma)).repeated().at_least(1).ignored()
}

pub(super) fn skip_item_sep<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  just(TokenKind::Semi).or(just(TokenKind::Comma)).repeated().ignored()
}
```

## Parser — collection locations (change to accept Comma)

Each location below currently uses `Semi`-only separation. Change to accept `Semi` or `Comma`.

**`crates/lx/src/parser/expr_helpers.rs:76`** — record fields:
```rust
// Before:
skip_semis().ignore_then(field.separated_by(skip_semis()).at_least(1).allow_trailing().collect::<Vec<_>>()).then_ignore(skip_semis())
// After:
skip_item_sep().ignore_then(field.separated_by(item_sep()).at_least(1).allow_trailing().collect::<Vec<_>>()).then_ignore(skip_item_sep())
```
Also update the import at line 4 to include `item_sep, skip_item_sep`.

**`crates/lx/src/parser/expr_helpers.rs:23-25`** — list elements:
```rust
// Before:
.ignore_then(super::expr::skip_semis())
.ignore_then(elem.separated_by(super::expr::semi_sep()).allow_trailing().collect::<Vec<_>>())
.then_ignore(super::expr::skip_semis())
// After:
.ignore_then(super::expr::skip_item_sep())
.ignore_then(elem.separated_by(super::expr::item_sep()).allow_trailing().collect::<Vec<_>>())
.then_ignore(super::expr::skip_item_sep())
```

**`crates/lx/src/parser/expr_helpers.rs:109`** — map entries (`%{k: v}`):
```rust
// Before:
.separated_by(just(TokenKind::Semi).or_not())
// After:
.separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)).or_not())
```

**`crates/lx/src/parser/expr_compound.rs:117`** — tuple elements:
```rust
// Before:
.separated_by(just(TokenKind::Semi).or_not())
// After:
.separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)).or_not())
```

**`crates/lx/src/parser/expr_compound.rs:45`** — `with` resources:
```rust
// Before:
.separated_by(just(TokenKind::Semi))
// After:
.separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)))
```

**`crates/lx/src/parser/pattern.rs:43`** — tuple patterns:
```rust
// Before:
.separated_by(just(TokenKind::Semi).or_not())
// After:
.separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)).or_not())
```

**`crates/lx/src/parser/pattern.rs:79-82`** — record patterns:
```rust
// Before: 4 uses of skip_semis()
// After: 4 uses of skip_item_sep()
```
Add import of `skip_item_sep` (currently imports `skip_semis` from `super::expr`).

**`crates/lx/src/parser/pattern.rs:97`** — list patterns:
```rust
// Before:
.separated_by(just(TokenKind::Semi).or_not())
// After:
.separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)).or_not())
```

**`crates/lx/src/parser/type_ann.rs:83-86`** — record type annotations:
```rust
// Before:
let record_ty = just(TokenKind::Semi)
  .repeated()
  .ignore_then(record_field.separated_by(just(TokenKind::Semi).repeated().at_least(1)).allow_trailing()...)
  .then_ignore(just(TokenKind::Semi).repeated())
// After:
let record_ty = just(TokenKind::Semi).or(just(TokenKind::Comma))
  .repeated()
  .ignore_then(record_field.separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)).repeated().at_least(1)).allow_trailing()...)
  .then_ignore(just(TokenKind::Semi).or(just(TokenKind::Comma)).repeated())
```

**`crates/lx/src/parser/type_ann.rs:121`** — generic params `[T; U]`:
```rust
// Before:
.separated_by(just(TokenKind::Semi))
// After:
.separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)))
```

**`crates/lx/src/parser/stmt.rs:106`** — `use` selective imports `{parse encode}`:
```rust
// Before:
.separated_by(just(TokenKind::Semi).or_not())
// After:
.separated_by(just(TokenKind::Semi).or(just(TokenKind::Comma)).or_not())
```

**`crates/lx/src/parser/stmt.rs:171`** — error recovery token filter:
```rust
// Before:
.filter(|k: &TokenKind| !matches!(k, TokenKind::Pipe | TokenKind::Semi | TokenKind::Eof | TokenKind::RBrace))
// After:
.filter(|k: &TokenKind| !matches!(k, TokenKind::Pipe | TokenKind::Semi | TokenKind::Comma | TokenKind::Eof | TokenKind::RBrace))
```

## Locations that stay Semi-only (NO change)

- `stmt.rs:19` — `skip_to_semi` error recovery — stays Semi-only (commas inside collections shouldn't stop error recovery)
- `stmt.rs:24` — statement separation in program body — Semi-only (commas don't separate statements)
- `expr.rs:229` — `stmts_block` block body — Semi-only
- `expr_pratt.rs:103-114` — match arm separation — Semi-only
- `stmt_class.rs:27-29,54` — class/trait body members — Semi-only (newlines between fields/methods, not commas)
- `stmt.rs:180-181,245` — trait/type definitions — Semi-only
- `lexer/mod.rs:37` — Semi dedup — unchanged (only deduplicates Semi, not Comma)
- `lexer/mod.rs:105` — newline → Semi emission — unchanged

## No changes needed

- **Formatter** — works on AST, not tokens. No Semi references.
- **Checker** — works on AST. No TokenKind::Semi references.
- **Linter** — works on AST. No TokenKind::Semi references.
- **Interpreter** — works on AST. No TokenKind::Semi references.
- **Func params** (`expr_compound.rs:106`) — uses `param.repeated()` with no separator. Space-separated by design. No change.

# Task List

### Task 1: Add Comma token to lexer
Add `Comma` to `TokenKind` in `token.rs`. Split the `Semi | Comma` arm in `lexer/mod.rs:143` into two separate arms. Add `item_sep` and `skip_item_sep` helpers to `expr.rs`.

### Task 2: Update collection parsers
Change all 12 collection separator locations listed above to accept `Comma`. Update imports where needed (`expr_helpers.rs`, `pattern.rs`).

### Task 3: Update error recovery filter
Add `TokenKind::Comma` to the filter in `stmt.rs:171`.

### Task 4: Run diagnostics and tests
Run `just rust-diagnose` — must be 0 errors, 0 warnings. Run `just test` — all 27 tests must pass. If any test references comma behavior, verify it still works.

### Task 5: Write comma tests
Create `tests/comma_separator.lx`:
- `{a: 1, b: 2}` — comma-separated record fields
- `{a: Str, b: Int, c: Bool}` — three Type-valued fields, single line
- `[1, 2, 3]` — comma-separated list
- `(1, 2, 3)` — comma-separated tuple
- `%{"a": 1, "b": 2}` — comma-separated map
- Verify values with assert

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/COMMA_SEPARATOR.md" })
```
