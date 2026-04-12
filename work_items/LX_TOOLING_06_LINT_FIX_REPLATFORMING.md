---
unit: 6
title: Lint and Fix Replatforming
scope: lx-linter, lx-checker
depends_on: LX_TOOLING_05_SEMANTIC_INDEX
optional: false
---

## Goal
Move lint rules off manual AST recursion and onto the shared semantic model, while keeping fixes explicit and applicability-aware.

## Why this boundary is isolated
This unit is a consumer rewrite of the semantic layer. It should not change AST shape or semantic storage, only how lint rules query and report on the already-built model.

## Primary crates/files touched
- `crates/lx-linter/src/runner.rs`
- `crates/lx-linter/src/rule.rs`
- `crates/lx-linter/src/registry.rs`
- `crates/lx-linter/src/rules/mut_never_mutated.rs`
- `crates/lx-linter/src/rules/*.rs` for rules that can switch from recursive scans to semantic queries
- `crates/lx-checker/src/diagnostics.rs` only if fix applicability or fix payloads need to be widened for lint output

## Mechanical task list
1. Replace any rule-local traversal that recomputes scope or mutation information with direct semantic-model queries.
2. Make `mut_never_mutated` consume semantic references or control-flow facts instead of walking the AST by hand.
3. Keep rule registration and runner flow unchanged except for any new semantic accessors needed by the rules.
4. Ensure rule diagnostics continue to surface through the existing `Diagnostic` pipeline, including fixes and applicability where available.
5. Update or remove any rule helper code that becomes dead after semantic queries cover the same information.

