# Goal

Fix three parser bugs that force workarounds in every lx program: lambda body greedily consuming pipes, assert message parsing, and trait body name conflict.

# Why

- `filter (e) { e > 1 } | len` parses `| len` inside the lambda body. Every program splits filter+len into two lines.
- `assert (condition) "message"` parses as applying the Bool result to the String. Every test omits assert messages.
- `{name: value}` record literals inside trait method bodies are misinterpreted as trait field declarations. Programs use semicolons or avoid record literals in methods.

# Root Causes (verified by reading source)

**Lambda body is greedy (pipe-after-block AND assert message — same root cause):**

`crates/lx/src/parser/expr_compound.rs` line 104-113: the `func_def` parser matches `(params) [type_params] [-> ret_type] [& guard] body` where `body` is `expr.clone()` (line 110) — the FULL expression parser including pipes. So `(x) { x > 2 } | len` — the func_def matches `(x)` as params, then `{ x > 2 } | len` as the body. The `{ x > 2 }` block is parsed, then `| len` is consumed as a pipe on the block result inside the lambda body.

Same root cause for assert: line 138-143 in `expr.rs`, assert uses `.ignore_then(expr.clone()).then(expr.clone().or_not())`. The first `expr.clone()` greedily consumes `(cond) "msg"` as an Apply expression (Bool applied to String).

**Trait body name conflict:**

`crates/lx/src/parser/stmt.rs` `trait_body()` at line 228: `field_entry` matches `ident : type_name`. The `type_name()` parser only matches capitalized identifiers and keyword tokens — NOT lowercase identifiers. So `name: rub.name` should NOT match as a field entry because `rub` is lowercase. If it does match, the issue is that chumsky's `choice` doesn't backtrack after `ident :` partially succeeds with `field_entry` but then `type_name()` fails on the lowercase token. The parser error cascades instead of falling through to `default_method`.

# Files Affected

- `crates/lx/src/parser/expr_compound.rs` — Fix func_def body parsing (line 110)
- `crates/lx/src/parser/expr.rs` — Fix assert parsing (line 138-143)
- `crates/lx/src/parser/stmt.rs` — Verify/fix trait_body backtracking

# Task List

### Task 1: Fix lambda body — block body stops at `}`

**Subject:** Lambda with `{ }` body should not consume pipe after closing brace

**Description:** Edit `crates/lx/src/parser/expr_compound.rs`. The `func_def` parser at line 104-113 has `.then(expr.clone())` at line 110 for the body. This consumes the full expression including pipes.

Fix: when the body starts with `{`, parse only the block (stops at `}`). Otherwise parse the full expression (for bare bodies like `(x) x + 1`).

Change line 110 from:
```rust
.then(expr.clone())
```
to:
```rust
.then(
    just(TokenKind::LBrace)
        .ignore_then(super::expr::stmts_block(expr.clone(), a_body.clone()))
        .then_ignore(just(TokenKind::RBrace))
        .map_with(move |stmts, e| a_body.borrow_mut().alloc_expr(Expr::Block(ExprBlock { stmts }), ss(e.span())))
        .or(expr.clone())
)
```

You'll need an additional ArenaRef clone (`a_body`) for the block body parser. Import `ExprBlock` from `crate::ast`.

The block alternative is tried first (because `or` tries left then right in chumsky). If `{` is the next token, it parses the block and stops at `}`. If not, falls through to the full expression parser.

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
```

**ActiveForm:** Fixing lambda block body parsing

---

### Task 2: Fix assert message parsing

**Subject:** Assert condition should not consume the message string as an argument

**Description:** Edit `crates/lx/src/parser/expr.rs` lines 138-143. Current assert parser:

```rust
just(TokenKind::Assert)
    .ignore_then(expr.clone())
    .then(expr.clone().or_not())
```

The first `expr.clone()` greedily consumes `(cond) "msg"` as Apply(cond, msg). The `.then(expr.clone().or_not())` gets nothing because everything was consumed.

Fix: the assert condition should be parsed at a restricted precedence that stops before function application of a string literal. The simplest fix — post-parse: after parsing the full expression, check if it's an `Apply` where the argument is a string literal. If so, split: the function part is the condition, the string argument is the message.

```rust
let assert_expr = {
    let al = arena.clone();
    let al2 = arena.clone();
    just(TokenKind::Assert)
        .ignore_then(expr.clone())
        .map_with(move |ex, e| {
            let arena_ref = al.borrow();
            // Check if expr is Apply(cond, Literal::Str(...))
            if let Expr::Apply(app) = arena_ref.expr(ex) {
                if let Expr::Literal(Literal::Str(_)) = arena_ref.expr(app.arg) {
                    let cond = app.func;
                    let msg = Some(app.arg);
                    return al2.borrow_mut().alloc_expr(
                        Expr::Assert(ExprAssert { expr: cond, msg }),
                        ss(e.span()),
                    );
                }
            }
            drop(arena_ref);
            al2.borrow_mut().alloc_expr(
                Expr::Assert(ExprAssert { expr: ex, msg: None }),
                ss(e.span()),
            )
        })
};
```

This checks the parsed expression: if it's `Apply(cond, string_literal)`, split into assert with condition and message. Otherwise, it's a plain assert with no message. No parser restructuring needed — just a post-parse fixup.

Note: this requires borrowing the arena to inspect the expression, then dropping the borrow before mutably borrowing to allocate. Handle the borrow carefully.

Write test `tests/parser_assert_message.lx`:
```lx
assert (1 == 1) "one equals one"
assert (2 > 1) "two greater than one"
assert (true)
x = 5
assert (x > 0) "x must be positive"
```

All should pass. For a failing assert with message, verify the message appears in the error output.

**ActiveForm:** Fixing assert message parsing

---

### Task 3: Fix trait body backtracking

**Subject:** Ensure trait body parser falls through to default_method when field_entry partially fails

**Description:** Read `crates/lx/src/parser/stmt.rs` `trait_body()` around line 228. The `field_entry` parser matches `ident : type_name`. The `type_name()` in `expr.rs` only matches `TokenKind::TypeName` and keyword tokens — NOT lowercase `TokenKind::Ident`. So `name: rub.name` should fail at `type_name()` because `rub` is lowercase.

Test whether the current parser already handles this correctly:

```
echo '+Trait T = { m = () { {name: "hello"} } }' > /tmp/test_trait_record.lx
cargo run -p lx-cli -- run /tmp/test_trait_record.lx
```

If it parses, trait body backtracking already works — task is done (just add a regression test).

If it fails, the issue is that chumsky's `choice` in `trait_body` partially consumes `ident :` before `type_name()` fails, and doesn't backtrack. Fix by wrapping `field_entry` in `.try_map()` or restructuring so `ident :` is not consumed until `type_name()` succeeds. In chumsky, use `.then(type_name()).rewind()` or an explicit `.try()` to enable backtracking.

Write test `tests/parser_trait_record.lx`:
```lx
+Trait HasMethod = {
  describe = () {
    n = "test"
    {name: n; value: 42}
  }
}
Class Impl : [HasMethod] = {}
obj = Impl {}
result = obj.describe ()
assert (result.name == "test")
assert (result.value == 42)
```

**ActiveForm:** Fixing trait body backtracking

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
