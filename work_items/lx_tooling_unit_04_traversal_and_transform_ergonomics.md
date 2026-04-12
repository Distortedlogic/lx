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
- `AstTransformer::transform_stmts` in `crates/lx-ast/src/visitor/transformer.rs` currently rewrites whole statement vectors without any owner context or defined pre-order expansion contract.

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
- `crates/lx-ast/tests/traversal_transform_regressions.rs`
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
- `StmtListOwner`
- `StmtListRewrite`
- `AstTransformer::rewrite_stmt_list_item`
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
7. In `crates/lx-ast/src/visitor/transformer.rs`, replace the vague whole-vector `transform_stmts` contract with one exact pre-order hook:
   - add `enum StmtListOwner { Program, ExprBlock(ExprId), ExprLoop(ExprId), ExprPar(ExprId), ExprWith(ExprId) }`
   - add `enum StmtListRewrite { Keep, Remove, Replace(Vec<(Stmt, SourceSpan)>) }`
   - add `fn rewrite_stmt_list_item(&mut self, owner: StmtListOwner, id: StmtId, stmt: &Stmt, span: SourceSpan, arena: &mut AstArena) -> StmtListRewrite`
   - default behavior must be `StmtListRewrite::Keep`
8. In `crates/lx-ast/src/visitor/walk_transform/mod.rs`, implement that contract exactly for every statement-list owner named in Step 7. Execution order must be:
   - iterate the owner list in source order
   - call `rewrite_stmt_list_item(...)` before any child traversal for that statement
   - if the result is `Keep`, recursively transform the original `StmtId`
   - if the result is `Remove`, emit nothing
   - if the result is `Replace(Vec<(Stmt, SourceSpan)>)`, allocate each replacement through `AstArena`, then recursively transform each replacement statement in source order before appending it to the rewritten list
9. Add typed transform hooks for the surface-only expression variants currently desugared in one giant match:
   - add pre-order hooks on `AstTransformer` with these exact signatures:
     - `fn transform_pipe(&mut self, id: ExprId, pipe: &ExprPipe, span: SourceSpan, arena: &AstArena) -> TransformOp<Expr>`
     - `fn transform_section(&mut self, id: ExprId, section: &Section, span: SourceSpan, arena: &AstArena) -> TransformOp<Expr>`
     - `fn transform_ternary(&mut self, id: ExprId, ternary: &ExprTernary, span: SourceSpan, arena: &AstArena) -> TransformOp<Expr>`
     - `fn transform_coalesce(&mut self, id: ExprId, coalesce: &ExprCoalesce, span: SourceSpan, arena: &AstArena) -> TransformOp<Expr>`
     - `fn transform_with(&mut self, id: ExprId, with: &ExprWith, span: SourceSpan, arena: &AstArena) -> TransformOp<Expr>`
   - add matching post-order hooks on `AstTransformer` with these exact signatures:
     - `fn leave_pipe(&mut self, id: ExprId, pipe: ExprPipe, span: SourceSpan, arena: &mut AstArena) -> (Expr, SourceSpan)`
     - `fn leave_section(&mut self, id: ExprId, section: Section, span: SourceSpan, arena: &mut AstArena) -> (Expr, SourceSpan)`
     - `fn leave_ternary(&mut self, id: ExprId, ternary: ExprTernary, span: SourceSpan, arena: &mut AstArena) -> (Expr, SourceSpan)`
     - `fn leave_coalesce(&mut self, id: ExprId, coalesce: ExprCoalesce, span: SourceSpan, arena: &mut AstArena) -> (Expr, SourceSpan)`
     - `fn leave_with(&mut self, id: ExprId, with: ExprWith, span: SourceSpan, arena: &mut AstArena) -> (Expr, SourceSpan)`
   - execution order must be: `transform_expr` first for every expression, then the typed pre-order hook for the matching surface-only variant if the generic hook returned `Continue`, then child recursion if the typed hook also returned `Continue`, then the matching typed leave hook, then the generic `leave_expr`
   - `TransformOp::SkipChildren` short-circuits child recursion and both leave hooks for that node while preserving the original node ID
10. Update `crates/lx-desugar/src/folder/desugar.rs` to move each surface-only expression rewrite into its matching typed hook instead of matching the whole `Expr` enum in one `leave_expr`.
11. Replace the current `Desugarer::transform_stmts` keyword special-case with `rewrite_stmt_list_item(...)` on `StmtListOwner`. For `Stmt::KeywordDecl`, return `StmtListRewrite::Replace(...)` from that pre-order hook using the desugared statements and original statement span. For all other statements, return `StmtListRewrite::Keep`.
12. Preserve existing default behavior for current visitors and transformers. Any new hook must default to no-op behavior so downstream code compiles with minimal changes.
13. Update `crates/lx-ast/src/visitor/prelude.rs` exports so the new hooks and control-flow types are available through the standard prelude.
14. Create `crates/lx-ast/tests/traversal_transform_regressions.rs` and put all traversal/transform API regression tests for this unit there. That file must prove:
   - generic traversal now reaches keyword internals
   - trait method output types are visited
   - `rewrite_stmt_list_item(...)` runs pre-order on `Program`, `ExprBlock`, `ExprLoop`, `ExprPar`, and `ExprWith` statement lists
   - statement list rewriting can expand one statement into multiple statements and the replacements are recursively transformed in source order
   - `TransformOp::SkipChildren` preserves the original node ID and skips child recursion exactly as documented
15. Do not add a CST, token stream rewrite layer, or formatter-specific trivia walker in this unit.

## Verification
1. Run `cargo test -p lx-ast --test traversal_transform_regressions`.
2. Run `cargo test -p lx-desugar --test surface_to_core_regressions`.
3. Run the Unit 01 parser, formatter, checker, and linter tests to prove the traversal changes did not alter observable behavior.
4. Run `just test`.

## Out Of Scope
- Any new syntax layer
- Any semantic-model storage changes
- Checker pass refactoring
