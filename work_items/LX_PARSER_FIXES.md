# Goal

Fix two parser bugs: lambda body greedily consuming pipes, and assert message parsing.

Note: "trait body name conflict" was investigated and is NOT a separate bug. Record literals inside trait bodies work when fields are on separate lines (lexer brace depth fix) or separated by semicolons. Multi-field records on ONE line always need semicolons — this is universal lx behavior, not trait-body-specific. Verified: `Trait T = { m = () { {name: "hello"} } }` parses fine. `{name: "hello"  age: 42}` fails everywhere (not just traits) because space-separated fields on one line need semicolons.

# Root Causes (verified by reading source and testing)

**Lambda body is greedy (pipe-after-block):**

`crates/lx/src/parser/expr_compound.rs` line 110: `func_def` parser's body is `expr.clone()` — the FULL expression parser. `(x) { x > 2 } | len` — func_def matches `(x)` as params, then `{ x > 2 } | len` as body. Verified: `filter ((x) { x > 2 }) | len` works (explicit parens force the lambda to be one atom), `filter (x) { x > 2 } | len` doesn't.

Fix: use `block_or_record_parser` (already exists in `expr_helpers.rs` as `pub(super)`) when `{` follows params. It parses `{ }` blocks and records, stopping at `}`. Falls back to `expr.clone()` for bare bodies. `block_or_record_parser` handles: empty records `{:}`, records `{name: val}` (via `looks_like_record` lookahead), and blocks `{stmts; expr}`.

**Assert message consumed by greedy expr:**

`crates/lx/src/parser/expr.rs` lines 138-143: `just(Assert).ignore_then(expr.clone()).then(expr.clone().or_not())`. First `expr.clone()` consumes `(cond) "msg"` as `Apply(cond, msg)`.

Fix: post-parse fixup — after parsing, if result is `Apply(cond, Literal::Str(...))`, split into assert condition + message. Must scope the immutable arena borrow in a block, drop it, then do the mutable alloc.

# Files Affected

- `crates/lx/src/parser/expr_compound.rs` — Fix func_def body (line 110)
- `crates/lx/src/parser/expr.rs` — Fix assert (lines 138-143)

# Task List

### Task 1: Fix lambda body — block body stops at `}`

**Subject:** Lambda with `{ }` body should not consume pipe after closing brace

**Description:** Edit `crates/lx/src/parser/expr_compound.rs`. At line 110, the func_def body is `.then(expr.clone())`.

Change to:
```rust
.then(
    super::expr_helpers::block_or_record_parser(expr.clone(), a_body.clone())
        .or(expr.clone())
)
```

Add `let a_body = arena.clone();` alongside the existing arena clones at lines 61-68.

`block_or_record_parser` is already `pub(super)` in `expr_helpers.rs`. chumsky `.or()` tries left first — if `{` is next, `block_or_record_parser` matches (stops at `}`). If not, falls through to `expr.clone()` for bare bodies like `(x) x + 1`.

Write test `tests/parser_lambda_pipe.lx`:
```lx
items = [1; 2; 3; 4; 5]

result = items | filter (x) { x > 2 } | len
assert (result == 3)

mapped = items | map (x) { x * 2 } | filter (x) { x > 4 } | len
assert (mapped == 3)

inner_pipe = items | map (x) { x | to_str }
assert (inner_pipe | len == 5)

bare_body = items | map (x) x * 2
assert (bare_body | len == 5)

record_body = items | map (x) {value: x}
assert (record_body | len == 5)
```

**ActiveForm:** Fixing lambda block body parsing

---

### Task 2: Fix assert message parsing

**Subject:** Assert condition should not consume the message string

**Description:** Edit `crates/lx/src/parser/expr.rs` lines 138-143. Replace the assert parser with:

```rust
let assert_expr = {
    let al = arena.clone();
    just(TokenKind::Assert)
        .ignore_then(expr.clone())
        .map_with(move |ex, e| {
            let (cond, msg) = {
                let ar = al.borrow();
                if let Expr::Apply(app) = ar.expr(ex) {
                    if let Expr::Literal(Literal::Str(_)) = ar.expr(app.arg) {
                        (app.func, Some(app.arg))
                    } else {
                        (ex, None)
                    }
                } else {
                    (ex, None)
                }
            };
            al.borrow_mut().alloc_expr(Expr::Assert(ExprAssert { expr: cond, msg }), ss(e.span()))
        })
};
```

The immutable borrow `al.borrow()` is scoped in a block that returns `(cond, msg)`. After the block ends, the borrow is dropped. Then `al.borrow_mut()` allocates. This avoids the simultaneous immutable+mutable borrow panic.

Remove the `.then(expr.clone().or_not())` from the current parser — it's replaced by the post-parse splitting.

Add `Expr::Apply` and `ExprApply` to the imports at the top of `expr.rs` if not already present. Check the existing `use crate::ast::{ ... }` statement.

Write test `tests/parser_assert_message.lx`:
```lx
assert (1 == 1) "one equals one"
assert (2 > 1) "two greater than one"
assert (true)
x = 5
assert (x > 0) "x must be positive"
```

**ActiveForm:** Fixing assert message parsing

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/LX_PARSER_FIXES.md" })
```
