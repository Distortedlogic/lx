# Goal

Eliminate per-call heap allocation in AST traversal by switching `children()` to `SmallVec`, replace the hardcoded type-name string lists in the `AstWalk` derive macro with a marker-trait approach, and generate walk functions from the macro so hand-written walk functions can be removed.

# Why

- Every call to `.children()` allocates a `Vec<NodeId>` on the heap. Nodes like `ExprBinary` (2 children) and `ExprUnary` (1 child) cause allocations that a `SmallVec<[NodeId; 4]>` would avoid entirely
- `field_strategy.rs` in `lx-macros` has hardcoded `PASSTHROUGH_TYPES` and `WALKABLE_TYPES` string arrays. Adding a new AST struct requires manually updating these lists — if forgotten, the macro silently treats the type as passthrough and skips walking its children
- Walk functions in `walk_expr.rs`, `walk_expr2.rs`, `walk_pattern.rs`, `walk_type.rs` are hand-written for every node type. Most just call `dispatch_children(v, &node.children(), arena)` then the leave hook — pure boilerplate that the macro could generate

# What changes

1. Add `smallvec` dependency to `crates/lx/Cargo.toml`
2. Change `children()` return type from `Vec<NodeId>` to `SmallVec<[NodeId; 4]>` in all generated and hand-written impls
3. Replace hardcoded `PASSTHROUGH_TYPES` and `WALKABLE_TYPES` in `field_strategy.rs` with a pattern: types that derive `AstWalk` get a generated marker const (e.g., `const _AST_WALKABLE: () = ();`), and the macro checks for this const via a type-level assertion. Alternatively, since the macro operates at compile time and cannot check for trait impls, use a simpler approach: add a `#[walk(walkable)]` attribute that types use to opt in, and remove the hardcoded lists. All types that currently derive `AstWalk` are walkable by definition — the list just needs to include types that the *parent* macro sees. The simplest robust fix: move the classification from string-matching to checking whether the type has `#[derive(AstWalk)]` by requiring all walkable child types to also derive AstWalk, and having the macro emit a known associated item that the parent can reference
4. Generate `walk_*` functions from the macro that iterate fields directly (not via `children()`) for AstVisitor traversal. This eliminates the hand-written boilerplate in `walk_expr.rs`, `walk_expr2.rs`, etc.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/Cargo.toml` | Add smallvec dependency |
| `crates/lx-macros/src/field_strategy.rs` | Replace PASSTHROUGH_TYPES/WALKABLE_TYPES with attribute-based classification |
| `crates/lx-macros/src/walk_enum.rs` | Update children() codegen to emit SmallVec |
| `crates/lx-macros/src/walk_enum_children.rs` | Update children() codegen to emit SmallVec |
| `crates/lx-macros/src/walk_struct.rs` | Update children() and recurse_children() codegen |
| `crates/lx-macros/src/lib.rs` | Update derive macro entry point |
| `crates/lx/src/ast/arena.rs` | Import and re-export SmallVec, update NodeId-related code |
| `crates/lx/src/ast/walk_impls.rs` | Change hand-written children() to return SmallVec |
| `crates/lx/src/ast/parent_map.rs` | Update to consume SmallVec from children() |
| `crates/lx/src/visitor/walk/mod.rs` | Update dispatch_children to accept SmallVec or slice |
| `crates/lx/src/visitor/walk/walk_expr.rs` | Replace with macro-generated walk functions (or thin wrappers) |
| `crates/lx/src/visitor/walk/walk_expr2.rs` | Replace with macro-generated walk functions |
| `crates/lx/src/visitor/walk/walk_pattern.rs` | Replace with macro-generated walk functions |
| `crates/lx/src/visitor/walk/walk_type.rs` | Replace with macro-generated walk functions |
| `crates/lx/src/folder/validate_core.rs` | Update children() call sites |

# Task List

### Task 1: Add smallvec dependency

In `crates/lx/Cargo.toml`, add `smallvec = { version = "1" }` to `[dependencies]`.

### Task 2: Change children() return type to SmallVec

In `crates/lx-macros/src/walk_enum_children.rs` and `crates/lx-macros/src/walk_struct.rs`, update the generated `children()` method signature from `pub fn children(&self) -> Vec<crate::ast::NodeId>` to `pub fn children(&self) -> smallvec::SmallVec<[crate::ast::NodeId; 4]>`.

Update the codegen to use `smallvec::smallvec![]` for constructing return values instead of `vec![]`. For nodes with known small child counts (0-4), the compiler will use the inline buffer. For nodes with variable children (Vec fields), use `SmallVec::from_iter(...)` or `.collect()`.

In `crates/lx/src/ast/walk_impls.rs`, update the three hand-written `children()` methods on `WithKind`, `TraitDeclData`, and `ClassDeclData` to return `SmallVec<[NodeId; 4]>` instead of `Vec<NodeId>`. Replace `vec![]` with `smallvec::smallvec![]` and `Vec::new()` with `SmallVec::new()`.

### Task 3: Update dispatch_children and callers

In `crates/lx/src/visitor/walk/mod.rs`, change `dispatch_children` to accept a slice:

```rust
fn dispatch_children<V: AstVisitor + ?Sized>(v: &mut V, children: &[NodeId], arena: &AstArena) -> ControlFlow<()>
```

This already works because `SmallVec` implements `Deref<Target=[T]>`, so callers passing `&node.children()` will auto-deref to a slice. Verify all callers compile.

In `crates/lx/src/ast/parent_map.rs`, update `build_parent_map` to work with SmallVec (it iterates children, so it should work as-is since SmallVec is iterable).

In `crates/lx/src/folder/validate_core.rs`, the `for child in expr.children()` loop should work as-is since SmallVec implements IntoIterator.

### Task 4: Replace hardcoded type lists with attribute-based opt-in

In `crates/lx-macros/src/field_strategy.rs`:

Remove the `PASSTHROUGH_TYPES` and `WALKABLE_TYPES` const arrays entirely.

Replace `classify_type` logic with an inverted-default approach:

1. ID types (ExprId, StmtId, PatternId, TypeExprId) — detect by exact name match (keep as-is)
2. Vec/Option wrappers of IDs — detect by generic arg name match (keep as-is)
3. **Invert the default:** Keep a single `PASSTHROUGH_TYPES` list containing only primitives that are obviously not AST nodes: `Sym`, `BinOp`, `UnaryOp`, `bool`, `i64`, `f64`, `usize`, `String`, `BigInt`, `UseKind`, `UseStmt`, `StmtTypeDef`, `TraitUnionDef`. Remove the `WALKABLE_TYPES` list entirely. All other non-ID, non-primitive types are treated as walkable (default to WalkableStruct instead of Passthrough). If a type lacks a `children()` method, the compiler emits a clear error — which forces the developer to either derive AstWalk on it or add it to the passthrough list.

This eliminates the maintenance trap where adding a new AST struct type requires updating a hardcoded list — the compiler enforces correctness.

### Task 5: Generate walk functions from the AstWalk derive macro

Extend the `AstWalk` derive macro to generate a `walk_*` function for each derived type that performs direct field-by-field visitor dispatch without going through `children()`.

For each field classified as an ID type, generate a direct dispatch call:
- ExprId → `super::dispatch_expr(v, self.field, arena)?;`
- StmtId → `super::dispatch_stmt(v, self.field, arena)?;`
- PatternId → `super::walk_pattern_dispatch(v, self.field, arena)?;`
- TypeExprId → `super::walk_type_expr_dispatch(v, self.field, arena)?;`

For Vec<Id> fields, generate a loop.
For Option<Id> fields, generate an if-let.
For walkable struct fields, generate a `dispatch_children(v, &self.field.children(), arena)?;` call.

The generated walk function goes into the type's impl block as `pub fn walk_children<V: crate::visitor::AstVisitor + ?Sized>(&self, v: &mut V, arena: &crate::ast::AstArena) -> std::ops::ControlFlow<()>`. Note: this method does NOT take an ID parameter because different node types use different ID types (ExprId, PatternId, etc.). The method only walks children — the caller handles the leave hook.

Then update the hand-written walk functions in `walk_expr.rs`, `walk_expr2.rs`, `walk_pattern.rs`, `walk_type.rs` to delegate to the generated method. For example, `walk_binary` becomes:

```rust
pub fn walk_binary<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, binary: &ExprBinary, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
    binary.walk_children(v, arena)?;
    v.leave_binary(id, binary, span, arena)
}
```

This preserves the dispatch/walk/leave pattern while eliminating the per-node boilerplate.

The following 6 walk functions must remain hand-written because they inline-iterate slice fields instead of going through children():
- `walk_block` — iterates `&[StmtId]` via `dispatch_stmt`
- `walk_tuple` — iterates `&[ExprId]` via `dispatch_expr`
- `walk_loop` — iterates `&[StmtId]` via `dispatch_stmt`
- `walk_par` — iterates `&[StmtId]` via `dispatch_stmt`
- `walk_propagate` — dispatches a single `ExprId` (not a struct, just a raw ID in the enum)
- `walk_break` — dispatches an `Option<ExprId>` (same reason)

### Task 6: Format and commit

Run `just fmt` then `git add -A && git commit -m "refactor: SmallVec children, attribute-based walk classification, generated walk functions"`.

### Task 7: Verify

Run `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **The macro crate is `crates/lx-macros`** — changes there require rebuilding the proc macro before the main crate can use them.
5. **SmallVec with capacity 4** covers all fixed-child nodes. Variable-child nodes (Vec fields) will spill to heap — that is expected and correct.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AST_WALK_CODEGEN.md" })
```

Then call `next_task` to begin.
