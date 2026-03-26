# Goal

Remove `looks_like_record` lookahead from the parser. Make `{...}` default to record when content has no bindings or control flow. `{kv; kv2}` becomes shorthand for `{kv: kv; kv2: kv2}`. Blocks are the fallback when record parsing fails (content has `=`, `:=`, `<-`, function application, etc.).

# Root Cause

`looks_like_record()` in `crates/lx/src/parser/expr_helpers.rs` (lines 62-67) only triggers record parsing when the FIRST field has an explicit colon or `..`. This means `{kv; kv2}` (all shorthand) and `{kv; other: val}` (shorthand first) are parsed as blocks, not records. The shorthand field support in `record_fields` (line 80-81) exists but is gated behind this broken disambiguator.

# Design Decision

Flip the priority: try record first, fall back to block. Content self-disambiguates:

- `ident: expr` is only valid as a record field, never a block statement
- `ident = expr` is only valid as a binding, never a record field
- Bare `ident` with no side effects is useless in a block (discarded), useful in a record (shorthand field)
- `{:}` stays as empty record (separate parser branch, unchanged)
- `{}` stays as empty block returning Unit (record_fields requires at_least 1 field, fails, falls to block)
- `{foo}` is a record with one shorthand field — if you want just the value, write `foo` without braces

Chumsky 0.12 `choice`/`.or()` backtracks naturally when an alternative fails, even after consuming tokens. This is verified by the existing code where `choice((empty_record, record, block))` works despite all three branches consuming `{`.

# Verified Facts

- `record_fields` (lines 69-89) already supports shorthand via `.or_not()` on the colon — no changes needed to field parsing logic
- `block_or_record_parser` is called from exactly 2 places: `expr.rs` line 116 (atom in expr_parser) and `expr_compound.rs` line 111 (function body, with `.or(expr.clone())` fallback so braceless bodies still work)
- `loop`, `par`, `with` all use `stmts_block` directly — unaffected by this change
- `empty_record` parser (`{:}`) is a separate `choice` branch tried first — unaffected
- No changes needed to AST (`RecordField`, `ExprBlock`), interpreter, checker, visitor, linter, or desugarer
- All needed imports (`Expr`, `ExprBlock`, `RecordField`) already exist in `expr_helpers.rs` line 6
- Double-parse cost for blocks is negligible: record_fields consumes 1 ident, hits non-separator (`=`, `+`, `(`), fails in ~3 token peeks

# Span Correctness

The current code puts `map_with` (which captures span for AST node allocation) on each branch that starts with `just(LBrace)`, so spans cover `{` through `}`. In the new code, `{` is consumed once by a shared `just(LBrace)`, and the record/block alternatives are inside `.or()` AFTER that. If `map_with` were on the inner alternatives, spans would miss `{`.

Fix: the inner alternatives produce `Expr` values (not `ExprId`), and a single outer `map_with` after `just(LBrace).ignore_then(...)` allocates the `ExprId` with the correct full span. This requires 4 arena clones (1 for empty_record, 1 for record_fields internal ident allocs, 1 for stmts_block internal allocs, 1 for the outer alloc).

# Files Affected

- EDIT: `crates/lx/src/parser/expr_helpers.rs` — core parser change (only file)

# Task List

### Task 1: Delete looks_like_record

**Subject:** Remove the looks_like_record function

**Description:** In `crates/lx/src/parser/expr_helpers.rs`, delete the `looks_like_record` function (lines 62-67):

```rust
fn looks_like_record<'a, I>() -> impl Parser<'a, I, (), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  choice((ident_or_keyword().then_ignore(just(TokenKind::Colon)).ignored(), just(TokenKind::DotDot).ignored()))
}
```

This function is only referenced within `block_or_record_parser` at line 49. It will be replaced in the next task.

**ActiveForm:** Deleting looks_like_record function

---

### Task 2: Add at_least(1) to record_fields

**Subject:** Require at least one field for record parsing

**Description:** In `crates/lx/src/parser/expr_helpers.rs`, in the `record_fields` function (lines 69-89), change the `separated_by` chain to require at least 1 field. Find:

```rust
  skip_semis().ignore_then(field.separated_by(skip_semis()).allow_trailing().collect::<Vec<_>>()).then_ignore(skip_semis())
```

Change to:

```rust
  skip_semis().ignore_then(field.separated_by(skip_semis()).at_least(1).allow_trailing().collect::<Vec<_>>()).then_ignore(skip_semis())
```

This ensures `{}` (empty braces) fails record parsing and falls through to block (returns Unit). Empty records use `{:}` via the separate `empty_record` parser.

**ActiveForm:** Adding at_least(1) to record_fields separated_by

---

### Task 3: Restructure block_or_record_parser

**Subject:** Try record first, fall back to block, remove lookahead

**Description:** In `crates/lx/src/parser/expr_helpers.rs`, rewrite `block_or_record_parser` (lines 30-60). The current implementation:

