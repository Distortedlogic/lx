---
unit: 1
title: Safety Net and Baseline Coverage
scope: lx-parser, lx-desugar, lx-checker, lx-linter, lx-fmt
depends_on: none
optional: false
---

## Goal
Add regression coverage around the current LX tooling pipeline before any AST, semantic, or visitor changes land. This unit locks down the existing parse, desugar, check, lint, and format behavior so the later refactors can be done against concrete fixtures instead of memory.

## Why this boundary is isolated
This unit is test-only. It does not change production code paths, so it can be executed independently and used as the baseline for all later units.

## Primary crates/files touched
- `crates/lx-parser/tests/*.rs`
- `crates/lx-desugar/tests/*.rs`
- `crates/lx-checker/tests/*.rs`
- `crates/lx-linter/tests/*.rs`
- `crates/lx-fmt/tests/*.rs`
- `tests/fixtures/**` or crate-local fixture directories created for this work

## Mechanical task list
1. Add parser fixtures that cover keyword declarations, trait declarations, class declarations, and type annotations that currently flow through `lx-parser`.
2. Add desugar fixtures that cover surface-to-core lowering for keyword declarations, `use` forms, pipe/section/ternary/coalesce lowering, and comment attachment survival.
3. Add checker fixtures that cover type inference, type annotation resolution, scope resolution, match exhaustiveness, and the current `unreachable!()` surface-only assumptions.
4. Add linter fixtures for `unused_import`, `mut_never_mutated`, `redundant_propagate`, `empty_match`, `break_outside_loop`, and `single_branch_par`.
5. Add formatter fixtures that cover trait/class/keyword emission and any current type annotation formatting path.
6. Record the expected diagnostics or formatted output for each fixture so the later refactors have a stable reference.

