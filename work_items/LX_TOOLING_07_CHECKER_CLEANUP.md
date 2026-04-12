---
unit: 7
title: Checker Cleanup and Pass Split
scope: lx-checker
depends_on: LX_TOOLING_02_PHASE_HARDENING, LX_TOOLING_03_TYPE_SYNTAX_NORMALIZATION, LX_TOOLING_05_SEMANTIC_INDEX, LX_TOOLING_06_LINT_FIX_REPLATFORMING
optional: false
---

## Goal
Simplify the checker architecture after the phase, type, semantic, and lint boundaries are stable. The end state should make declaration collection, type resolution, inference, and semantic emission easier to reason about than the current mixed visitor flow.

## Why this boundary is isolated
This is a checker-internal refactor. It can lean on the earlier structural changes, but it should not introduce new AST or linter concepts of its own.

## Primary crates/files touched
- `crates/lx-checker/src/lib.rs`
- `crates/lx-checker/src/check_expr.rs`
- `crates/lx-checker/src/visit_stmt.rs`
- `crates/lx-checker/src/type_ops.rs`
- `crates/lx-checker/src/capture.rs`
- `crates/lx-checker/src/synth_control.rs`
- `crates/lx-checker/src/synth_compound.rs`
- `crates/lx-checker/src/infer_pattern.rs`
- `crates/lx-checker/src/module_graph.rs` if the new pass split changes signature extraction flow

## Mechanical task list
1. Separate the checker into explicit phases for collection, expression synthesis/checking, and semantic finalization if any of those are still interleaved.
2. Remove duplicated traversal assumptions between `visit_stmt`, `check_stmt`, `check_expr`, and `synth_expr`.
3. Centralize type-resolution helpers so core inference paths do not each re-interpret the same AST nodes.
4. Rework capture, narrowing, and compound-expression handling so each helper owns one responsibility instead of depending on incidental visitor order.
5. Keep the emitted diagnostics and semantic output identical where behavior is already correct, and only change structure where the old code was knotted.

