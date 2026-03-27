# Goal

Fix the 11 actionable bugs from `crates/lx-cli/tests/bug_probes.rs`. When each test passes, remove `#[ignore]` and rename `bug_` to `fixed_`.

3 of the 14 probes are not code bugs — they're missing separators (bugs 4, 5) or an undesigned feature (bug 1). Those get test updates only.

# Why

These are the last known parser/runtime correctness issues. Every one blocks real lx programs from expressing natural patterns: screaming-case constants, multi-line ternaries, filter sections, paren blocks, assert messages. Fixing them removes workarounds from every future program.

# What Changes

## Unit 1 — Lexer token classification

**bug_screaming_case_constants** — `raw_token.rs:110`: TypeName regex `[A-Z][a-zA-Z0-9]*` splits `TARGET_GRADE` into `TypeName("TARGET")` + `Ident("_GRADE")` because `_` isn't in the character class.

Fix: Add a new logos variant with higher priority that matches SCREAMING_CASE (uppercase + digits + underscores, requiring at least one underscore):

```
#[regex(r"[A-Z][A-Z0-9]*(_[A-Z0-9]+)+", priority = 15)]
ScreamingCase,
```

In `lexer/mod.rs` dispatch, emit as `TokenKind::Ident(intern(slice))`. No changes to `type_name_or_keyword`.

**bug_uppercase_keyword_field_names** — `expr_helpers.rs:68`: `record_fields` uses `ident_or_keyword()` which doesn't match `AgentKw`, `ToolKw`, etc. So `{Agent: 1}` fails.

Fix: Change `ident_or_keyword()` to `ident_or_keyword().or(type_name())` in the record field name position. `type_name()` (from `super::expr`) handles `TypeName(n)` and all type-level keywords via `keyword_as_type_name()`. Add `type_name` to the import: `use super::expr::{ident, ident_or_keyword, item_sep, skip_item_sep, skip_semis, type_name};`.

## Unit 2 — Builtin return values

**bug_find_returns_value** — `builtins/hof.rs:175`: `find` returns `LxVal::some(v.clone())`. Tests assert `found == 4` but get `Some(4)`.

**bug_first_returns_value** — `builtins/coll.rs:29`: `first` uses `maybe()` which wraps in `LxVal::some()`.

**bug_last_returns_value** — `builtins/coll.rs:33`: `last` uses same `maybe()`.

Fix: Return `v.clone()` directly when found, `LxVal::None` when empty/not-found. Remove the `maybe()` helper (used only by `bi_first` and `bi_last`). The `??` coalesce operator still works because its desugaring has a catchall arm (`other -> other`) that passes raw values through unchanged.

**Cascading changes required** — 6 files depend on the `Some(val)` wrapping:

The `^` propagate operator (`interpreter/mod.rs:171-178`) errors on non-Result/Maybe values: `"^ expects Result or Maybe, got {type}"`. Four files use `| first ^` and must have `^` removed:

| File | Line | Current | After |
|------|------|---------|-------|
| `std/guard.lx` | 29 | `head = window \| first ^` | `head = window \| first` |
| `pkg/store/trace.lx` | 36 | `prev_score = (drop (p.0 - 1) recent) \| first ^` | `prev_score = (drop (p.0 - 1) recent) \| first` |
| `programs/brain/introspect.lx` | 17 | `pid_val = pid_result \| split " " \| first ^ \| to_int ?? 1` | `pid_val = pid_result \| split " " \| first \| to_int ?? 1` |
| `programs/brain/store/context.lx` | 73 | `... \| first ^)` (3 occurrences) | `... \| first)` |

Two files use explicit `Some(val)` pattern matching on `find` results and must switch to direct value matching:

`std/workflow.lx:38-42` — change:
```
step = self.steps | find (s) s.id == step_id
step ? {
  Some s -> s.undo != None ? (s.undo (self.status.get step_id))
  None -> ()
}
```
to:
```
step = self.steps | find (s) s.id == step_id
step ? {
  None -> ()
  s -> s.undo != None ? (s.undo (self.status.get step_id))
}
```

