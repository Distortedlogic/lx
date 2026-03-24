# Goal

Skip arena allocation in the AST transformer when the node is structurally unchanged after recursion and leave.

# Prerequisites

None.

# Why

- The `TransformOp::Continue` branch in `walk_transform_fn!` always allocates a new arena slot even when nothing changed. A desugaring pass that transforms 5 nodes in a 1000-node AST allocates 1000 new entries. Old entries become unreachable garbage in the append-only arena.

# Verified preconditions

- All AST node types (`Expr`, `Stmt`, `Pattern`, `TypeExpr`) derive `PartialEq` — verified in `ast/mod.rs` line 31, 46, 62; `ast/types.rs` lines 20, 99, 105, 106; `ast/expr_types.rs` throughout
- `Literal` has a manual `PartialEq` impl (expr_types.rs line 35-49) using `to_bits()` for Float — this is correct for structural comparison
- `SourceSpan` from miette is `Copy + PartialEq`
- The arena is append-only — `arena.$get_node(id)` returns a stable reference as long as no mutable borrow is active
- After `t.$leave(id, recursed, span, arena)` completes, the `&mut AstArena` borrow is released, making `arena.$get_node(id)` valid for the comparison

# What changes

One file, one macro rewrite.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/visitor/walk_transform/mod.rs` | Rewrite Continue branch of walk_transform_fn! |

# Task List

### Task 1: Rewrite walk_transform_fn! macro

In `crates/lx/src/visitor/walk_transform/mod.rs`, replace the entire macro with:

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
                    if final_span == span && final_node == *arena.$get_node(id) {
                        id
                    } else {
                        arena.$alloc(final_node, final_span)
                    }
                },
            }
        }
    };
}
```

The comparison `final_node == *arena.$get_node(id)` is valid because:
1. `t.$leave(...)` takes `arena: &mut AstArena` — this mutable borrow ends when `leave` returns
2. `arena.$get_node(id)` re-borrows arena immutably — valid after the mutable borrow ends
3. The original node at `id` is never modified (arena is append-only, `alloc` appends)
4. `PartialEq` on all node types compares child IDs (which are `Copy` integers) — cheap for unchanged nodes

For unchanged subtrees: all child IDs match, `final_span == span`, so `final_node == *arena.$get_node(id)` is true. Returns `id` with zero allocation.

For the Desugarer: `leave_expr` transforms Pipe→Apply, Section→lambda, etc. The `final_node` differs from the original, so `PartialEq` returns false and allocation proceeds normally.

### Task 2: Verify

Run `just fmt` then `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

### Task 3: Commit

Run `just fmt` then `git add -A && git commit -m "perf: skip arena allocation in transformer when node is unchanged"`.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **Do not change the `Replace` or `Stop` branches** — only the `Continue` branch is modified.
5. **The `walk_transform_fn!` invocations below the macro** (lines 30-33) are unchanged — they instantiate the macro for stmt/expr/pattern/type_expr.
6. **`walk_transform_program`** (lines 35-40) is unchanged — it calls `walk_transform_stmt` which uses the updated macro.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/TRANSFORMER_SKIP_UNCHANGED.md" })
```

Then call `next_task` to begin.
