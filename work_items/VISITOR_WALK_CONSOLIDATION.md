# Goal

Merge `PatternVisitor` and `TypeVisitor` into `AstVisitor` as a single trait, then extend the `AstWalk` derive macro to generate per-variant dispatch and walk functions for `Expr`, `Pattern`, and `TypeExpr` enums — eliminating the ~677 lines of handwritten walk code in `walk_expr.rs`, `walk_expr2.rs`, `walk_pattern.rs`, and `walk_type.rs`, plus the `walk_dispatch_id!` and `walk_dispatch_id_slice!` macros.

# Why

- `AstVisitor` requires `PatternVisitor + TypeVisitor` as supertraits. Every implementor must provide or default all three traits. No consumer ever implements `PatternVisitor` or `TypeVisitor` in isolation — the split adds three files and two trait bounds with no compositional benefit
- `walk_expr.rs` (120 lines), `walk_expr2.rs` (139 lines), `walk_pattern.rs` (179 lines), and `walk_type.rs` (239 lines) are almost entirely boilerplate. Each walk function follows the same pattern: call `.walk_children(v, arena)?`, then call the leave hook. Each dispatch function follows the same pattern: call `visit_*`, check the action, call the walk function or skip
- The `walk_dispatch_id!` and `walk_dispatch_id_slice!` macros (lines 8-38 of `walk/mod.rs`) generate dispatch functions that the walk files invoke ~40 times. These macro invocations and their generated functions should instead be produced by the `AstWalk` derive macro on the enum itself
- Adding a new `Expr` variant currently requires: adding the variant to the enum, adding a visitor hook to the trait, adding a dispatch macro invocation, adding a walk function, and adding a match arm in `walk_expr`. With macro generation, it requires only: adding the variant with a `#[walk(visit = "...", leave = "...")]` attribute

# Prerequisites

This work item depends on **UNIVERSAL_ASTWALK_DERIVATION** being complete. All AST types must derive `AstWalk` and the `walk_impls.rs` file must be deleted before this work item begins.

# What changes

**Trait consolidation:** `PatternVisitor` and `TypeVisitor` method definitions move into `AstVisitor`. The two trait files are deleted. The supertrait bounds are removed. All consumer `impl PatternVisitor for X` and `impl TypeVisitor for X` blocks merge into their `impl AstVisitor for X` blocks.

**Macro extension:** The `AstWalk` derive macro gains the ability to generate dispatch and walk functions when applied to `Expr`, `Pattern`, and `TypeExpr` enums. Each variant gets a `#[walk(visit = "visit_binary", leave = "leave_binary")]` attribute that tells the macro which visitor methods to call. The macro generates:
1. A dispatch function per variant: calls visit, checks action, calls walk, calls leave
2. A walk function per variant: calls `.walk_children(v, arena)?` on the inner data, then calls leave
3. The top-level `walk_expr` function that matches on the enum and dispatches to per-variant functions

**Handwritten walk files deleted:** `walk_expr.rs`, `walk_expr2.rs`, `walk_pattern.rs`, `walk_type.rs` are deleted. The `walk_dispatch_id!` and `walk_dispatch_id_slice!` macros are deleted.