`programs/brain/agents/dispatcher.lx:88-90` — change:
```
matched ? {
  Some (cap worker) -> worker
  None -> specialist ? {
```
to:
```
matched ? {
  None -> specialist ? {
  ...
  (cap worker) -> worker
```
(Move `None` arm first since the value is now a raw tuple, not `Some(tuple)`. The tuple destructuring pattern `(cap worker)` matches the raw value directly.)

## Unit 3 — Assert parsing

**bug_assert_greedy_callable** — `parser/expr.rs:176-178`: post-processing after `assert expr` checks `Expr::Grouped` on the func position. `assert done "msg"` produces `Apply{Ident(done), Str("msg")}` but the `Grouped` gate rejects it.

Fix: Remove the `Grouped` check at line 177. Any `Apply{func, Str(msg)}` after assert becomes condition + message:

```rust
if let Expr::Apply(app) = ar.expr(ex)
    && let Expr::Literal(Literal::Str(_)) = ar.expr(app.arg)
```

Note on implicit application precedence: `assert x > 5 "msg"` still requires parens around the condition — `5 "msg"` binds tighter (prec 31) than `>` (prec 17), producing `x > Apply{5, "msg"}`. So the standard usage remains `assert (x > 5) "msg"`. The fix only affects the simple case: `assert ident "msg"`.

## Unit 4 — Ternary and pipe operators

**bug_multiline_ternary** — `expr_pratt.rs:117`: ternary tail is `expr.then(just(Colon).ignore_then(expr).or_not())`. Newlines become `Semi` tokens (lexer/mod.rs:103-106 emits Semi when `paren_bracket_depth <= 0`), so `? "one"\n: "other"` becomes `? "one" ; : "other"`. The `:` is separated from the `?` branch by a semi.

Fix: Allow optional semis before the colon:

```rust
let ternary_tail = expr.clone().then(skip_semis().ignore_then(just(TokenKind::Colon)).ignore_then(expr.clone()).or_not());
```

`skip_semis` is already imported from `super::expr` at line 6. Chumsky's `.or_not()` backtracks if the inner parser (including `skip_semis`) fails, restoring consumed semis. Safe because `:` has no other meaning in post-`?` expression context.

**bug_pipe_plus_precedence** — `expr_pratt.rs:157`: pipe `|` has precedence 19, below `+` at 25. So `[1;2;3] | len + [4;5] | len` parses as `[1;2;3] | (len + [4;5]) | len`.

Fix: Change pipe precedence from 19 to 26 (above `+` at 25, below `*` at 27):

```rust
infix(left(26), just(TokenKind::Pipe), move |l: ExprId, _, r: ExprId, e| {
```

Audit result: All 116 existing `.lx` files that mix pipe and arithmetic already parenthesize the arithmetic or use pipes in non-conflicting positions. No existing code breaks. The tradeoff: `a + b | f` now means `a + (b | f)` — users must write `(a + b) | f`. This pattern does not appear in the codebase.

## Unit 5 — Paren parser disambiguation

**bug_unit_before_closure_param** — `expr_compound.rs:105-114,126`: `func_def` in the choice list comes before `unit`. `func_def` matches `()` with zero params, then `.or(expr.clone())` for the body (line 111) greedily consumes the next expression. So `() (x) { body }` becomes `Func{[], Func{[x], body}}` instead of `Unit, Func{[x], body}`.

Fix: Add a `zero_arg_func` parser before `unit` in the choice list. `zero_arg_func` handles `() [type_params] [-> ret_type] [& guard] { body }` — same as func_def but requires brace body (no `.or(expr)` fallback). This is critical: 20+ files use `() -> Type { body }` (e.g., `pkg/agent/quality.lx:17`, `programs/brain/identity.lx:13`, `programs/brain/main.lx:178`).

Do NOT change `func_def`'s `param.repeated()` — leave it as-is. `()` is already caught by `zero_arg_func` (if followed by `{`, `[`, `->`, or `&`) or `unit` (otherwise), so func_def never sees zero params.

One file needs migration: `programs/brain/report.lx:26` has `+divider = () [md.hr]` which is currently parsed as a zero-arg func with bare list body. Change to `+divider = () { [md.hr] }`.

