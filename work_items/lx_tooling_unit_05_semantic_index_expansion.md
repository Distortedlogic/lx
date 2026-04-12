---
unit: 5
title: Semantic Index Expansion
scope: lx-checker
depends_on: lx_tooling_unit_04_traversal_and_transform_ergonomics
optional: false
---

## Goal
Expand the semantic model so lints and checker-adjacent analyses can query parents, scopes, control context, and mutation sites directly instead of rebuilding that information through ad-hoc walks.

## Dependency Contract
Assume Units 01-04 are merged. This unit widens the semantic substrate only. Do not rewrite lint rules here; that belongs to Unit 06.

## Verified Preconditions
- `SemanticModel` in `crates/lx-checker/src/semantic.rs` currently stores:
  - `scopes`
  - `definitions`
  - `references`
  - `def_references`
  - `expr_types`
  - `type_defs`
  - `trait_fields`
  - `type_arena`
- There is currently no parent index for `ExprId`, `StmtId`, `PatternId`, or `TypeExprId`.
- There is currently no node-to-scope index beyond the transient `scope_stack` inside `SemanticModelBuilder`.
- There is currently no mutation index keyed by `DefinitionId`.
- Scope-sensitive logic is still reimplemented manually in multiple places:
  - `crates/lx-checker/src/capture.rs`
  - `crates/lx-linter/src/rules/mut_never_mutated.rs`
- Definitions and references are already recorded during checking:
  - `add_definition` is used from `visit_stmt.rs`, `check_expr.rs`, `synth_compound.rs`, `synth_control.rs`, and `infer_pattern.rs`
  - `add_reference` is used from `type_ops.rs`
- `free_vars` in `crates/lx-checker/src/capture.rs` is currently called during checking from `Checker::check_mutable_captures`, before `SemanticModelBuilder::build(...)` produces the final `SemanticModel`.

## Files To Create Or Change
- `crates/lx-checker/src/semantic.rs`
- `crates/lx-checker/src/lib.rs`
- `crates/lx-checker/src/check_expr.rs`
- `crates/lx-checker/src/visit_stmt.rs`
- `crates/lx-checker/src/type_ops.rs`
- `crates/lx-checker/src/synth_control.rs`
- `crates/lx-checker/src/synth_compound.rs`
- `crates/lx-checker/src/infer_pattern.rs`
- `crates/lx-checker/src/capture.rs`
- `crates/lx-checker/tests/checker_regressions.rs`
- `crates/lx-linter/tests/pipeline_regressions.rs`

## Exact Structs And Functions To Inspect Or Change
- `SemanticModel`
- `SemanticModelBuilder`
- `SemanticModelBuilder::build`
- `Scope`
- `DefinitionInfo`
- `Reference`
- `DefKind`
- `Checker::record_type`
- `Checker::check_program`
- `Checker::check_mutable_captures`
- `Checker::check_stmt`
- `Checker::check_expr`
- `Checker::synth_expr`
- `free_vars`

## Mechanical Task List
1. In `crates/lx-checker/src/semantic.rs`, add one `NodeId`-based parent index that covers every AST node kind already represented by `lx_ast::ast::NodeId`.
2. Add one `NodeId`-based scope index that maps every indexed node to its owning `ScopeId`.
3. Add a mutation index keyed by `DefinitionId`. Define `MutationSite` and `MutationKind` in `semantic.rs`:
   - `MutationKind::Reassign`
   - `MutationKind::FieldUpdate { fields: Vec<Sym> }`
   - `MutationSite { stmt_id: StmtId, span: SourceSpan, kind: MutationKind }`
   Record both reassignment sites and field-update sites.
4. Add a compact control-context record keyed by expression ID. It must at least answer:
   - inside function?
   - inside loop?
   - inside par?
   - inside with?
5. Add public query methods on `SemanticModel` for every new index instead of exposing raw maps directly. The minimum required surface for downstream units is:
   - `parent_of(node: NodeId) -> Option<NodeId>`
   - `scope_of(node: NodeId) -> Option<ScopeId>`
   - `definition(id: DefinitionId) -> &DefinitionInfo`
   - `control_context(expr: ExprId) -> Option<&ControlContext>`
   - `mutation_sites(def: DefinitionId) -> &[MutationSite]`
   - `import_definitions() -> Vec<DefinitionId>`
   `import_definitions()` must be implemented by filtering existing `definitions` for `DefKind::Import` in definition order. Do not add a separate import index for this method.