```rust
pub(super) fn block_or_record_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let a1 = arena.clone();
  let a2 = arena.clone();
  let a3 = arena;

  let empty_record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(just(TokenKind::Colon))
    .then(just(TokenKind::RBrace))
    .map_with(move |_, e| a1.borrow_mut().alloc_expr(Expr::Record(vec![]), ss(e.span())));

  let record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(looks_like_record().rewind())
    .ignore_then(record_fields(expr.clone(), a2.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(move |fields, e| a2.borrow_mut().alloc_expr(Expr::Record(fields), ss(e.span())));

  let block = just(TokenKind::LBrace)
    .ignore_then(super::expr::stmts_block(expr, a3.clone()))
    .then_ignore(just(TokenKind::RBrace))
    .map_with(move |stmts, e| a3.borrow_mut().alloc_expr(Expr::Block(ExprBlock { stmts }), ss(e.span())));

  choice((empty_record, record, block))
}
```

Replace with:

```rust
pub(super) fn block_or_record_parser<'a, I>(
  expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
  arena: ArenaRef,
) -> impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone
where
  I: ValueInput<'a, Token = TokenKind, Span = Span>,
{
  let a1 = arena.clone();
  let a2 = arena.clone();
  let a3 = arena.clone();
  let a4 = arena;

  let empty_record = just(TokenKind::LBrace)
    .then(skip_semis())
    .then(just(TokenKind::Colon))
    .then(just(TokenKind::RBrace))
    .map_with(move |_, e| a1.borrow_mut().alloc_expr(Expr::Record(vec![]), ss(e.span())));

  let record_inner = record_fields(expr.clone(), a2)
    .then_ignore(just(TokenKind::RBrace))
    .map(|fields| Expr::Record(fields));

  let block_inner = super::expr::stmts_block(expr, a3)
    .then_ignore(just(TokenKind::RBrace))
    .map(|stmts| Expr::Block(ExprBlock { stmts }));

  let brace_expr = just(TokenKind::LBrace)
    .ignore_then(record_inner.or(block_inner))
    .map_with(move |node, e| a4.borrow_mut().alloc_expr(node, ss(e.span())));

  choice((empty_record, brace_expr))
}
```

Key details:
- `empty_record` is tried first (unchanged — handles `{:}`)
- `record_inner` and `block_inner` produce `Expr` values (not `ExprId`) via `.map()` — no arena alloc yet
- `brace_expr` consumes `{` once, tries record then block via `.or()`, and a single outer `map_with` allocates the `ExprId` with the full `{` through `}` span
- 4 arena clones: `a1` for empty_record alloc, `a2` for record_fields internal shorthand ident allocs, `a3` for stmts_block internal allocs, `a4` for the outer brace_expr alloc
- chumsky `.or()` backtracks if record_inner fails — tries block_inner from the same position (after `{`)

**ActiveForm:** Restructuring block_or_record_parser to record-first

---

### Task 4: Write parser tests

**Subject:** Test record shorthand parsing and block fallback

**Description:** Create test file `tests/record_shorthand.lx`:

```lx
-- record shorthand: {ident} and {ident; ident} parse as records
-- blocks fall back when content has bindings or expressions

-- all-shorthand records
a = 1
b = 2
c = 3
r = {a; b; c}
assert (r.a == 1)
assert (r.b == 2)
assert (r.c == 3)

-- single shorthand field
name = "alice"
r2 = {name}
assert (r2.name == "alice")

-- mixed shorthand and explicit
x = 10
r3 = {x; y: 20; z: 30}
assert (r3.x == 10)
assert (r3.y == 20)
assert (r3.z == 30)

-- shorthand first, explicit after
val = "hello"
r4 = {val; len: 5}
assert (r4.val == "hello")
assert (r4.len == 5)

-- spread still works with shorthand
base = {a: 1; b: 2}
extra = 3
r5 = {..base; extra}
assert (r5.a == 1)
assert (r5.b == 2)
assert (r5.extra == 3)

-- empty braces is block returning unit
r6 = {}
assert (r6 == ())

-- empty record still uses {:}
r7 = {:}
assert (r7 == {:})

-- blocks with bindings still work
r8 = {
  x = 10
  y = 20
  x + y
}
assert (r8 == 30)

-- block with function application
items = [1; 2; 3]
r9 = {
  items | len
}
assert (r9 == 3)

-- constructor call with shorthand fields
Trait +Pt = {x: Int; y: Int}
px = 5
py = 10
p1 = Pt {x: px; y: py}
assert (p1.x == 5)
assert (p1.y == 10)

-- constructor call with variable names matching field names
x = 100
y = 200
p2 = Pt {x; y}
assert (p2.x == 100)
assert (p2.y == 200)

-- nested records with shorthand
inner = "nested"
outer = {inner; tag: "wrapper"}
assert (outer.inner == "nested")
assert (outer.tag == "wrapper")
```

Run with `just test` to verify all assertions pass.

**ActiveForm:** Writing record shorthand parser tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/RECORD_SHORTHAND_PARSER.md" })
```
