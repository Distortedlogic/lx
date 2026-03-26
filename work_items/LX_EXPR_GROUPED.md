# Goal

Add `Expr::Grouped(ExprId)` to the AST — a transparent wrapper that marks parenthesized expressions. This is standard compiler practice (GCC has PAREN_EXPR, Clang has ParenExpr, Roslyn has ParenthesizedExpressionSyntax). It solves a class of parser ambiguities where the parser loses the information that parens were used.

# Why

The parser currently discards parenthesization — `(x)` produces the same AST as `x`. This causes:

- **Assert messages**: `assert (cond) "msg"` can't be distinguished from `assert (f "arg")` because both are Apply at the AST level. With Grouped, the fixup checks `Apply(Grouped(_), Str)` — the Grouped wrapper proves parens were used.
- **Lambda vs application**: `filter (x) { x > 1 }` — `(x)` is indistinguishable from a standalone expression. With Grouped, the func_def parser can check for `Grouped(params)` followed by `{ body }`.
- **Formatter fidelity**: the formatter can't preserve user parenthesization because the AST doesn't record it.
- **Future syntax**: any construct where "parenthesized expr followed by something" has different meaning than "bare expr followed by something."

# What Changes

**AST — `crates/lx/src/ast/mod.rs` line 64:**

Add to the `Expr` enum:
```rust
Grouped(ExprId),
```

The `AstWalk` derive macro auto-generates `walk_children`, `recurse_children`, and `children` for the new variant since `ExprId` is a recognized walkable type.

**Parser — `crates/lx/src/parser/expr_compound.rs` line 120-123:**

The `grouped` parser currently clones the inner node:
```rust
let grouped = just(TokenKind::LParen).ignore_then(expr).then_ignore(just(TokenKind::RParen)).map_with(move |inner, e| {
    let node = arena.borrow().expr(inner).clone();
    arena.borrow_mut().alloc_expr(node, ss(e.span()))
});
```

Change to wrap in `Expr::Grouped`:
```rust
let grouped = just(TokenKind::LParen).ignore_then(expr).then_ignore(just(TokenKind::RParen)).map_with(move |inner, e| {
    arena.borrow_mut().alloc_expr(Expr::Grouped(inner), ss(e.span()))
});
```

This is simpler (no clone, no double borrow) and preserves the grouping information.

**7 exhaustive match sites — add transparent recursion:**

All 7 sites handle Grouped by recursing into the inner expression:

1. `formatter/emit_expr.rs:36` — `Expr::Grouped(inner) => { self.write("("); self.emit_expr(*inner); self.write(")"); }`
2. `interpreter/mod.rs:123` — `Expr::Grouped(inner) => self.eval(*inner).await`
3. `checker/check_expr.rs:14` — `Expr::Grouped(inner) => self.check_expr(*inner, expected)`
4. `checker/type_ops.rs:21` — `Expr::Grouped(inner) => self.synth_expr(*inner)`
5. `visitor/walk/mod.rs:178` — `Expr::Grouped(inner) => dispatch_expr(v, *inner, arena)?`
6. `folder/desugar.rs:55` — `Expr::Grouped(_) => expr` (pass through, not desugared)
7. `stdlib/diag/diag_helpers.rs:13` (unwrap_propagate) — `Expr::Grouped(inner) => unwrap_propagate(arena.expr(*inner), arena)`

Also update `expr_label` in `diag_helpers.rs:34` to recurse into Grouped.

**Assert message fix — `crates/lx/src/parser/expr.rs` lines 138-144:**

After Grouped exists, the assert parser can use the post-parse fixup safely:

```rust
let assert_expr = {
    let al = arena.clone();
    just(TokenKind::Assert)
        .ignore_then(expr.clone())
        .map_with(move |ex, e| {
            let (cond, msg) = {
                let ar = al.borrow();
                if let Expr::Apply(app) = ar.expr(ex)
                    && let Expr::Grouped(_) = ar.expr(app.func)
                    && let Expr::Literal(Literal::Str(_)) = ar.expr(app.arg)
                {
                    (app.func, Some(app.arg))
                } else {
                    (ex, None)
                }
            };
            al.borrow_mut().alloc_expr(Expr::Assert(ExprAssert { expr: cond, msg }), ss(e.span()))
        })
};
```

The check: Apply where func is Grouped AND arg is Str → split into condition + message. This distinguishes:
- `assert (cond) "msg"` → Apply(Grouped(cond), Str) → split ✓
- `assert (s.has "a")` → Grouped(Apply(s.has, "a")) → not Apply at top level → no split ✓
- `assert s.has "a"` → Apply(FieldAccess, Str) → func is NOT Grouped → no split ✓

**Visitor trait — `crates/lx/src/visitor/visitor_trait.rs`:**

Add visit/leave hooks:
```rust
fn visit_grouped(&mut self, _id: ExprId, _inner: ExprId, _span: SourceSpan) -> VisitAction { VisitAction::Descend }
fn leave_grouped(&mut self, _id: ExprId, _inner: ExprId, _span: SourceSpan) {}
```

**walk/generated.rs:**

Add dispatch:
```rust
define_walk_and_dispatch!(walk_grouped_dispatch, walk_grouped, visit_grouped, leave_grouped, ExprId, ExprId);
```

Wait — `define_walk_and_dispatch!` expects a struct type, not ExprId. Grouped is just `Expr::Grouped(ExprId)` — a single ExprId, not a struct. The walk for Grouped should dispatch into the inner expression:

In `walk/mod.rs` `walk_expr`, add:
```rust
Expr::Grouped(inner) => dispatch_expr(v, *inner, arena)?,
```

No separate walk/dispatch function needed. The visitor hooks are optional — Grouped is transparent for most visitors.

