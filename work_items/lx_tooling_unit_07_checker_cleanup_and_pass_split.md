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
2. Create `crates/lx-checker/src/pass_collect.rs` and move all top-level declaration/import collection logic there:
   - type definitions
   - type constructors
   - trait declarations
   - class declarations
   - imports
   - top-level binding names that must exist before body checking
3. Create `crates/lx-checker/src/pass_body.rs` and move the executable statement/body checking flow there:
   - `check_stmts`
   - `check_stmt`
   - top-level expression statements
   - binding value checking
4. Create `crates/lx-checker/src/pass_finalize.rs` and move final semantic assembly there. Keep `SemanticModelBuilder::build(...)` as the last step.
5. Update `crates/lx-checker/src/lib.rs` so `check_program` executes passes explicitly and in order:
   - collection
   - body checking
   - finalization
6. Leave expression-specific logic in `check_expr.rs`, `type_ops.rs`, `synth_control.rs`, and `synth_compound.rs`, but make those modules helpers called by the explicit body pass rather than indirect visitor callbacks.
7. Remove duplicate function-scope and match-arm logic by extracting shared helpers used by both “check against expected type” and “synthesize type” paths.
8. Make `infer_pattern.rs` own only pattern-type binding inference. It must not also schedule statement traversal.
9. Simplify `capture.rs` only as needed to align with the new pass boundaries and semantic helpers from Unit 05.
10. Update `module_graph.rs` if the pass split changes when exported binding types or trait fields become available. Preserve the observable `ModuleSignature` shape.
11. Delete any now-dead visitor glue in `lib.rs` after the explicit pass pipeline compiles and tests pass.
12. Keep diagnostics, semantic output, and exported-module signatures behaviorally stable against the Unit 01 baselines.
13. Do not add new language features, new type forms, or a CST in this unit.

## Verification
1. Run `cargo test -p lx-checker --test checker_regressions`.
2. Run `cargo test -p lx-linter --test pipeline_regressions` because linting depends on checker semantics.
3. Run `just test`.
4. Compare the Unit 01 checker baselines before and after the pass split. Any diagnostic-code change requires an explicit reason in the implementation diff.

## Out Of Scope
- New syntax features
- Lint framework changes beyond compile fixes
- CST or lossless syntax work