**Remaining handwritten walk code:** `walk/mod.rs` retains `walk_program`, `dispatch_stmt`, `walk_stmt`, `dispatch_expr` (the entry points that call into generated code), `walk_binding`, and `dispatch_children`. These are ~100 lines of orchestration logic that doesn't benefit from generation.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/visitor/mod.rs` | Remove re-exports of PatternVisitor, TypeVisitor |
| `crates/lx/src/visitor/visitor_trait.rs` | Absorb all methods from PatternVisitor and TypeVisitor |
| `crates/lx/src/visitor/visitor_pattern_hooks.rs` | Delete |
| `crates/lx/src/visitor/visitor_type_hooks.rs` | Delete |
| `crates/lx/src/visitor/walk/mod.rs` | Remove dispatch macros, remove re-exports of deleted walk files, keep entry point functions |
| `crates/lx/src/visitor/walk/walk_expr.rs` | Delete |
| `crates/lx/src/visitor/walk/walk_expr2.rs` | Delete |
| `crates/lx/src/visitor/walk/walk_pattern.rs` | Delete |
| `crates/lx/src/visitor/walk/walk_type.rs` | Delete |
| `crates/lx/src/ast/mod.rs` | Add `#[walk(visit/leave)]` attributes to Expr, Pattern, TypeExpr variants |
| `crates/lx-macros/src/walk_enum.rs` | Generate dispatch and walk functions from visit/leave attributes |
| `crates/lx-macros/src/lib.rs` | No structural change — AstWalk derive still the entry point |
| `crates/lx/src/checker/capture.rs` | Move `visit_pattern_bind` and `visit_pattern_list` from `impl PatternVisitor` into `impl AstVisitor for FreeVarCollector`; delete both `impl PatternVisitor` and `impl TypeVisitor` blocks |
| `crates/lx/src/folder/validate_core.rs` | Delete empty `impl PatternVisitor` and `impl TypeVisitor` blocks |
| `crates/lx/src/stdlib/diag/diag_walk.rs` | Delete empty `impl PatternVisitor` and `impl TypeVisitor` blocks |
| `crates/lx/src/linter/rules/unused_import.rs` | Delete empty `impl PatternVisitor` and `impl TypeVisitor` blocks |
| `crates/lx/src/linter/rules/empty_match.rs` | Delete empty `impl PatternVisitor` and `impl TypeVisitor` blocks |
| `crates/lx/src/linter/rules/redundant_propagate.rs` | Delete empty `impl PatternVisitor` and `impl TypeVisitor` blocks |
| `crates/lx/src/linter/rules/single_branch_par.rs` | Delete empty `impl PatternVisitor` and `impl TypeVisitor` blocks |
| `crates/lx/src/linter/rules/duplicate_record_field.rs` | Delete empty `impl PatternVisitor` and `impl TypeVisitor` blocks |
| `crates/lx/src/linter/rules/break_outside_loop.rs` | Delete empty `impl PatternVisitor` and `impl TypeVisitor` blocks |
| `crates/lx/src/linter/rules/unreachable_code.rs` | Delete empty `impl PatternVisitor` and `impl TypeVisitor` blocks (if file exists) |

# Task List

### Task 1: Merge PatternVisitor into AstVisitor

In `crates/lx/src/visitor/visitor_trait.rs`:

Remove the supertrait bound `PatternVisitor` from `AstVisitor`:

From:
```rust
pub trait AstVisitor: PatternVisitor + TypeVisitor {
```

To (intermediate step — TypeVisitor removed in next task):
```rust
pub trait AstVisitor: TypeVisitor {
```

Copy all method definitions from `visitor_pattern_hooks.rs` (`PatternVisitor` trait body) into the `AstVisitor` trait body. These are:
- `visit_pattern`, `leave_pattern`
- `visit_pattern_literal`, `visit_pattern_bind`, `visit_pattern_wildcard`
- `visit_pattern_tuple`, `leave_pattern_tuple`
- `visit_pattern_list`, `leave_pattern_list`
- `visit_pattern_record`, `leave_pattern_record`
- `visit_pattern_constructor`, `leave_pattern_constructor`

Delete `crates/lx/src/visitor/visitor_pattern_hooks.rs`.

In `crates/lx/src/visitor/mod.rs`, remove the `mod visitor_pattern_hooks;` declaration and `pub use visitor_pattern_hooks::*;` re-export.

Delete ALL `impl PatternVisitor for X` blocks. There are exactly 10:

Empty impls (delete entirely):
- `crates/lx/src/folder/validate_core.rs`
- `crates/lx/src/stdlib/diag/diag_walk.rs`
- `crates/lx/src/linter/rules/unused_import.rs`
- `crates/lx/src/linter/rules/empty_match.rs`
- `crates/lx/src/linter/rules/redundant_propagate.rs`
- `crates/lx/src/linter/rules/single_branch_par.rs`
- `crates/lx/src/linter/rules/duplicate_record_field.rs`
- `crates/lx/src/linter/rules/break_outside_loop.rs`
- `crates/lx/src/linter/rules/unreachable_code.rs` (if it exists)

