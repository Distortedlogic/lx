# Goal

Eliminate the double-clone and PartialEq-based deduplication in `walk_transform_fn!`, and allow transformers to update spans by changing leave hooks to return `(Node, SourceSpan)`.

# Why

- `walk_transform_fn!` clones the original node from the arena (line 9: `arena.$get_node(id).clone()`), then clones it again to pass to the transform method (line 10: `t.$transform(id, original.clone(), ...)`). Every node in the AST is cloned at least once per transformation pass, even when the transformer does nothing to it
- After transformation, the result is compared to the original via `PartialEq` to decide whether to allocate a new arena slot. This deep equality check traverses the entire subtree and is likely more expensive than the arena allocation it saves — arena alloc is an append to a vec
- Transformer leave hooks return only the node, not the span. There is no way for a transformer to update the span of a synthesized node. The span is always copied from the original

# What changes

1. Restructure `walk_transform_fn!` to avoid the double clone: pass the original by reference to the transform hook for inspection, clone only when `TransformOp::Continue` (needs recursion) or `TransformOp::Skip` (replacement provided)
2. Remove the `PartialEq` deduplication check — always allocate a new arena slot for changed nodes. The arena is append-only; the old slot becomes unreachable but costs no ongoing overhead
3. Change leave hook signatures to return `(Node, SourceSpan)` so transformers can update spans

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/visitor/transformer.rs` | Change transform hook signatures to receive &Node, change leave hooks to return (Node, SourceSpan) |
| `crates/lx/src/visitor/walk_transform/mod.rs` | Rewrite walk_transform_fn! macro to avoid double-clone and PartialEq |
| `crates/lx/src/folder/desugar.rs` | Update Desugarer impl to match new signatures |

# Task List

### Task 1: Update AstTransformer trait signatures

In `crates/lx/src/visitor/transformer.rs`:

Change `TransformOp` to not carry a node in the Continue case — instead, Continue means "clone from arena and recurse":

```rust
pub enum TransformOp<T> {
    Continue,
    Replace(T),
    Stop,
}
```

`Continue` — clone the node from the arena, recurse into children, then call leave.
`Replace(T)` — use this replacement node, call leave (no child recursion).
`Stop` — return original ID unchanged, do not call leave.

Change the transform hook signatures to receive a reference:

```rust
fn transform_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) -> TransformOp<Expr> {
    TransformOp::Continue
}
```

Note: `arena` parameter changes from `&mut AstArena` to `&AstArena` in the transform hooks — mutation only happens in leave hooks and during arena allocation (which the walk function handles).

Change leave hooks to return a tuple of node and span:

```rust
fn leave_expr(&mut self, _id: ExprId, expr: Expr, span: SourceSpan, _arena: &mut AstArena) -> (Expr, SourceSpan) {
    (expr, span)
}
```

Apply the same changes to `transform_stmt`/`leave_stmt`, `transform_pattern`/`leave_pattern`, `transform_type_expr`/`leave_type_expr`.

### Task 2: Rewrite walk_transform_fn! macro

In `crates/lx/src/visitor/walk_transform/mod.rs`, rewrite the macro:

```rust
macro_rules! walk_transform_fn {
    ($fn_name:ident, $id_ty:ty, $transform:ident, $leave:ident, $get_span:ident, $get_node:ident, $alloc:ident) => {
        pub fn $fn_name<T: AstTransformer + ?Sized>(t: &mut T, id: $id_ty, arena: &mut AstArena) -> $id_ty {
            let span = arena.$get_span(id);
            let action = {
                let node_ref = arena.$get_node(id);
                t.$transform(id, node_ref, span, arena)
            };
            match action {
                TransformOp::Stop => id,
                TransformOp::Replace(node) => {
                    let (final_node, final_span) = t.$leave(id, node, span, arena);
                    arena.$alloc(final_node, final_span)
                },
                TransformOp::Continue => {
                    let node = arena.$get_node(id).clone();
                    let recursed = node.recurse_children(t, arena);
                    let (final_node, final_span) = t.$leave(id, recursed, span, arena);
                    arena.$alloc(final_node, final_span)
                },
            }
        }
    };
}
```

Key changes:
- No clone for Stop (returns original ID)
- Single clone for Continue (clone from arena, recurse, leave, alloc)
- No clone for Replace (transformer provides the node)
- No PartialEq comparison — always alloc for non-Stop paths
- Leave hooks return (node, span), alloc uses the returned span

**Borrow checker solution:** The function takes `arena: &mut AstArena`. Inside the block, `arena.$get_node(id)` creates a shared reborrow `&AstArena`. The `&*arena` in the transform call creates another shared reborrow. Multiple shared reborrows of a `&mut` ref are allowed simultaneously. After the block ends, both shared borrows are dropped and the `&mut` is available again for `leave` and `alloc` calls. In the `Continue` branch, `arena.$get_node(id).clone()` produces an owned value (ending the borrow), then `node.recurse_children(t, arena)` takes `&mut AstArena` — no conflict because the clone owns the data independently.

The block-scoped reborrow pattern:

```rust
let action = {
    let node_ref = arena.$get_node(id);
    t.$transform(id, node_ref, span, &*arena)
};
```

### Task 3: Update Desugarer

In `crates/lx/src/folder/desugar.rs`, update the `Desugarer` impl:

The `leave_expr` signature changes from `fn leave_expr(&mut self, _id: ExprId, expr: Expr, span: SourceSpan, arena: &mut AstArena) -> Expr` to `fn leave_expr(&mut self, _id: ExprId, expr: Expr, span: SourceSpan, arena: &mut AstArena) -> (Expr, SourceSpan)`.

Wrap all return values in `(result, span)`. The Desugarer always preserves the original span, so every return becomes `(transformed_expr, span)`. The `other => other` catch-all becomes `other => (other, span)`.

Verify all the helper functions (`desugar_section`, `desugar_ternary`, `desugar_coalesce`, `desugar_with_binding`, `desugar_interp`) still work — they return `Expr`, not tuples. The leave_expr wraps their result: `(desugar_section(s, span, arena), span)`.

### Task 4: Format and commit

Run `just fmt` then `git add -A && git commit -m "refactor: eliminate double-clone in transformer, add span update support"`.

### Task 5: Verify

Run `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **Watch the borrow checker** — the transform hook receives `&AstArena` (shared ref) while the walk function holds `&mut AstArena`. Use block scoping to ensure the shared borrow ends before any mutation.
5. **`recurse_children` takes `self` by value** — it consumes the cloned node. This is correct and unchanged.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/TRANSFORMER_EFFICIENCY.md" })
```

Then call `next_task` to begin.
