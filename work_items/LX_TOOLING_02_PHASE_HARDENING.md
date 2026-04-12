---
unit: 2
title: Phase Hardening
scope: lx-ast, lx-desugar, lx-checker
depends_on: LX_TOOLING_01_SAFETY_NET
optional: false
---

## Goal
Make the `Surface`/`Core` split real instead of nominal. Core consumers should not rely on debug-only validation or panic-shaped invariants to exclude surface-only syntax.

## Why this boundary is isolated
This work is confined to the phase boundary and the code that consumes `Program<Core>`. It should not require traversal or semantic redesign, which keeps the agent scope focused and reduces overlap with later units.

## Primary crates/files touched
- `crates/lx-ast/src/ast/mod.rs`
- `crates/lx-desugar/src/folder/desugar.rs`
- `crates/lx-desugar/src/folder/validate_core.rs`
- `crates/lx-checker/src/check_expr.rs`
- `crates/lx-checker/src/type_ops.rs`
- `crates/lx-checker/src/lib.rs` if phase-boundary assumptions are centralized there

## Mechanical task list
1. Replace the debug-only `validate_core` call in desugaring with unconditional phase validation.
2. Convert the current core validator from panic-based checks into a real validation step that reports structured failures in the same style as the rest of the tooling.
3. Remove checker assumptions that rely on `unreachable!()` for surface-only expressions and statements.
4. Ensure every `Program<Core>` path only admits core-safe nodes, including the variants that are currently only excluded by convention.
5. Update any phase-related documentation or comments in the touched files so the post-change contract is explicit.

