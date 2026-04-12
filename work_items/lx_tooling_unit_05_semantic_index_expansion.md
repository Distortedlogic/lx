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
- `Scope`
- `DefinitionInfo`
- `Reference`
- `Checker::record_type`
- `Checker::check_program`
- `Checker::check_stmt`
- `Checker::check_expr`
- `Checker::synth_expr`
- `free_vars`

## Mechanical Task List
1. In `crates/lx-checker/src/semantic.rs`, add one `NodeId`-based parent index that covers every AST node kind already represented by `lx_ast::ast::NodeId`.
2. Add one `NodeId`-based scope index that maps every indexed node to its owning `ScopeId`.
3. Add a mutation index keyed by `DefinitionId`. Record both reassignment sites and field-update sites.
4. Add a compact control-context record keyed by expression ID. It must at least answer:
   - inside function?
   - inside loop?
   - inside par?
   - inside with?
5. Add public query methods on `SemanticModel` for every new index instead of exposing raw maps directly.
6. Extend `SemanticModelBuilder` with explicit record methods for:
   - entering a `NodeId` under a parent `NodeId`
   - assigning a `NodeId` to the current scope
   - recording a mutation site for a definition
   - recording control context for an expression
7. Populate the new indexes during the checker’s existing traversal instead of creating rule-local recursion later.
8. Update `check_expr.rs`, `visit_stmt.rs`, `type_ops.rs`, `synth_control.rs`, `synth_compound.rs`, and `infer_pattern.rs` so they record scope and parent information at the points where the checker already knows the owning node and current scope.
9. Where current checker structure makes a parent or scope assignment hard to record, add small helper functions in `semantic.rs` or `lib.rs`. Do not spread ad-hoc map writes throughout unrelated modules.
10. In `crates/lx-checker/src/capture.rs`, change `free_vars` to consume the semantic model in addition to the arena, and replace manual scope bookkeeping with the new parent/scope queries. Update the call site in `crates/lx-checker/src/synth_compound.rs` in the same unit. Keep the returned free-variable set behavior unchanged.
11. Add regression coverage that proves the new queries are populated correctly for:
    - nested function scopes
    - match-arm scopes
    - loop/par/with control contexts
    - pattern bindings
    - reassignment and field update mutation sites
12. Keep the existing public APIs working:
    - `type_of_expr`
    - `type_of_def`
    - `display_type`
    - `references_to`
13. Do not move lint-rule logic into this unit. Only expose the query substrate they will need next.
14. Do not add any CST or formatter-trivia data to the semantic model.

## Verification
1. Run `cargo test -p lx-checker --test checker_regressions`.
2. Add assertions in checker tests that exercise the new query methods directly.
3. Run `cargo test -p lx-linter --test pipeline_regressions` to confirm the semantic model still supports current lints.
4. Run `just test`.

## Out Of Scope
- Lint framework redesign
- Checker pass splitting
- CST or lossless syntax work