Choice ordering becomes:
```
choice((field_section, index_section, binop_section, right_section, zero_arg_func, unit, func_def, left_section, tuple, grouped, paren_block))
```

**bug_parens_not_blocks** — No paren parser alternative handles statements inside parens. `( x = 10; x + 5 )` fails because `=` isn't valid in any current paren production.

Fix: Add `paren_block` at the END of the choice list (after `grouped`). `grouped` fails on `( x = 10; ... )` because `=` after `x` isn't a valid expression continuation, so `paren_block` catches it. Simple `(expr)` is still caught by `grouped` first. `(x; y)` tuples are caught by the `tuple` parser (before `grouped`) because two exprs are found with semicolon separator — the `=` in `(x = 10; ...)` prevents element 2 from parsing, so `at_least(2)` fails and tuple passes.

## Unit 6 — Section desugaring

**bug_sections_equality** — `expr_compound.rs:73-77`: section parser only handles `(.field)`. `(.status == "pass")` fails because `==` after the field name doesn't match `)`.

Fix: Extend the `field_section` parser to optionally accept a comparison operator and value. Parse `(.field op expr)` and emit `Section::FieldCompare { field, op, value }`. The `section_op()` function (already imported from `super::expr_pratt`) lists all operators including `Eq`, `NotEq`, `Lt`, `Gt`, `LtEq`, `GtEq`, `Plus`, `Minus`, etc.

Desugar follows the existing Section pattern in `folder/desugar.rs` — generate a lambda `(x) { x.field op value }`.

AstWalk derive macro handles the new variant automatically. `Section::Right` and `Section::Left` already have `ExprId` fields and the macro generates correct `recurse_children`/`children`/`walk_children` for them. `FieldCompare` follows the same pattern.

## Unit 7 — Test cleanup

**bug_shorthand_before_keyed** and **bug_spread_shorthand**: Record fields require `;` or `,` separators. Removing the requirement creates ambiguity between records and blocks — `{f x}` could be `Record{f, x}` (two shorthand fields) or `Block(Apply{f, x})` (function call). The current parser uses `record_inner.or(block_inner)` in `expr_helpers.rs:52`, and separator presence is what disambiguates them. Update the tests to use separators and rename to `fixed_`.

**bug_named_arg_ternary_colon**: Named arguments don't exist in the grammar. This test describes a future feature, not a current bug. Remove the test.

# Files Affected

| File | Changes |
|------|---------|
| `crates/lx/src/lexer/raw_token.rs` | Add `ScreamingCase` variant with `priority = 15` |
| `crates/lx/src/lexer/mod.rs` | Add dispatch arm for `ScreamingCase` → `Ident` (between lines 157-158) |
| `crates/lx/src/parser/expr_helpers.rs` | Add `type_name` import; use `ident_or_keyword().or(type_name())` in record field name |
| `crates/lx/src/builtins/coll.rs` | Remove `maybe` helper; `first`/`last` return raw value or `None` |
| `crates/lx/src/builtins/hof.rs` | `find` line 175: `Ok(v.clone())` instead of `Ok(LxVal::some(v.clone()))` |
| `crates/lx/std/guard.lx` | Line 29: remove `^` after `first` |
| `pkg/store/trace.lx` | Line 36: remove `^` after `first` |
| `programs/brain/introspect.lx` | Line 17: remove `^` after `first` |
| `programs/brain/store/context.lx` | Line 73: remove `^` after `first` (3 occurrences) |
| `std/workflow.lx` | Lines 39-41: remove `Some` wrapper from match arm |
| `programs/brain/agents/dispatcher.lx` | Lines 88-90: remove `Some` wrapper, reorder arms |
| `crates/lx/src/parser/expr.rs` | Line 177: remove `Grouped` check in assert |
| `crates/lx/src/parser/expr_pratt.rs` | Line 117: `skip_semis()` before ternary colon; line 157: pipe precedence 19→26 |
| `crates/lx/src/parser/expr_compound.rs` | Add `zero_arg_func`, `paren_block`; reorder choice; add imports |
| `crates/lx/src/ast/expr_types.rs` | Add `Section::FieldCompare { field: Sym, op: BinOp, value: ExprId }` |
| `crates/lx/src/folder/desugar.rs` | Add `BinOp` import; add `FieldCompare` desugar arm |
| `crates/lx/src/formatter/emit_expr_helpers.rs` | Add `FieldCompare` formatting arm |
| `programs/brain/report.lx` | Line 26: `() [md.hr]` → `() { [md.hr] }` |
| `crates/lx-cli/tests/bug_probes.rs` | Remove `#[ignore]`, rename `bug_` → `fixed_` for all; update 3 tests |