# Files Affected

- `crates/lx/src/ast/mod.rs` — Add Expr::Grouped(ExprId)
- `crates/lx/src/parser/expr_compound.rs` — Wrap in Grouped instead of clone
- `crates/lx/src/parser/expr.rs` — Assert message fixup using Grouped check
- `crates/lx/src/formatter/emit_expr.rs` — Emit parens around inner
- `crates/lx/src/interpreter/mod.rs` — Eval inner
- `crates/lx/src/checker/check_expr.rs` — Check inner
- `crates/lx/src/checker/type_ops.rs` — Synth inner
- `crates/lx/src/visitor/walk/mod.rs` — Dispatch inner
- `crates/lx/src/folder/desugar.rs` — Pass through
- `crates/lx/src/stdlib/diag/diag_helpers.rs` — Recurse inner

# Task List

### Task 1: Add Expr::Grouped to AST

**Subject:** Add Grouped(ExprId) variant to Expr enum

**Description:** Edit `crates/lx/src/ast/mod.rs`. Add `Grouped(ExprId)` to the `Expr` enum (line 64 area, after the last variant). The `AstWalk` derive handles walk_children/recurse_children/children automatically for ExprId fields.

**ActiveForm:** Adding Expr::Grouped

---

### Task 2: Update parser to emit Grouped

**Subject:** Change grouped parser from cloning inner to wrapping in Grouped

**Description:** Edit `crates/lx/src/parser/expr_compound.rs` lines 120-123. Change:
```rust
let grouped = just(TokenKind::LParen).ignore_then(expr).then_ignore(just(TokenKind::RParen)).map_with(move |inner, e| {
    let node = arena.borrow().expr(inner).clone();
    arena.borrow_mut().alloc_expr(node, ss(e.span()))
});
```
To:
```rust
let grouped = just(TokenKind::LParen).ignore_then(expr).then_ignore(just(TokenKind::RParen)).map_with(move |inner, e| {
    arena.borrow_mut().alloc_expr(Expr::Grouped(inner), ss(e.span()))
});
```

**ActiveForm:** Updating parser for Grouped

---

### Task 3: Update all 7 exhaustive match sites

**Subject:** Add Grouped arm to every exhaustive Expr match

**Description:** Add these arms:

1. `crates/lx/src/formatter/emit_expr.rs` — `Expr::Grouped(inner) => { self.write("("); self.emit_expr(*inner); self.write(")"); }`
2. `crates/lx/src/interpreter/mod.rs` — `Expr::Grouped(inner) => self.eval(*inner).await`
3. `crates/lx/src/checker/check_expr.rs` — Add Grouped to the catch-all arm that calls `self.synth_expr` (or handle explicitly by checking inner with expected type)
4. `crates/lx/src/checker/type_ops.rs` — `Expr::Grouped(inner) => self.synth_expr(*inner)`
5. `crates/lx/src/visitor/walk/mod.rs` — `Expr::Grouped(inner) => dispatch_expr(v, *inner, arena)?`
6. `crates/lx/src/folder/desugar.rs` — Falls through to `other => other` (Grouped is not desugared)
7. `crates/lx/src/stdlib/diag/diag_helpers.rs` unwrap_propagate — `Expr::Grouped(inner) => unwrap_propagate(arena.expr(*inner), arena)`

Also update `expr_label` in `diag_helpers.rs` — `Expr::Grouped(inner) => expr_label(arena.expr(*inner), arena)`

**ActiveForm:** Updating exhaustive match sites

---

### Task 4: Implement assert message using Grouped

**Subject:** Post-parse fixup checks Apply(Grouped, Str) for assert messages

**Description:** Edit `crates/lx/src/parser/expr.rs` lines 138-144. Replace the assert parser with:

```rust
let assert_expr = {
    let al = arena.clone();
    just(TokenKind::Assert)
        .ignore_then(expr.clone())
        .map_with(move |ex, e| {
            let (cond, msg) = {
                let ar = al.borrow();
                if let Expr::Apply(app) = ar.expr(ex)
                    && let Expr::Grouped(_) = ar.expr(app.func)
                    && let Expr::Literal(Literal::Str(_)) = ar.expr(app.arg)
                {
                    (app.func, Some(app.arg))
                } else {
                    (ex, None)
                }
            };
            al.borrow_mut().alloc_expr(Expr::Assert(ExprAssert { expr: cond, msg }), ss(e.span()))
        })
};
```

The condition for splitting: the top-level expr is Apply, the func is Grouped (was parenthesized), and the arg is a string literal. This precisely matches `assert (cond) "msg"` without false positives on `assert (s.has "a")` or `assert s.has "a"`.

Write test `tests/assert_message.lx`:
```lx
assert (1 == 1) "one equals one"
assert (2 > 1) "two is greater"
assert (true)
x = 5
assert (x > 0) "x must be positive"
assert (x == 5)
list = [1; 2; 3]
assert (list | len == 3)
has = (s.has "a") ?? false
```

**ActiveForm:** Implementing assert messages with Grouped

---

### Task 5: Run full test suite

**Subject:** Verify no regressions from Grouped addition

**Description:** Run ALL tests:
- `just rust-diagnose`
- `cargo test -p lx --test formatter_roundtrip`
- All tests/*.lx files
- All programs/workgen/tests/*.lx files
- All programs/workrunner/tests/*.lx files

The Grouped wrapper is transparent at runtime (evaluator just evals inner) so existing behavior should be preserved. The formatter now emits explicit parens for Grouped nodes, which may affect formatter roundtrip — verify and fix.

**ActiveForm:** Full regression testing

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/LX_EXPR_GROUPED.md" })
```