Has overrides (move methods into `impl AstVisitor`):
- `crates/lx/src/checker/capture.rs` — overrides `visit_pattern_bind` and `visit_pattern_list`. Move both methods into the existing `impl AstVisitor for FreeVarCollector` block in the same file.

### Task 2: Merge TypeVisitor into AstVisitor

Same process as Task 1 for TypeVisitor.

In `crates/lx/src/visitor/visitor_trait.rs`:

Remove the supertrait bound `TypeVisitor`:

From:
```rust
pub trait AstVisitor: TypeVisitor {
```

To:
```rust
pub trait AstVisitor {
```

Copy all method definitions from `visitor_type_hooks.rs` (`TypeVisitor` trait body) into `AstVisitor`:
- `visit_type_expr`, `leave_type_expr`
- `visit_type_named`, `visit_type_var`
- `visit_type_applied`, `leave_type_applied`
- `visit_type_list`, `leave_type_list`
- `visit_type_map`, `leave_type_map`
- `visit_type_record`, `leave_type_record`
- `visit_type_tuple`, `leave_type_tuple`
- `visit_type_func`, `leave_type_func`
- `visit_type_fallible`, `leave_type_fallible`

Delete `crates/lx/src/visitor/visitor_type_hooks.rs`.

In `crates/lx/src/visitor/mod.rs`, remove the `mod visitor_type_hooks;` declaration and `pub use visitor_type_hooks::*;` re-export.

Delete ALL `impl TypeVisitor for X` blocks. There are exactly 10, in the same files as PatternVisitor. ALL are empty (no overrides). Delete every one.

Update imports: any file that imports `PatternVisitor` or `TypeVisitor` directly — remove those imports (the methods are now on `AstVisitor`).

### Task 3: Add walk attribute support to the AstWalk derive macro

In `crates/lx-macros/src/walk_enum.rs`:

Add parsing for `#[walk(visit = "method_name", leave = "leave_name")]` attributes on enum variants. When present, the macro generates:

1. A dispatch function named `walk_{variant_snake}_dispatch` that:
   - Calls `v.{visit}(id, data, span)` (or the appropriate destructured form)
   - Matches on `VisitAction`: Stop breaks, Skip calls leave and continues, Descend calls the walk function

2. A walk function named `walk_{variant_snake}` that:
   - Calls `data.walk_children(v, arena)?` (using the already-generated walk_children)
   - Calls `v.{leave}(id, data, span)`
   - Returns `ControlFlow::Continue(())`

The attribute format:
```rust
#[walk(visit = "visit_binary", leave = "leave_binary")]
Binary(ExprBinary),
```

For variants with no leave hook (leaf nodes), use:
```rust
#[walk(visit = "visit_ident")]
Ident(Sym),
```

For variants with slice-based visitor hooks (where the visitor method takes `&[T]` instead of `&StructType`), use:
```rust
#[walk(visit = "visit_block", leave = "leave_block", slice)]
Block(Vec<StmtId>),
```

The `slice` flag tells the macro to pass the inner Vec as a slice to the visitor methods instead of as a struct reference.

The generated functions should be placed in the type's `impl` block alongside `recurse_children`, `children`, and `walk_children`. They should be `pub(crate)` to match the current visibility of the handwritten functions.

Additionally, generate a top-level `walk_{enum_snake}` function that matches on the enum and dispatches to the per-variant functions. For `Expr`, this replaces the handwritten `walk_expr` in `walk/mod.rs`. The generated function must call `v.leave_expr(id, expr, span)` after dispatching (matching current behavior in `walk_expr` line 218).

### Task 4: Add walk attributes to Expr variants

In `crates/lx/src/ast/mod.rs`, add `#[walk(visit, leave)]` attributes to each `Expr` variant:

```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum Expr {
    #[walk(visit = "visit_literal", leave = "leave_literal")]
    Literal(Literal),
    #[walk(visit = "visit_ident")]
    Ident(Sym),
    #[walk(visit = "visit_type_constructor")]
    TypeConstructor(Sym),

    #[walk(visit = "visit_binary", leave = "leave_binary")]
    Binary(ExprBinary),
    #[walk(visit = "visit_unary", leave = "leave_unary")]
    Unary(ExprUnary),
    #[walk(visit = "visit_pipe", leave = "leave_pipe")]
    Pipe(ExprPipe),

    #[walk(visit = "visit_apply", leave = "leave_apply")]
    Apply(ExprApply),
    #[walk(visit = "visit_section", leave = "leave_section")]
    Section(Section),

    #[walk(visit = "visit_field_access", leave = "leave_field_access")]
    FieldAccess(ExprFieldAccess),

    #[walk(visit = "visit_block", leave = "leave_block", slice)]
    Block(Vec<StmtId>),
    #[walk(visit = "visit_tuple", leave = "leave_tuple", slice)]
    Tuple(Vec<ExprId>),

    #[walk(visit = "visit_list", leave = "leave_list", slice)]
    List(Vec<ListElem>),
    #[walk(visit = "visit_record", leave = "leave_record", slice)]
    Record(Vec<RecordField>),
    #[walk(visit = "visit_map", leave = "leave_map", slice)]
    Map(Vec<MapEntry>),

    #[walk(visit = "visit_func", leave = "leave_func")]
    Func(ExprFunc),
    #[walk(visit = "visit_match", leave = "leave_match")]
    Match(ExprMatch),
    #[walk(visit = "visit_ternary", leave = "leave_ternary")]
    Ternary(ExprTernary),

    #[walk(visit = "visit_propagate", leave = "leave_propagate")]
    Propagate(ExprId),
    #[walk(visit = "visit_coalesce", leave = "leave_coalesce")]
    Coalesce(ExprCoalesce),

    #[walk(visit = "visit_slice", leave = "leave_slice")]
    Slice(ExprSlice),
    #[walk(visit = "visit_named_arg", leave = "leave_named_arg")]
    NamedArg(ExprNamedArg),

    #[walk(visit = "visit_loop", leave = "leave_loop", slice)]
    Loop(Vec<StmtId>),
    #[walk(visit = "visit_break", leave = "leave_break")]
    Break(Option<ExprId>),
    #[walk(visit = "visit_assert", leave = "leave_assert")]
    Assert(ExprAssert),

    #[walk(visit = "visit_par", leave = "leave_par", slice)]
    Par(Vec<StmtId>),
    #[walk(visit = "visit_sel", leave = "leave_sel", slice)]
    Sel(Vec<SelArm>),
    #[walk(visit = "visit_timeout", leave = "leave_timeout")]
    Timeout(ExprTimeout),

    #[walk(visit = "visit_emit", leave = "leave_emit")]
    Emit(ExprEmit),
    #[walk(visit = "visit_yield", leave = "leave_yield")]
    Yield(ExprYield),
    #[walk(visit = "visit_with", leave = "leave_with")]
    With(ExprWith),
}
```

Note: variants with `#[walk(skip)]` (if AST_ERROR_RECOVERY_NODES adds `Invalid`) should stay as `#[walk(skip)]`.

### Task 5: Add walk attributes to Pattern variants

In `crates/lx/src/ast/types.rs`, add attributes to `Pattern` variants:

```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum Pattern {
    #[walk(visit = "visit_pattern_literal")]
    Literal(Literal),
    #[walk(visit = "visit_pattern_bind")]
    Bind(Sym),
    #[walk(visit = "visit_pattern_wildcard")]
    Wildcard,
    #[walk(visit = "visit_pattern_tuple", leave = "leave_pattern_tuple", slice)]
    Tuple(Vec<PatternId>),
    #[walk(visit = "visit_pattern_list", leave = "leave_pattern_list")]
    List(PatternList),
    #[walk(visit = "visit_pattern_record", leave = "leave_pattern_record")]
    Record(PatternRecord),
    #[walk(visit = "visit_pattern_constructor", leave = "leave_pattern_constructor")]
    Constructor(PatternConstructor),
}
```