# Task List

### Task 1: Add ScreamingCase lexer variant

**Subject:** SCREAMING_CASE identifiers lex as one Ident token

**Description:** In `crates/lx/src/lexer/raw_token.rs`, add a new variant before `TypeName` (before line 110):

```rust
#[regex(r"[A-Z][A-Z0-9]*(_[A-Z0-9]+)+", priority = 15)]
ScreamingCase,
```

In `crates/lx/src/lexer/mod.rs`, add a dispatch arm between the existing `RawToken::Ident` (line 156) and `RawToken::TypeName` (line 157) arms:

```rust
RawToken::ScreamingCase => self.emit(Token::new(TokenKind::Ident(crate::sym::intern(slice)), span)),
```

In `crates/lx-cli/tests/bug_probes.rs`, on `bug_screaming_case_constants`: remove `#[ignore]`, rename to `fixed_screaming_case_constants`.

**ActiveForm:** Adding ScreamingCase lexer variant

---

### Task 2: Allow uppercase keywords as record field names

**Subject:** `{Agent: 1}` parses as a record with keyword field name

**Description:** In `crates/lx/src/parser/expr_helpers.rs`:

Add `type_name` to the import on line 4:

```rust
use super::expr::{ident, ident_or_keyword, item_sep, skip_item_sep, skip_semis, type_name};
```

In `record_fields()` at line 68, change:

```rust
ident_or_keyword().then(just(TokenKind::Colon).ignore_then(expr).or_not())
```

to:

```rust
ident_or_keyword().or(type_name()).then(just(TokenKind::Colon).ignore_then(expr).or_not())
```

In `crates/lx-cli/tests/bug_probes.rs`, on `bug_uppercase_keyword_field_names`: remove `#[ignore]`, rename to `fixed_uppercase_keyword_field_names`.

**ActiveForm:** Allowing uppercase keyword field names

---

### Task 3: Make find/first/last return values directly

**Subject:** `find`, `first`, `last` return raw values instead of `Some(value)`

**Description:**

**Step 1 — Change builtins:**

In `crates/lx/src/builtins/coll.rs`, remove the `maybe` helper (lines 24-26) and replace `bi_first` (line 28-30) and `bi_last` (line 32-34):

```rust
fn bi_first(args: &[LxVal], sp: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(args[0].require_list("first", sp)?.first().cloned().unwrap_or(LxVal::None))
}

fn bi_last(args: &[LxVal], sp: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(args[0].require_list("last", sp)?.last().cloned().unwrap_or(LxVal::None))
}
```

In `crates/lx/src/builtins/hof.rs`, line 175, change `return Ok(LxVal::some(v.clone()));` to `return Ok(v.clone());`.

**Step 2 — Remove `^` from `| first ^` call sites (4 files):**

These use `^` to unwrap `Some(val)`. With raw values, `^` would error ("expects Result or Maybe"). Remove the `^`:

- `crates/lx/std/guard.lx:29` — `head = window | first ^` → `head = window | first`
- `pkg/store/trace.lx:36` — `prev_score = (drop (p.0 - 1) recent) | first ^` → `prev_score = (drop (p.0 - 1) recent) | first`
- `programs/brain/introspect.lx:17` — `pid_val = pid_result | split " " | first ^ | to_int ?? 1` → `pid_val = pid_result | split " " | first | to_int ?? 1`
- `programs/brain/store/context.lx:73` — remove `^` after all 3 occurrences of `| first ^` on that line

