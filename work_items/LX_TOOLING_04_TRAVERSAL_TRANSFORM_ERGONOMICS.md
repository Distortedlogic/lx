---
unit: 4
title: Traversal and Transform Ergonomics
scope: lx-ast, lx-desugar
depends_on: LX_TOOLING_03_TYPE_SYNTAX_NORMALIZATION
optional: false
---

## Goal
Make the AST visitor and transformer APIs more ergonomic for future modifiers, linters, and type-directed rewrites. The target is clearer hooks, better local rewrite control, and less giant-enum matching in consumers.

## Why this boundary is isolated
This is an API-layer change inside `lx-ast`. It should stay backward-compatible enough for current consumers, so it can be developed without bundling semantic or checker work into the same agent.

## Primary crates/files touched
- `crates/lx-ast/src/visitor/visitor_trait.rs`
- `crates/lx-ast/src/visitor/transformer.rs`
- `crates/lx-ast/src/visitor/walk/mod.rs`
- `crates/lx-ast/src/visitor/walk/generated.rs`
- `crates/lx-ast/src/visitor/walk_transform/mod.rs`
- `crates/lx-ast/src/ast/walk_impls.rs`
- `crates/lx-ast/src/visitor/prelude.rs` if exported hooks change

## Mechanical task list
1. Extend the visitor surface with more variant-specific hooks where the current generic `visit_expr`/`leave_expr` layer forces downstream enum matching.
2. Clarify transform control flow so skip, replace, remove, and child-splice behavior are named for what they do instead of overloading `Stop`.
3. Add rewrite support for list-like children so transformers can rebuild vectors without manual reconstruction in every consumer.
4. Keep existing traversal behavior intact for current consumers while exposing the new ergonomic hooks as additive API.
5. Update the generated walk/rewrite helpers so the new control-flow semantics are implemented in one place rather than re-created per consumer.

