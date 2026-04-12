---
unit: 2
title: Phase Hardening
scope: lx-ast, lx-parser, lx-desugar, lx-checker, lx-linter, lx-fmt, lx-eval
depends_on: lx_tooling_unit_01_safety_net_and_baseline_coverage
optional: false
---

## Goal
Make the `Surface` to `Core` boundary enforced by construction instead of by convention. `Program<Core>` must only be produced after validation, validation must run in all builds, and core consumers must stop depending on `unreachable!()` branches for surface-only syntax.

## Dependency Contract
Assume Unit 01 is merged and green. Use the new regression tests to hold behavior steady while tightening the phase boundary.

## Verified Preconditions
- `Program<Phase>` in `crates/lx-ast/src/ast/mod.rs` currently exposes public fields and uses `PhantomData<Phase>` only.
- `crates/lx-parser/src/parser/mod.rs` currently constructs `Program { stmts, arena, comments, comment_map, file, _phase: PhantomData }` directly.
- `crates/lx-desugar/src/folder/desugar.rs` currently returns `Program<Core>` by reusing the transformed arena and only calls `validate_core` behind `if cfg!(debug_assertions)`.
- `crates/lx-desugar/src/folder/validate_core.rs` currently panics on invalid core nodes instead of returning structured failures.
- The checker still assumes desugaring succeeded:
  - `Expr::Pipe`, `Expr::Section`, `Expr::Ternary`, and `Expr::Coalesce` are treated as impossible in `crates/lx-checker/src/check_expr.rs` and `crates/lx-checker/src/type_ops.rs`
  - `Expr::Tell` and `Expr::Ask` are also `unreachable!()` there even though `crates/lx-desugar/src/folder/desugar.rs` preserves them and `crates/lx-eval/src/interpreter/mod.rs` evaluates them
- Multiple crates read `program.stmts`, `program.arena`, or `program.file` directly:
  - `crates/lx-fmt/src/formatter/mod.rs`
  - `crates/lx-linter/src/runner.rs`
  - `crates/lx-linter/src/rules/unused_import.rs`
  - `crates/lx-linter/src/rules/mut_never_mutated.rs`
  - `crates/lx-checker/src/lib.rs`
  - `crates/lx-checker/src/module_graph.rs`
  - `crates/lx-eval/src/interpreter/*.rs`
  - `crates/lx-eval/src/stdlib/diag/*.rs`
  - `crates/lx-eval/src/stdlib/test_mod/test_invoke.rs`

## Files To Create Or Change
- `crates/lx-ast/src/ast/mod.rs`
- `crates/lx-ast/src/visitor/walk/mod.rs`
- `crates/lx-ast/src/visitor/walk_transform/mod.rs`
- `crates/lx-parser/src/parser/mod.rs`
- `crates/lx-desugar/src/folder/desugar.rs`
- `crates/lx-desugar/src/folder/validate_core.rs`
- `crates/lx-checker/src/lib.rs`
- `crates/lx-checker/src/check_expr.rs`
- `crates/lx-checker/src/type_ops.rs`
- `crates/lx-checker/src/diagnostics.rs`
- `crates/lx-checker/src/module_graph.rs`
- `crates/lx-linter/src/runner.rs`
- `crates/lx-linter/src/rules/unused_import.rs`
- `crates/lx-linter/src/rules/mut_never_mutated.rs`
- `crates/lx-fmt/src/formatter/mod.rs`
- `crates/lx-eval/src/interpreter/mod.rs`
- `crates/lx-eval/src/interpreter/default_tools.rs`
- `crates/lx-eval/src/interpreter/modules.rs`
- `crates/lx-eval/src/stdlib/diag/mod.rs`
- `crates/lx-eval/src/stdlib/diag/diag_walk.rs`
- `crates/lx-eval/src/stdlib/test_mod/test_invoke.rs`