**Step 3 — Update explicit `Some(val)` pattern matches (2 files):**

In `crates/lx/std/workflow.lx`, lines 38-42, change:
```
step = self.steps | find (s) s.id == step_id
step ? {
  Some s -> s.undo != None ? (s.undo (self.status.get step_id))
  None -> ()
}
```
to:
```
step = self.steps | find (s) s.id == step_id
step ? {
  None -> ()
  s -> s.undo != None ? (s.undo (self.status.get step_id))
}
```

In `programs/brain/agents/dispatcher.lx`, lines 88-90, change:
```
matched ? {
  Some (cap worker) -> worker
  None -> specialist ? {
```
to:
```
matched ? {
  None -> specialist ? {
  ...  (keep the None arm body as-is)
  (cap worker) -> worker
```
(Reorder arms: `None` first, tuple destructure second. The raw tuple value matches `(cap worker)` directly without `Some` wrapping.)

**Step 4 — Update bug probes:**

In `crates/lx-cli/tests/bug_probes.rs`, on all three (`bug_find_returns_value`, `bug_first_returns_value`, `bug_last_returns_value`): remove `#[ignore]`, rename `bug_` to `fixed_`.

**ActiveForm:** Fixing find/first/last return values

---

### Task 4: Remove Grouped gate in assert parsing

**Subject:** `assert done "msg"` treats `done` as condition and `"msg"` as message

**Description:** In `crates/lx/src/parser/expr.rs`, lines 176-178, the assert post-processing has:

```rust
if let Expr::Apply(app) = ar.expr(ex)
    && let Expr::Grouped(_) = ar.expr(app.func)
    && let Expr::Literal(Literal::Str(_)) = ar.expr(app.arg)
```

Remove the middle line (`Grouped` check). Result:

```rust
if let Expr::Apply(app) = ar.expr(ex)
    && let Expr::Literal(Literal::Str(_)) = ar.expr(app.arg)
```

In `crates/lx-cli/tests/bug_probes.rs`, on `bug_assert_greedy_callable`: remove `#[ignore]`, rename to `fixed_assert_greedy_callable`.

**ActiveForm:** Removing Grouped gate in assert

---

### Task 5: Allow semis before ternary colon

**Subject:** Multi-line ternary `? val\n: val` parses correctly

**Description:** In `crates/lx/src/parser/expr_pratt.rs`, line 117, change:

```rust
let ternary_tail = expr.clone().then(just(TokenKind::Colon).ignore_then(expr.clone()).or_not());
```

to:

```rust
let ternary_tail = expr.clone().then(skip_semis().ignore_then(just(TokenKind::Colon)).ignore_then(expr.clone()).or_not());
```

`skip_semis` is already imported at line 6: `use super::expr::{semi_sep, skip_semis, type_name};`.

In `crates/lx-cli/tests/bug_probes.rs`, on `bug_multiline_ternary`: remove `#[ignore]`, rename to `fixed_multiline_ternary`.

**ActiveForm:** Allowing semis before ternary colon

---

### Task 6: Raise pipe precedence above plus

**Subject:** `x | len + y | len` binds as `(x | len) + (y | len)`

**Description:** In `crates/lx/src/parser/expr_pratt.rs`, line 157, change pipe precedence from 19 to 26:

```rust
infix(left(26), just(TokenKind::Pipe), move |l: ExprId, _, r: ExprId, e| {
```

All 116 existing `.lx` files that mix pipe and arithmetic already parenthesize or use non-conflicting positions. No existing code breaks.

In `crates/lx-cli/tests/bug_probes.rs`, on `bug_pipe_plus_precedence`: remove `#[ignore]`, rename to `fixed_pipe_plus_precedence`.

**ActiveForm:** Raising pipe precedence

---

### Task 7: Split zero-arg function from unit in paren parser

**Subject:** `() (x) { body }` parses as `Unit, Func{[x], body}` not `Func{[], Func{[x], body}}`

**Description:** In `crates/lx/src/parser/expr_compound.rs`:

Add imports — change line 7 to:

```rust
use crate::ast::{BinOp, Expr, ExprBlock, ExprFunc, ExprTuple, ExprWith, Literal, Section, WithKind};
```

Add `let a_zf = arena.clone();` alongside the existing arena clones (after line 68).

Add a `zero_arg_func` parser before the existing `unit` parser. It handles `() [type_params] [-> ret_type] [& guard] { body }` with brace body only (no bare expression fallback):

```rust
let zero_arg_func = just(TokenKind::LParen)
    .then(just(TokenKind::RParen))
    .ignore_then(super::type_ann::generic_params())
    .then(just(TokenKind::Arrow).ignore_then(super::type_ann::type_parser(arena.clone())).or_not())
    .then(just(TokenKind::Amp).ignore_then(expr.clone().delimited_by(just(TokenKind::LParen), just(TokenKind::RParen))).or_not())
    .then(super::expr_helpers::func_body_parser(expr.clone(), a_body.clone()))
    .map_with(move |(((type_params, ret_type), guard), body), e| {
        a_zf.borrow_mut().alloc_expr(Expr::Func(ExprFunc {
            params: vec![], type_params, ret_type, guard, body
        }), ss(e.span()))
    });
```

The key difference from `func_def`: body uses `func_body_parser` only (requires `{`), not `func_body_parser.or(expr)`. This means `()` without a following `{` falls through to `unit`. If `()` is followed by `->`, `[`, or `&` then `{`, `zero_arg_func` matches. If `()` is followed by nothing valid, `zero_arg_func` fails (chumsky backtracks) and `unit` catches it.

Do NOT change `func_def`'s `param.repeated()` to `at_least(1)` — leave it as-is. `()` is always caught by `zero_arg_func` or `unit` first due to choice ordering, so `func_def` never sees zero params.

Reorder the choice at line 126 to place `zero_arg_func` before `unit`:

```rust
choice((field_section, index_section, binop_section, right_section, zero_arg_func, unit, func_def, left_section, tuple, grouped))
```

(The `paren_block` from Task 8 will be appended to the end of this list.)

Migrate one file: `programs/brain/report.lx:26` — change `+divider = () [md.hr]` to `+divider = () { [md.hr] }`. This is currently parsed as a zero-arg func with bare list body via `func_def`'s `.or(expr)` fallback, which is removed for zero-param functions.

In `crates/lx-cli/tests/bug_probes.rs`, on `bug_unit_before_closure_param`: remove `#[ignore]`, rename to `fixed_unit_before_closure_param`.

**ActiveForm:** Splitting zero-arg function from unit

---

### Task 8: Add paren-block parser

**Subject:** `( x = 10; x + 5 )` evaluates as a block expression

**Description:** In `crates/lx/src/parser/expr_compound.rs`:

Add `let a_pb = arena.clone();` alongside the existing arena clones.

Add a `paren_block` parser:

```rust
let paren_block = just(TokenKind::LParen)
    .ignore_then(super::expr::stmts_block(expr.clone(), a_pb.clone()))
    .then_ignore(just(TokenKind::RParen))
    .map_with(move |stmts, e| {
        a_pb.borrow_mut().alloc_expr(Expr::Block(ExprBlock { stmts }), ss(e.span()))
    });
```

`ExprBlock` is already added to imports in Task 7. `stmts_block` is accessed via `super::expr::stmts_block` (same pattern as `with_parser` at line 26).

Append `paren_block` at the END of the choice list (after `grouped`):

```rust
choice((field_section, index_section, binop_section, right_section, zero_arg_func, unit, func_def, left_section, tuple, grouped, paren_block))
```

Why this ordering is safe: `grouped` parses `(expr)` where `expr` is the full expression parser. For `( x = 10; x + 5 )`, the expr parser starts with `x` (Ident), then sees `=` (Assign) which is not a valid expression operator or continuation — so `expr` returns just `x`. Then `grouped` expects `)` but sees `=` → fails. `paren_block` then tries `stmts_block` which parses `x = 10` as a binding statement, `;` as separator, `x + 5` as expression statement → succeeds.

