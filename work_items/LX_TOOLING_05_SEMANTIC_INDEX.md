---
unit: 5
title: Semantic Index Expansion
scope: lx-checker
depends_on: LX_TOOLING_03_TYPE_SYNTAX_NORMALIZATION
optional: false
---

## Goal
Expand the reusable semantic model so lints and checker-adjacent analyses can query parents, scope structure, and basic control-flow facts without re-walking the AST.

## Why this boundary is isolated
This unit only widens the shared semantic data structure and the builder that produces it. It should not change lint rules or checker algorithms yet, which keeps the agent scope to one reusable analysis substrate.

## Primary crates/files touched
- `crates/lx-checker/src/semantic.rs`
- `crates/lx-checker/src/lib.rs`
- `crates/lx-checker/src/module_graph.rs` only if the new semantic fields need to be threaded into module signatures

## Mechanical task list
1. Add parent/containment metadata for expressions, statements, patterns, and type expressions where the current semantic model only stores defs and refs.
2. Add scope-boundary and control-flow summary data that callers can query directly instead of recomputing during each lint pass.
3. Extend `SemanticModelBuilder` so the new indexes are populated during checker traversal instead of as a second ad-hoc pass.
4. Preserve the existing `type_of_expr`, `type_of_def`, and `references_to` APIs so current consumers continue to compile unchanged.
5. Add accessor methods for the new semantic facts so later lint and checker work can stay query-based.
