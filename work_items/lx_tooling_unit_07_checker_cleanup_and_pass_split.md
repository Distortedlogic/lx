---
unit: 7
title: Checker Cleanup and Pass Split
scope: lx-checker
depends_on: lx_tooling_unit_06_lint_and_fix_replatforming
optional: false
---

## Goal
Replace the current mixed visitor-driven checker flow with explicit passes whose responsibilities are clear: top-level collection, statement/body checking, expression inference/checking, and semantic finalization.

## Dependency Contract
Assume Units 01-06 are merged. This is the final structural cleanup unit. It may move code across modules, but it must not change language behavior except to remove duplicated internal logic and panic-shaped invariants already covered by earlier units.

## Verified Preconditions
- `Checker::check_program` in `crates/lx-checker/src/lib.rs` currently drives checking with `walk_program(self, program)` and `leave_stmt`.
- `impl AstVisitor for Checker` in the same file uses `visit_stmt -> Skip` and `leave_stmt -> check_stmt`, so statement checking is scheduled indirectly by visitor order.
- Statement handling is spread across:
  - `crates/lx-checker/src/visit_stmt.rs`
  - `crates/lx-checker/src/check_expr.rs`
  - `crates/lx-checker/src/type_ops.rs`
  - `crates/lx-checker/src/synth_control.rs`
  - `crates/lx-checker/src/synth_compound.rs`
- Scope push/pop logic for functions and match arms exists in both `check_expr.rs` and `synth_compound.rs`.
- Top-level import/type/trait/class handling is mixed into `check_stmt` instead of being an explicit collection pass.
- `ModuleSignature` extraction in `crates/lx-checker/src/module_graph.rs` depends on final semantic output.
- `check_binding` in `crates/lx-checker/src/visit_stmt.rs` currently introduces `BindTarget::Name` definitions only after RHS checking and introduces pattern bindings through `infer_pattern_bindings(...)` after the value type is known.

## Files To Create Or Change
- `crates/lx-checker/src/lib.rs`
- `crates/lx-checker/src/visit_stmt.rs`
- `crates/lx-checker/src/check_expr.rs`
- `crates/lx-checker/src/type_ops.rs`
- `crates/lx-checker/src/synth_control.rs`
- `crates/lx-checker/src/synth_compound.rs`
- `crates/lx-checker/src/infer_pattern.rs`
- `crates/lx-checker/src/module_graph.rs`
- `crates/lx-checker/src/capture.rs`
- `crates/lx-checker/src/pass_collect.rs`
- `crates/lx-checker/src/pass_body.rs`
- `crates/lx-checker/src/pass_finalize.rs`
- `crates/lx-checker/tests/checker_regressions.rs`

## Exact Structs, Enums, And Functions To Inspect Or Change
- `Checker`
- `Checker::new`
- `Checker::check_program`
- `Checker::check_stmt`
- `Checker::check_stmts`
- `Checker::check_expr`
- `Checker::synth_expr`
- `Checker::synth_func_type`
- `Checker::synth_match_type`
- `Checker::synth_with_type`
- `ModuleSignature`
- `extract_signature`

## Mechanical Task List
1. In `crates/lx-checker/src/lib.rs`, stop using `walk_program(self, program)` plus `leave_stmt` as the primary scheduling mechanism.
2. Create `crates/lx-checker/src/pass_collect.rs` and move only these top-level predeclaration steps there:
   - type definitions
   - type constructors
   - trait declarations
   - class declarations
   - imports
   `pass_collect` must not predeclare ordinary top-level `Stmt::Binding` targets. Do not create definitions there for:
   - `BindTarget::Name`
   - `BindTarget::Pattern`
   - `BindTarget::Reassign`
   This unit must preserve current visibility and order semantics rather than inventing a new forward-declaration rule for bindings.
3. Create `crates/lx-checker/src/pass_body.rs` and move the executable statement/body checking flow there. `pass_body` must execute statements in source order and preserve the current binding-introduction contract:
   - `check_stmts`
   - `check_stmt`
   - top-level expression statements
   - binding value checking
   - after RHS checking, introduce `BindTarget::Name` definitions exactly where `check_binding` does today
   - after value-type inference/checking, introduce pattern bindings exactly where `infer_pattern_bindings(...)` does today
   - keep `BindTarget::Reassign` as a lookup-and-unify operation against an existing definition rather than a declaration step
   The pass split must therefore preserve current forward-reference and self-reference behavior for ordinary top-level bindings.
4. Create `crates/lx-checker/src/pass_finalize.rs` and move final semantic assembly there. Keep `SemanticModelBuilder::build(...)` as the last step.
5. Update `crates/lx-checker/src/lib.rs` so `check_program` executes passes explicitly and in order:
   - collection
   - body checking
   - finalization
6. Leave expression-specific logic in `check_expr.rs`, `type_ops.rs`, `synth_control.rs`, and `synth_compound.rs`, but make those modules helpers called by the explicit body pass rather than indirect visitor callbacks.
7. Remove duplicate function-scope and match-arm logic by extracting shared helpers used by both “check against expected type” and “synthesize type” paths.
8. Make `infer_pattern.rs` own only pattern-type binding inference. It must not also schedule statement traversal.
9. In `capture.rs`, rewrite capture analysis to use the new semantic helpers from Unit 05. Remove recursive scope bookkeeping and compute free variables only from `parent_of(...)`, `scope_of(...)`, and `definition(...)` queries.
10. In `module_graph.rs`, update `extract_signature` to consume the finalized semantic output after `pass_finalize`, including exported binding types and trait fields, while preserving the observable `ModuleSignature` shape and public fields.
11. Delete any now-dead visitor glue in `lib.rs` after the explicit pass pipeline compiles and tests pass.
12. Keep diagnostics, semantic output, and exported-module signatures behaviorally stable against these exact Unit 01 baselines:
   - `crates/lx-checker/tests/checker_regressions.rs`
   - `crates/lx-checker/tests/semantic_signature_regressions.rs::semantic_model_baseline_for_import_and_binding_resolution`
   - `crates/lx-checker/tests/semantic_signature_regressions.rs::module_signature_baseline_for_exported_binding_and_trait`
13. Do not add new language features, new type forms, or a CST in this unit.

## Verification
1. Run `cargo test -p lx-checker --test checker_regressions --test semantic_signature_regressions`.
2. Run `cargo test -p lx-linter --test pipeline_regressions` because linting depends on checker semantics.
3. Run `just test`.
4. Treat any failure in the two Unit 01 semantic/signature baseline tests as a blocker for this unit unless the implementation diff also updates the baseline assertions and gives an explicit reason for the observable change.

## Out Of Scope
- New syntax features
- Lint framework changes beyond compile fixes
- CST or lossless syntax work