6. Extend `SemanticModelBuilder` with the same in-progress query surface needed while checking is still running. At minimum, add:
   - `parent_of(node: NodeId) -> Option<NodeId>`
   - `scope_of(node: NodeId) -> Option<ScopeId>`
   - `definition(id: DefinitionId) -> &DefinitionInfo`
   - `control_context(expr: ExprId) -> Option<&ControlContext>`
   - `mutation_sites(def: DefinitionId) -> &[MutationSite]`
   - `import_definitions() -> Vec<DefinitionId>`
   Back these queries with the same collections that `build(...)` later moves into `SemanticModel`; do not recompute them after checking finishes.
7. Extend `SemanticModelBuilder` with explicit record methods for:
   - entering a `NodeId` under a parent `NodeId`
   - assigning a `NodeId` to the current scope
   - recording a mutation site for a definition
   - recording control context for an expression
8. Populate the new indexes during the checker’s existing traversal instead of creating rule-local recursion later.
9. In `crates/lx-checker/src/visit_stmt.rs`, record mutation sites with these exact resolution rules:
   - for `BindTarget::Reassign(name)`, resolve the target with `self.sem.resolve_in_scope(*name)` at the statement point
   - nearest visible definition wins under shadowing because `resolve_in_scope` walks `scope_stack` from innermost to outermost
   - only record a mutation if the resolved definition kind is one of `Binding`, `FuncParam`, `PatternBind`, `WithBinding`, or `ResourceBinding`
   - if resolution returns `None`, or resolves to `Import`, `TypeDef`, `TraitDef`, or `ClassDef`, record no mutation and do not synthesize a placeholder definition
10. In the same file, apply the exact same name-resolution rules to `Stmt::FieldUpdate(fu)` before recording `MutationKind::FieldUpdate { fields: fu.fields.clone() }`.
11. Update `check_expr.rs`, `visit_stmt.rs`, `type_ops.rs`, `synth_control.rs`, `synth_compound.rs`, and `infer_pattern.rs` so they record scope and parent information at the points where the checker already knows the owning node and current scope.
12. Where current checker structure makes a parent or scope assignment hard to record, add small helper functions in `semantic.rs` or `lib.rs`. Do not spread ad-hoc map writes throughout unrelated modules.
13. In `crates/lx-checker/src/capture.rs`, change `free_vars` to consume builder-backed semantic queries while checking is in progress. Use this exact contract:
   - `free_vars(expr: ExprId, ctx: &CaptureContext<'_>) -> HashSet<Sym>` where `CaptureContext` wraps `&AstArena` and `&SemanticModelBuilder`
   Replace manual scope bookkeeping with `parent_of(...)`, `scope_of(...)`, and `definition(...)` queries from the builder-backed semantic indices. Update `Checker::check_mutable_captures` and the `synth_compound.rs` / `synth_control.rs` call paths in the same unit. Keep the returned free-variable set behavior unchanged.
14. Add regression coverage that proves the new queries are populated correctly for:
   - nested function scopes
   - match-arm scopes
   - loop/par/with control contexts
   - pattern bindings
   - reassignment and field update mutation sites
   - shadowed reassignments resolve to the innermost matching definition
   - unresolved `BindTarget::Reassign` and unresolved `Stmt::FieldUpdate` produce no mutation record
   - mutable-capture diagnostics in concurrent constructs still match Unit 01 behavior after `free_vars` moves onto builder-backed semantic queries
15. Keep the existing public APIs working:
   - `type_of_expr`
   - `type_of_def`
   - `display_type`
   - `references_to`
16. Do not move lint-rule logic into this unit. Only expose the query substrate they will need next.
17. Do not add any CST or formatter-trivia data to the semantic model.

## Verification
1. Run `cargo test -p lx-checker --test checker_regressions`.
2. Add assertions in checker tests that exercise the new query methods directly.
3. Run `cargo test -p lx-linter --test pipeline_regressions` to confirm the semantic model still supports current lints.
4. Run `just test`.

## Out Of Scope
- Lint framework redesign
- Checker pass splitting
- CST or lossless syntax work