Normalize pattern visitor hook signatures to take struct references. Change:
- `visit_pattern_list(&mut self, id: PatternId, elems: &[PatternId], rest: Option<Sym>, span: SourceSpan)` → `visit_pattern_list(&mut self, id: PatternId, list: &PatternList, span: SourceSpan)`
- `leave_pattern_list` — same change
- `visit_pattern_record(&mut self, id: PatternId, fields: &[FieldPattern], rest: Option<Sym>, span: SourceSpan)` → `visit_pattern_record(&mut self, id: PatternId, record: &PatternRecord, span: SourceSpan)`
- `leave_pattern_record` — same change
- `visit_pattern_constructor(&mut self, id: PatternId, name: Sym, args: &[PatternId], span: SourceSpan)` → `visit_pattern_constructor(&mut self, id: PatternId, ctor: &PatternConstructor, span: SourceSpan)`
- `leave_pattern_constructor` — same change

Update the one consumer with overrides: `capture.rs` `visit_pattern_list` currently accesses `rest` directly — change to `list.rest`. `visit_pattern_bind` takes `Sym` directly — this signature stays unchanged since `Bind(Sym)` has no wrapper struct.

### Task 6: Add walk attributes to TypeExpr variants

In `crates/lx/src/ast/types.rs`, add attributes to `TypeExpr` variants:

```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum TypeExpr {
    #[walk(visit = "visit_type_named")]
    Named(Sym),
    #[walk(visit = "visit_type_var")]
    Var(Sym),
    #[walk(visit = "visit_type_applied", leave = "leave_type_applied")]
    Applied(TypeExprApplied),
    #[walk(visit = "visit_type_list", leave = "leave_type_list")]
    List(TypeExprId),
    #[walk(visit = "visit_type_map", leave = "leave_type_map")]
    Map(TypeExprMap),
    #[walk(visit = "visit_type_record", leave = "leave_type_record", slice)]
    Record(Vec<TypeField>),
    #[walk(visit = "visit_type_tuple", leave = "leave_type_tuple", slice)]
    Tuple(Vec<TypeExprId>),
    #[walk(visit = "visit_type_func", leave = "leave_type_func")]
    Func(TypeExprFunc),
    #[walk(visit = "visit_type_fallible", leave = "leave_type_fallible")]
    Fallible(TypeExprFallible),
}
```

Normalize TypeExpr visitor hook signatures. For named-field variants, wrap in structs so the macro can generate uniform dispatch:

Add to `crates/lx/src/ast/types.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, AstWalk)]
pub struct TypeExprMap {
    pub key: TypeExprId,
    pub value: TypeExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, AstWalk)]
pub struct TypeExprFunc {
    pub param: TypeExprId,
    pub ret: TypeExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, AstWalk)]
pub struct TypeExprFallible {
    pub ok: TypeExprId,
    pub err: TypeExprId,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct TypeExprApplied {
    pub name: Sym,
    pub args: Vec<TypeExprId>,
}
```

Change TypeExpr enum variants:
- `Map { key: TypeExprId, value: TypeExprId }` → `Map(TypeExprMap)`
- `Func { param: TypeExprId, ret: TypeExprId }` → `Func(TypeExprFunc)`
- `Fallible { ok: TypeExprId, err: TypeExprId }` → `Fallible(TypeExprFallible)`
- `Applied(Sym, Vec<TypeExprId>)` → `Applied(TypeExprApplied)`