## Exact Structs, Enums, And Functions To Inspect Or Change
- `lx_ast::ast::Program`
- `lx_ast::ast::Surface`
- `lx_ast::ast::Core`
- `lx_ast::visitor::walk_program`
- `lx_ast::visitor::walk_transform_program`
- `lx_desugar::desugar`
- `validate_core`
- `Checker::check_program`
- `Checker::check_expr`
- `Checker::synth_expr`
- `extract_signature`

## Mechanical Task List
1. In `crates/lx-ast/src/ast/mod.rs`, stop constructing `Program<Phase>` directly from outside the module.
2. Add explicit constructors and accessors on `Program<Surface>` and `Program<Core>`:
   - one constructor for parser output
   - one constructor for validated core output
   - read-only accessors for statements, arena, comments, comment map, and file
3. Keep the existing `leading_comments`, `trailing_comments`, and `dangling_comments` helpers working through the new accessors. Do not remove them.
4. Update `crates/lx-parser/src/parser/mod.rs` to construct `Program<Surface>` only through the new parser-side constructor.
5. In `crates/lx-desugar/src/folder/validate_core.rs`, replace panic-based validation with a structured error type that carries:
   - the invalid node kind
   - the failing `SourceSpan`
   - a stable human-readable message
6. Cover every currently invalid core case in the validator:
   - `Stmt::KeywordDecl`
   - `Expr::Pipe`
   - `Expr::Section`
   - `Expr::Ternary`
   - `Expr::Coalesce`
   - `Expr::With(Binding)`
7. In `crates/lx-desugar/src/folder/desugar.rs`, remove the `cfg!(debug_assertions)` gate. Validation must execute in all builds.
8. Make `desugar` return validated core output only. If validation fails, convert it into a hard error at the desugar boundary rather than letting invalid `Program<Core>` escape.
9. Update `crates/lx-ast/src/visitor/walk/mod.rs` and `crates/lx-ast/src/visitor/walk_transform/mod.rs` to use the new `Program` accessors instead of reading public fields.
10. Update every direct `program.stmts`, `program.arena`, `program.comments`, `program.comment_map`, and `program.file` access in the files listed above to call the new accessors.
11. In `crates/lx-checker/src/check_expr.rs` and `crates/lx-checker/src/type_ops.rs`, remove `unreachable!()` branches for surface-only nodes.
12. Replace those `unreachable!()` branches with one internal helper that records an explicit diagnostic and returns `type_arena.error()` if a surface-only node still reaches the checker.
13. In the same checker files, replace `Expr::Tell` and `Expr::Ask` `unreachable!()` branches with real checking logic:
    - `Tell`: check the target and message expressions and return `Unit`
    - `Ask`: check the target and message expressions and return `Unknown` until LX has a stronger protocol-level type
14. If `DiagnosticKind` needs a dedicated internal-invariant case to support Step 12, add it in `crates/lx-checker/src/diagnostics.rs` and give it a stable code.
15. Update `crates/lx-checker/src/lib.rs`, `crates/lx-linter/src/runner.rs`, and `crates/lx-checker/src/module_graph.rs` to work against the hardened `Program<Core>` accessors.
16. Update the `lx-eval` files listed above only as compile fixes for the new `Program` accessor API. Do not change evaluator behavior in this unit.
17. Do not split the arena into separate surface/core node types in this unit. The hardening target here is validated construction and unconditional enforcement, not a generic arena rewrite.
18. Do not introduce a CST or trivia-preserving syntax layer in this unit.

## Verification
1. Run the Unit 01 tests added for parser, desugar, formatter, checker, and linter.
2. Add one desugar test that proves invalid core validation is active in release-mode test runs by asserting that the validator is invoked without relying on `debug_assertions`.
3. Run `cargo test -p lx-desugar`.
4. Run `cargo test -p lx-checker`.
5. Run `cargo test -p lx-linter`.
6. Run `just test`.

## Out Of Scope
- Splitting `Expr` and `Stmt` into separate surface/core enums
- Traversal API redesign
- Semantic-model expansion
- CST or lossless syntax work