In `crates/lx-cli/tests/bug_probes.rs`, on `bug_parens_not_blocks`: remove `#[ignore]`, rename to `fixed_parens_not_blocks`.

**ActiveForm:** Adding paren-block parser

---

### Task 9: Add FieldCompare section variant

**Subject:** `(.status == "pass")` desugars to `(x) { x.status == "pass" }`

**Description:**

**AST** — In `crates/lx/src/ast/expr_types.rs`, add to `Section` enum (after `BinOp(BinOp)` on line 57):

```rust
FieldCompare { field: Sym, op: BinOp, value: ExprId },
```

The `AstWalk` derive macro handles `ExprId` fields automatically — `Section::Right` and `Section::Left` already have `ExprId` fields and generate correct walk/recurse/children implementations. No manual walk impl needed.

**Parser** — In `crates/lx/src/parser/expr_compound.rs`, replace the `field_section` parser (lines 73-77) with:

```rust
let field_section = just(TokenKind::LParen)
    .ignore_then(just(TokenKind::Dot))
    .ignore_then(ident())
    .then(section_op().then(expr.clone()).or_not())
    .then_ignore(just(TokenKind::RParen))
    .map_with(move |(name, cmp), e| {
        let section = match cmp {
            Some((op_tok, value)) => Section::FieldCompare { field: name, op: tok_to_op(&op_tok), value },
            None => Section::Field(name),
        };
        a2.borrow_mut().alloc_expr(Expr::Section(section), ss(e.span()))
    });
```

`section_op` and `tok_to_op` are already imported at line 5: `use super::expr_pratt::{section_op, tok_to_op};`. `BinOp` and `Section` are added to imports in Task 7.

**Desugar** — In `crates/lx/src/folder/desugar.rs`, add `BinOp` to the import on line 7:

```rust
use crate::ast::{
  AstArena, BinOp, BindTarget, Binding, ...
```

Add an arm in `desugar_section` (after `Section::BinOp` arm, before the closing `}`):

```rust
Section::FieldCompare { field, op, value } => {
    let p = gensym("x");
    let pi = arena.alloc_expr(Expr::Ident(p), span);
    let access = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: pi, field: FieldKind::Named(field) }), span);
    let body = arena.alloc_expr(Expr::Binary(ExprBinary { op, left: access, right: value }), span);
    make_lambda_expr(p, body)
},
```

**Formatter** — In `crates/lx/src/formatter/emit_expr_helpers.rs`, add an arm in `emit_section` (after `Section::Index` arm, before the closing `}`):

```rust
Section::FieldCompare { field, op, value } => {
    self.write(".");
    self.write(field.as_str());
    self.space();
    self.write(&op.to_string());
    self.space();
    self.emit_expr(*value);
},
```

In `crates/lx-cli/tests/bug_probes.rs`, on `bug_sections_equality`: remove `#[ignore]`, rename to `fixed_sections_equality`.

**ActiveForm:** Adding FieldCompare section variant

---

### Task 10: Update non-bug test probes

**Subject:** Close separator and named-arg test probes

**Description:** In `crates/lx-cli/tests/bug_probes.rs`:

**bug_shorthand_before_keyed:** Remove `#[ignore]`, rename to `fixed_shorthand_before_keyed`. Update the test code to use semicolons between fields:

```rust
let (ok, stdout, _) = run_lx("steps = [1; 2; 3]\ntask = \"do it\"\nr = {steps; task; step_count: steps | len}\nemit r.step_count");
```

**bug_spread_shorthand:** Remove `#[ignore]`, rename to `fixed_spread_shorthand`. Update to use semicolons:

```rust
let (ok, stdout, _) = run_lx("entry = {name: \"a\"; value: 1}\nscore = 100\nr = {..entry; score}\nemit r.score");
```

**bug_named_arg_ternary_colon:** Remove the test entirely. Named arguments are an undesigned feature — the test code is semantically incoherent (`id = (x) x` takes one arg, but the expression passes multiple arguments including a `key: "v"` named arg that has no grammar support).

**ActiveForm:** Updating non-bug test probes

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/BUG_PROBE_FIXES.md" })
```