Change visitor hook signatures:
- `visit_type_map(id, key, value, span)` → `visit_type_map(id, &TypeExprMap, span)`
- `visit_type_func(id, param, ret, span)` → `visit_type_func(id, &TypeExprFunc, span)`
- `visit_type_fallible(id, ok, err, span)` → `visit_type_fallible(id, &TypeExprFallible, span)`
- `visit_type_applied(id, name, &[TypeExprId], span)` → `visit_type_applied(id, &TypeExprApplied, span)`
- Same for all corresponding leave methods.

No TypeVisitor consumer has overrides, so only the trait definitions and walk code need updating (and the walk code is being deleted anyway).

### Task 7: Implement macro code generation for dispatch and walk functions

In `crates/lx-macros/src/walk_enum.rs`:

Parse the `#[walk(visit = "...", leave = "...", slice)]` attribute. Extract:
- `visit_method`: the visitor method name (required)
- `leave_method`: the leave method name (optional — if absent, no leave call)
- `slice`: flag for slice-based dispatch (optional)

For each variant with a `visit` attribute, generate two functions in the impl block:

**Dispatch function** (`{variant_snake}_dispatch`):
```rust
pub(crate) fn binary_dispatch<V: crate::visitor::AstVisitor + ?Sized>(
    v: &mut V, id: ExprId, data: &ExprBinary, span: miette::SourceSpan, arena: &crate::ast::AstArena
) -> std::ops::ControlFlow<()> {
    let action = v.visit_binary(id, data, span);
    match action {
        crate::visitor::VisitAction::Stop => std::ops::ControlFlow::Break(()),
        crate::visitor::VisitAction::Skip => {
            v.leave_binary(id, data, span);
            std::ops::ControlFlow::Continue(())
        },
        crate::visitor::VisitAction::Descend => Self::binary_walk(v, id, data, span, arena),
    }
}
```

**Walk function** (`{variant_snake}_walk`):
```rust
pub(crate) fn binary_walk<V: crate::visitor::AstVisitor + ?Sized>(
    v: &mut V, id: ExprId, data: &ExprBinary, span: miette::SourceSpan, arena: &crate::ast::AstArena
) -> std::ops::ControlFlow<()> {
    data.walk_children(v, arena)?;
    v.leave_binary(id, data, span);
    std::ops::ControlFlow::Continue(())
}
```

For `slice` variants, the visitor methods receive `&[T]` instead of `&StructType`. The dispatch function destructures the variant and passes the inner vec as a slice.

For single-field ID variants like `Propagate(ExprId)`, generate inline dispatch that calls `dispatch_expr` on the inner ID.

For variants with no leave hook, omit the leave call in both dispatch and walk functions.

Also generate a `dispatch_variant<V>(v, id, arena)` method on the enum that matches on `self` and calls the appropriate per-variant dispatch function. This replaces the handwritten `walk_expr` function.

### Task 8: Delete handwritten walk files and macros

Delete:
- `crates/lx/src/visitor/walk/walk_expr.rs`
- `crates/lx/src/visitor/walk/walk_expr2.rs`
- `crates/lx/src/visitor/walk/walk_pattern.rs`
- `crates/lx/src/visitor/walk/walk_type.rs`

In `crates/lx/src/visitor/walk/mod.rs`:

Remove:
- `mod walk_expr;` and `pub use walk_expr::*;`
- `mod walk_expr2;` and `pub use walk_expr2::*;`
- `mod walk_pattern;` and `pub use walk_pattern::*;`
- `mod walk_type;` and `pub use walk_type::*;`
- The `walk_dispatch_id!` macro definition (lines 8-22)
- The `walk_dispatch_id_slice!` macro definition (lines 24-38)
- The three `walk_dispatch_id!` invocations for trait_decl, class_decl, field_update (lines 50-52)

Update `walk_expr` to call the generated `Expr::dispatch_variant` method (or whatever the generated dispatch entry point is called). The function body changes from a 40-arm match to:

```rust
pub fn walk_expr<V: AstVisitor + ?Sized>(v: &mut V, id: ExprId, arena: &AstArena) -> ControlFlow<()> {
    let expr = arena.expr(id);
    let span = arena.expr_span(id);
    expr.dispatch_variant(v, id, span, arena)?;
    v.leave_expr(id, expr, span);
    ControlFlow::Continue(())
}
```

