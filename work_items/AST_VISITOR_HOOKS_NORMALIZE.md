# Goal

Normalize the AstVisitor trait: fix inconsistent naming (`on_` vs `visit_` prefixes), inline macro-hidden hook methods, fix the broken `visit_binding` default, and add `StmtId` to statement-specific hooks so they match the convention used by expression hooks.

# Why

- `on_stmt`/`on_expr` use the `on_` prefix while every other hook uses `visit_` — confusing whether they behave differently (they do not)
- `visit_binding` defaults to calling `walk_binding` internally and returning `VisitAction::Skip`, unlike every other hook which returns `Descend`. Overriding it with `Descend` causes double-walking of children
- `hooks_pattern.rs` and `hooks_type.rs` inject ~60 methods into the trait via macro — hiding them from IDE navigation, docs, and code search
- Statement-specific hooks (`visit_binding`, `visit_trait_decl`, etc.) omit the `StmtId`, unlike expression hooks which all receive `ExprId`. This prevents correlating stmt-level hooks with the statement arena

# What changes

1. Inline all methods from `hooks_pattern::pattern_visitor_hooks!()` and `hooks_type::type_visitor_hooks!()` directly into the `AstVisitor` trait body, then delete the two macro files
2. Rename `on_stmt` → `visit_stmt` and `on_expr` → `visit_expr` throughout the trait, walk dispatch, and all implementors
3. Change `visit_binding` default from `{ walk_binding + Skip }` to just `{ VisitAction::Descend }`
4. Add `StmtId` as first parameter to: `visit_binding`, `leave_binding`, `visit_type_def`, `visit_trait_decl`, `leave_trait_decl`, `visit_class_decl`, `leave_class_decl`, `visit_trait_union`, `visit_field_update`, `leave_field_update`, `visit_use`

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/visitor/mod.rs` | Inline macro hooks, rename on_stmt/on_expr, fix visit_binding default, add StmtId to stmt hooks |
| `crates/lx/src/visitor/hooks_pattern.rs` | DELETE |
| `crates/lx/src/visitor/hooks_type.rs` | DELETE |
| `crates/lx/src/visitor/walk/mod.rs` | Update dispatch_stmt and walk_stmt to call visit_stmt instead of on_stmt, pass StmtId to stmt-specific hooks |
| `crates/lx/src/visitor/walk/walk_expr.rs` | No changes (expr hooks already use visit_ prefix) |
| `crates/lx/src/linter/runner.rs` | Rename on_expr → visit_expr, on_stmt → visit_stmt in AstVisitor impl |

# Task List

### Task 1: Inline pattern visitor hooks into AstVisitor trait

In `crates/lx/src/visitor/mod.rs`, replace the line `hooks_pattern::pattern_visitor_hooks!();` (line 255) with the full method definitions currently inside the macro in `hooks_pattern.rs`. Copy the 14 method signatures verbatim from `hooks_pattern.rs` lines 3-41 (visit_pattern, leave_pattern, visit_pattern_literal, visit_pattern_bind, visit_pattern_wildcard, visit_pattern_tuple, leave_pattern_tuple, visit_pattern_list, leave_pattern_list, visit_pattern_record, leave_pattern_record, visit_pattern_constructor, leave_pattern_constructor). All methods keep their default implementations. Remove the `hooks_pattern` module declaration (`mod hooks_pattern;` near the top of the file).

### Task 2: Inline type visitor hooks into AstVisitor trait

In `crates/lx/src/visitor/mod.rs`, replace the line `hooks_type::type_visitor_hooks!();` (line 256, now shifted after Task 1 inlining) with the full method definitions from `hooks_type.rs` lines 3-56. These are: visit_type_expr, leave_type_expr, visit_type_named, visit_type_var, visit_type_applied, leave_type_applied, visit_type_list, leave_type_list, visit_type_map, leave_type_map, visit_type_record, leave_type_record, visit_type_tuple, leave_type_tuple, visit_type_func, leave_type_func, visit_type_fallible, leave_type_fallible. All keep default implementations. Remove the `hooks_type` module declaration.

### Task 3: Delete macro files

Delete `crates/lx/src/visitor/hooks_pattern.rs` and `crates/lx/src/visitor/hooks_type.rs`.

### Task 4: Rename on_stmt and on_expr to visit_stmt and visit_expr

In `crates/lx/src/visitor/mod.rs`:
- Rename `fn on_stmt(` to `fn visit_stmt(`
- Rename `fn on_expr(` to `fn visit_expr(`

In `crates/lx/src/visitor/walk/mod.rs`:
- In `dispatch_stmt`, change `v.on_stmt(id, stmt, span, arena)` to `v.visit_stmt(id, stmt, span, arena)`
- In `dispatch_expr`, change `v.on_expr(id, expr, span, arena)` to `v.visit_expr(id, expr, span, arena)`

In `crates/lx/src/linter/runner.rs`:
- Rename `fn on_expr(` to `fn visit_expr(` in the `AstVisitor for LintRunner` impl
- Rename `fn on_stmt(` to `fn visit_stmt(` in the same impl

Search the entire `crates/lx/src/` tree for any other references to `on_stmt` or `on_expr` and update them. Use `rg --type rust 'on_stmt|on_expr' crates/lx/src/` to find all occurrences.

### Task 5: Fix visit_binding default implementation

In `crates/lx/src/visitor/mod.rs`, change the `visit_binding` method default from:

```rust
fn visit_binding(&mut self, binding: &Binding, span: SourceSpan, arena: &AstArena) -> VisitAction {
    match walk_binding(self, binding, span, arena) {
        ControlFlow::Continue(()) => VisitAction::Skip,
        ControlFlow::Break(()) => VisitAction::Stop,
    }
}
```

to:

```rust
fn visit_binding(&mut self, _id: StmtId, _binding: &Binding, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    VisitAction::Descend
}
```

This aligns it with every other hook's contract: return Descend to let the walker handle child traversal.

In `crates/lx/src/visitor/walk/mod.rs`, update the `Stmt::Binding` branch in `walk_stmt` to properly call `walk_binding` when the action is `Descend`, matching the pattern used for other stmt variants. Currently `walk_stmt` calls `v.visit_binding(binding, span, arena)` and only walks if Descend — this is correct, but verify the walk_binding call passes the StmtId (added in Task 6).

### Task 6: Add StmtId to all statement-specific hooks

In `crates/lx/src/visitor/mod.rs`, add `id: StmtId` as the first parameter after `&mut self` to all of these methods (both visit and leave variants):

- `visit_binding(&mut self, id: StmtId, binding: &Binding, span: SourceSpan, arena: &AstArena) -> VisitAction`
- `leave_binding(&mut self, _id: StmtId, _binding: &Binding, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()>`
- `visit_type_def(&mut self, _id: StmtId, _def: &StmtTypeDef, _span: SourceSpan, _arena: &AstArena) -> VisitAction`
- `visit_trait_decl(&mut self, _id: StmtId, _data: &TraitDeclData, _span: SourceSpan, _arena: &AstArena) -> VisitAction`
- `leave_trait_decl(&mut self, _id: StmtId, _data: &TraitDeclData, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()>`
- `visit_class_decl(&mut self, _id: StmtId, _data: &ClassDeclData, _span: SourceSpan, _arena: &AstArena) -> VisitAction`
- `leave_class_decl(&mut self, _id: StmtId, _data: &ClassDeclData, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()>`
- `visit_trait_union(&mut self, _id: StmtId, _def: &TraitUnionDef, _span: SourceSpan, _arena: &AstArena) -> VisitAction`
- `visit_field_update(&mut self, _id: StmtId, _update: &StmtFieldUpdate, _span: SourceSpan, _arena: &AstArena) -> VisitAction`
- `leave_field_update(&mut self, _id: StmtId, _update: &StmtFieldUpdate, _span: SourceSpan, _arena: &AstArena) -> ControlFlow<()>`
- `visit_use(&mut self, _id: StmtId, _stmt: &UseStmt, _span: SourceSpan, _arena: &AstArena) -> VisitAction`

In `crates/lx/src/visitor/walk/mod.rs`:

**Convert the three `walk_dispatch!` macro invocations to `walk_dispatch_id!` with `StmtId` as the `$id_ty` parameter.** Currently lines 57-59 read:
```
walk_dispatch!(walk_trait_decl_dispatch, walk_trait_decl, visit_trait_decl, leave_trait_decl, TraitDeclData);
walk_dispatch!(walk_class_decl_dispatch, walk_class_decl, visit_class_decl, leave_class_decl, ClassDeclData);
walk_dispatch!(walk_field_update_dispatch, walk_field_update, visit_field_update, leave_field_update, StmtFieldUpdate);
```
Change them to:
```
walk_dispatch_id!(walk_trait_decl_dispatch, walk_trait_decl, visit_trait_decl, leave_trait_decl, TraitDeclData, StmtId);
walk_dispatch_id!(walk_class_decl_dispatch, walk_class_decl, visit_class_decl, leave_class_decl, ClassDeclData, StmtId);
walk_dispatch_id!(walk_field_update_dispatch, walk_field_update, visit_field_update, leave_field_update, StmtFieldUpdate, StmtId);
```

**Update `walk_stmt`** to pass `id` to all stmt-specific dispatch calls. The `id: StmtId` is already in scope. Change:
- `walk_trait_decl_dispatch(v, data, span, arena)` → `walk_trait_decl_dispatch(v, id, data, span, arena)`
- `walk_class_decl_dispatch(v, data, span, arena)` → `walk_class_decl_dispatch(v, id, data, span, arena)`
- `walk_field_update_dispatch(v, fu, span, arena)` → `walk_field_update_dispatch(v, id, fu, span, arena)`
- `v.visit_binding(binding, span, arena)` → `v.visit_binding(id, binding, span, arena)`
- `v.visit_type_def(def, span, arena)` → `v.visit_type_def(id, def, span, arena)`
- `v.visit_trait_union(def, span, arena)` → `v.visit_trait_union(id, def, span, arena)`
- `v.visit_use(use_stmt, span, arena)` → `v.visit_use(id, use_stmt, span, arena)`

**Update `walk_binding`** signature to accept `id: StmtId` as first param after `v`, and pass it to `v.leave_binding(id, binding, span, arena)`.

**Update `walk_trait_decl`, `walk_class_decl`, `walk_field_update`** to accept `id: StmtId` as second parameter (after `v`) and pass it to their leave hooks.

**No changes needed in LintRunner** — it does not override any stmt-specific hooks (visit_binding, visit_type_def, visit_trait_decl, visit_class_decl, visit_trait_union, visit_field_update, visit_use). It only overrides visit_stmt, visit_expr, leave_expr, and visit_pattern, none of which change signature in this task.

### Task 7: Format and commit

Run `just fmt` then `git add -A && git commit -m "refactor: normalize AstVisitor hooks — inline macros, fix naming, add StmtId params"`.

### Task 8: Verify

Run `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **Search comprehensively** after renaming — use `rg` to find every occurrence before committing.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AST_VISITOR_HOOKS_NORMALIZE.md" })
```

Then call `next_task` to begin.
