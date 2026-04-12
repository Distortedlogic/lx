---
unit: 4
title: Traversal and Transform Ergonomics
scope: lx-ast, lx-desugar
depends_on: lx_tooling_unit_03_type_syntax_normalization
optional: false
---

## Goal
Make the AST traversal and rewrite API easier to use for future desugarers, AST modifiers, linters, and checker-side analyses. The target is less giant-enum matching, correct traversal of all surface nodes, and explicit rewrite control flow.

## Dependency Contract
Assume Units 01-03 are merged. This unit may change traversal APIs, but it must leave the tree shape and runtime behavior intact except where the current walker is demonstrably incomplete.

## Verified Preconditions
- `Stmt::KeywordDecl` is marked `#[walk(skip)]` in `crates/lx-ast/src/ast/mod.rs`.
- `walk_stmt` in `crates/lx-ast/src/visitor/walk/mod.rs` currently does nothing for `Stmt::KeywordDecl(_)`.
- `Desugarer::transform_stmts` in `crates/lx-desugar/src/folder/desugar.rs` currently special-cases `Stmt::KeywordDecl` because the generic transformer path does not walk it.
- `TransformOp::Stop` in `crates/lx-ast/src/visitor/transformer.rs` currently means “return the original node ID unchanged” in `crates/lx-ast/src/visitor/walk_transform/mod.rs`; it does not stop the overall traversal.
- `Desugarer::leave_expr` in `crates/lx-desugar/src/folder/desugar.rs` currently matches many `Expr` variants in one function.
- `TraitDeclData::children` and `walk_children` in `crates/lx-ast/src/ast/walk_impls.rs` currently ignore trait method output types.

## Files To Create Or Change
- `crates/lx-ast/src/ast/mod.rs`
- `crates/lx-ast/src/ast/walk_impls.rs`
- `crates/lx-ast/src/visitor/action.rs`
- `crates/lx-ast/src/visitor/transformer.rs`
- `crates/lx-ast/src/visitor/visitor_trait.rs`
- `crates/lx-ast/src/visitor/walk/mod.rs`
- `crates/lx-ast/src/visitor/walk/generated.rs`
- `crates/lx-ast/src/visitor/walk_transform/mod.rs`
- `crates/lx-ast/src/visitor/prelude.rs`
- `crates/lx-desugar/src/folder/desugar.rs`
- `crates/lx-desugar/tests/surface_to_core_regressions.rs`

## Exact Structs, Enums, And Functions To Inspect Or Change
- `Stmt`
- `KeywordDeclData`
- `TransformOp`
- `AstTransformer`
- `AstVisitor`
- `walk_stmt`
- `dispatch_stmt`
- `walk_transform_stmt`
- `walk_transform_expr`
- `Desugarer::transform_stmts`
- `Desugarer::leave_expr`

## Mechanical Task List
1. In `crates/lx-ast/src/ast/mod.rs`, remove `#[walk(skip)]` from `Stmt::KeywordDecl`.
2. In `crates/lx-ast/src/ast/walk_impls.rs`, add `children`, `walk_children`, and `recurse_children` support for `KeywordDeclData` so generic traversal reaches:
   - `fields[*].default`
   - `methods[*].handler`
   - `trait_entries[*].Field.default`
   - `trait_entries[*].Field.constraint`
   - `trait_entries[*].Field.type_name`
3. In `TraitDeclData` traversal helpers, add the missing traversal of method output type expressions introduced by Unit 03.
4. In `crates/lx-ast/src/visitor/visitor_trait.rs`, add typed visitor hooks for `KeywordDeclData` analogous to the existing typed hooks for trait/class declarations.
5. In `crates/lx-ast/src/visitor/walk/mod.rs`, route `Stmt::KeywordDecl` through the new typed keyword hooks instead of skipping it.
6. In `crates/lx-ast/src/visitor/transformer.rs`, rename the current `TransformOp::Stop` to `SkipChildren`. Do not keep a misleading alias behind the old name.
7. In the same transformer API, keep `Continue` and `Replace(T)`, and add explicit list-rewrite support for statement vectors so one statement can expand to zero or many statements without every consumer reimplementing it.
8. Implement the list-rewrite support in `crates/lx-ast/src/visitor/walk_transform/mod.rs`. The transform helper for statement lists must preserve source-order and allocate any replacement statements through `AstArena`.
9. Add typed transform hooks for the surface-only expression variants currently desugared in one giant match:
   - `Pipe`
   - `Section`
   - `Ternary`
   - `Coalesce`
   - `With`
10. Update `crates/lx-desugar/src/folder/desugar.rs` to move each surface-only expression rewrite into its matching typed hook instead of matching the whole `Expr` enum in one `leave_expr`.
11. Replace the current `Desugarer::transform_stmts` keyword special-case with the new statement-list rewrite hook from Step 7.
12. Preserve existing default behavior for current visitors and transformers. Any new hook must default to no-op behavior so downstream code compiles with minimal changes.
13. Update `crates/lx-ast/src/visitor/prelude.rs` exports so the new hooks and control-flow types are available through the standard prelude.
14. Add tests that prove:
   - generic traversal now reaches keyword internals
   - trait method output types are visited
   - statement list rewriting can expand a single statement into multiple statements
   - the renamed transform control flow behaves exactly as documented
15. Do not add a CST, token stream rewrite layer, or formatter-specific trivia walker in this unit.

## Verification
1. Run `cargo test -p lx-ast` if crate-local tests are added there.
2. Run `cargo test -p lx-desugar --test surface_to_core_regressions`.
3. Run the Unit 01 parser, formatter, checker, and linter tests to prove the traversal changes did not alter observable behavior.
4. Run `just test`.

## Out Of Scope
- Any new syntax layer
- Any semantic-model storage changes
- Checker pass refactoring