Similarly, update pattern and type expression dispatch to use generated methods:

```rust
pub(crate) fn walk_pattern_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: PatternId, arena: &AstArena) -> ControlFlow<()> {
    let pattern = arena.pattern(id);
    let span = arena.pattern_span(id);
    let action = v.visit_pattern(id, pattern, span);
    match action {
        VisitAction::Stop => ControlFlow::Break(()),
        VisitAction::Skip => { v.leave_pattern(id, pattern, span); ControlFlow::Continue(()) },
        VisitAction::Descend => {
            pattern.dispatch_variant(v, id, span, arena)?;
            v.leave_pattern(id, pattern, span);
            ControlFlow::Continue(())
        },
    }
}
```

The `dispatch_children`, `walk_program`, `dispatch_stmt`, `walk_stmt`, `dispatch_expr`, `walk_binding`, `walk_trait_decl`, `walk_class_decl`, and `walk_field_update` functions remain in `walk/mod.rs` as handwritten orchestration. These contain logic that doesn't follow a per-variant pattern.

### Task 9: Update walk/mod.rs for remaining handwritten dispatch

After deleting the walk files, `walk/mod.rs` needs to handle the stmt-level dispatch that was previously using the deleted macros.

For `walk_trait_decl`, `walk_class_decl`, and `walk_field_update`: these were using `walk_dispatch_id!` which is now deleted. Inline the dispatch pattern directly:

```rust
fn walk_trait_decl_dispatch<V: AstVisitor + ?Sized>(v: &mut V, id: StmtId, data: &TraitDeclData, span: SourceSpan, arena: &AstArena) -> ControlFlow<()> {
    let action = v.visit_trait_decl(id, data, span);
    match action {
        VisitAction::Stop => ControlFlow::Break(()),
        VisitAction::Skip => {
            v.leave_trait_decl(id, data, span);
            ControlFlow::Continue(())
        },
        VisitAction::Descend => {
            data.walk_children(v, arena)?;
            v.leave_trait_decl(id, data, span);
            ControlFlow::Continue(())
        },
    }
}
```

These three functions stay handwritten because they dispatch at the Stmt level (not per-variant on an enum — `TraitDecl` is a variant of `Stmt`, and `Stmt` dispatch is handled by `walk_stmt` which is already handwritten).

### Task 10: Compile, format, and verify

Run `just fmt` to format all changed files.

Run `just diagnose`. Common issues:
- Generated dispatch functions may have naming conflicts — resolve by adjusting the naming convention in the macro
- Visitor method signatures may not match what the macro expects — adjust either the macro or the signatures
- Missing imports in generated code — the macro must emit fully qualified paths

Run `just test` to verify all existing tests pass.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Do not add, skip, reorder, or combine tasks.**
3. **This depends on UNIVERSAL_ASTWALK_DERIVATION being complete.** Do not start until that work item is done.
4. **The macro generates `pub(crate)` functions in an `impl` block.** These are not freestanding functions — they're methods on the enum type. Callers use `Expr::binary_dispatch(v, id, data, span, arena)` or `expr.dispatch_variant(v, id, span, arena)`.
5. **Visitor hook signatures vary.** Some take struct references, some take slices, some take raw IDs. The macro must handle all shapes. The `slice` attribute flag distinguishes slice-based hooks.
6. **Pattern and type visitor hooks are normalized to struct references** (e.g., `visit_pattern_list(id, &PatternList, span)` not `visit_pattern_list(id, elems, rest, span)`). Task 5 normalizes pattern hooks, Task 6 normalizes type hooks. The one consumer with overrides (`capture.rs`) must be updated to access fields through the struct.
7. **Test the macro incrementally.** Start with one variant (e.g., `Binary`), verify it compiles and the generated dispatch works, then proceed to the rest.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/VISITOR_WALK_CONSOLIDATION.md" })
```

Then call `next_task` to begin.
