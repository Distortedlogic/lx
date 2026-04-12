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
- The actual `desugar(...)` call sites that must compile against the new boundary contract are:
  - `crates/lx-cli/src/agent_cmd.rs`
  - `crates/lx-cli/src/check.rs`
  - `crates/lx-cli/src/run.rs`
  - `crates/lx-eval/src/interpreter/default_tools.rs`
  - `crates/lx-eval/src/interpreter/modules.rs`
  - `crates/lx-eval/src/stdlib/test_mod/test_invoke.rs`

## Boundary Contract
- Change the desugar boundary to `pub fn desugar(program: Program<Surface>) -> Result<Program<Core>, DesugarError>`.
- Move core validation to the `lx-ast` boundary so `Program<Core>` cannot be constructed publicly without validation.
- Do not leave any public unchecked `Program::new_core(...)` constructor in the API.
- `CoreValidationError` must contain:
  - `node_kind: &'static str`
  - `span: SourceSpan`
  - `message: String`
- Define `pub fn Program::try_new_core(stmts, arena, comments, comment_map, file) -> Result<Program<Core>, Vec<CoreValidationError>>` in `crates/lx-ast/src/ast/mod.rs`.
- `Program::try_new_core(...)` must validate before returning `Program<Core>`. If `lx-ast` needs an unchecked constructor internally, keep it private to the module and use it only from `try_new_core(...)`.
- Define `DesugarError::InvalidCore(Vec<CoreValidationError>)` in `crates/lx-desugar/src/folder/validate_core.rs`.
- Define `validate_core_parts(stmts: &[StmtId], arena: &AstArena) -> Result<(), Vec<CoreValidationError>>` in `crates/lx-ast/src/ast/validate_core.rs`, and have `Program::try_new_core(...)` call it before constructing the public `Program<Core>`.
- Callers must not panic or process-exit inside `lx-desugar`. Every `desugar(...)` caller listed above must handle `Err(DesugarError::InvalidCore(_))` explicitly and map it into the caller’s existing error channel:
  - `lx-cli` command paths print the diagnostic text and return non-zero `ExitCode`
  - `lx-cli::run::run` returns `Err(Vec<LxError>)`
  - `lx-eval` module-loading paths return their existing `Result<..., LxError>` or `EvalResult<_>` failures

## Files To Create Or Change
- `crates/lx-ast/src/ast/mod.rs`
- `crates/lx-ast/src/ast/validate_core.rs`
- `crates/lx-ast/src/visitor/walk/mod.rs`
- `crates/lx-ast/src/visitor/walk_transform/mod.rs`
- `crates/lx-parser/src/parser/mod.rs`
- `crates/lx-desugar/src/lib.rs`
- `crates/lx-desugar/src/folder/desugar.rs`
- `crates/lx-desugar/src/folder/validate_core.rs`
- `crates/lx-desugar/tests/surface_to_core_regressions.rs`
- `crates/lx-checker/src/lib.rs`
- `crates/lx-checker/src/check_expr.rs`
- `crates/lx-checker/src/type_ops.rs`
- `crates/lx-checker/src/diagnostics.rs`
- `crates/lx-checker/src/module_graph.rs`
- `crates/lx-linter/src/runner.rs`
- `crates/lx-linter/src/rules/unused_import.rs`
- `crates/lx-linter/src/rules/mut_never_mutated.rs`
- `crates/lx-fmt/src/formatter/mod.rs`
- `crates/lx-cli/src/agent_cmd.rs`
- `crates/lx-cli/src/check.rs`
- `crates/lx-cli/src/run.rs`
- `crates/lx-eval/src/interpreter/mod.rs`
- `crates/lx-eval/src/interpreter/default_tools.rs`
- `crates/lx-eval/src/interpreter/modules.rs`
- `crates/lx-eval/src/stdlib/diag/mod.rs`
- `crates/lx-eval/src/stdlib/diag/diag_walk.rs`
- `crates/lx-eval/src/stdlib/test_mod/test_invoke.rs`

## Exact Structs, Enums, And Functions To Inspect Or Change
- `lx_ast::ast::Program`
- `lx_ast::ast::Program::try_new_core`
- `lx_ast::ast::Surface`
- `lx_ast::ast::Core`
- `validate_core_parts`
- `lx_ast::visitor::walk_program`
- `lx_ast::visitor::walk_transform_program`
- `lx_desugar::desugar`
- `Checker::check_program`
- `Checker::check_expr`
- `Checker::synth_expr`
- `extract_signature`

## Mechanical Task List
1. In `crates/lx-ast/src/ast/mod.rs`, stop constructing `Program<Phase>` directly from outside the module.
2. Add explicit constructors and accessors on `Program<Surface>` and `Program<Core>`:
   - `Program::new_surface(stmts, arena, comments, comment_map, file) -> Program<Surface>` for parser output
   - `Program::try_new_core(stmts, arena, comments, comment_map, file) -> Result<Program<Core>, Vec<CoreValidationError>>` for validated core output
   - no public unchecked `new_core` constructor
   - exact read-only accessors named `stmts()`, `arena()`, `comments()`, `comment_map()`, and `file()`
3. In the same file, add the exact write-path API needed by `walk_transform_program` inside `lx-ast`:
   - `pub(crate) fn arena_mut(&mut self) -> &mut AstArena`
   - `pub(crate) fn replace_stmts(&mut self, stmts: Vec<StmtId>)`
   `walk_transform_program` must use these crate-visible mutators instead of public field writes.
4. Keep the existing `leading_comments`, `trailing_comments`, and `dangling_comments` helpers working through the new accessors. Do not remove them.
5. Update `crates/lx-parser/src/parser/mod.rs` to construct `Program<Surface>` only through `Program::new_surface(...)`.
6. In `crates/lx-ast/src/ast/validate_core.rs`, move the core validator out of `lx-desugar` and replace panic-based validation with `CoreValidationError` plus `validate_core_parts(...)` as defined in the boundary contract above.
7. Cover every currently invalid core case in `validate_core_parts(...)`:
   - `Stmt::KeywordDecl`
   - `Expr::Pipe`
   - `Expr::Section`
   - `Expr::Ternary`
   - `Expr::Coalesce`
   - `Expr::With(Binding)`
8. In `crates/lx-desugar/src/folder/validate_core.rs`, keep only the `DesugarError` wrapper and any formatting helpers needed to surface `CoreValidationError` values from `lx-ast`.
9. In `crates/lx-desugar/src/folder/desugar.rs`, remove the `cfg!(debug_assertions)` gate. Validation must execute in all builds.
10. Implement `desugar(...) -> Result<Program<Core>, DesugarError>` with this exact flow:
   - lower surface syntax into transformed parts
   - call `Program::try_new_core(...)`
   - return `Ok(core)` on success
   - return `Err(DesugarError::InvalidCore(errors))` on failure
11. Update `crates/lx-desugar/src/lib.rs` to re-export the new `Result`-returning `desugar`.
12. Update `crates/lx-ast/src/visitor/walk/mod.rs` and `crates/lx-ast/src/visitor/walk_transform/mod.rs` to use the new `Program` accessors plus `arena_mut()` and `replace_stmts(...)` instead of public field access.
13. Update every direct `program.stmts`, `program.arena`, `program.comments`, `program.comment_map`, and `program.file` access in the files listed above to call the new accessors.
14. Update every actual `desugar(...)` caller listed in `Verified Preconditions` to handle `Result` explicitly. Do not leave any implicit `.unwrap()` or panic path in command or evaluator code.
15. In `crates/lx-checker/src/check_expr.rs` and `crates/lx-checker/src/type_ops.rs`, remove `unreachable!()` branches for surface-only nodes.
16. Replace those `unreachable!()` branches with one internal helper that records an explicit diagnostic and returns `type_arena.error()` if a surface-only node still reaches the checker.
17. In the same checker files, replace `Expr::Tell` and `Expr::Ask` `unreachable!()` branches with real checking logic:
   - `Tell`: check the target and message expressions and return `Unit`
   - `Ask`: check the target and message expressions and return `Unknown` until LX has a stronger protocol-level type
18. If `DiagnosticKind` needs a dedicated internal-invariant case to support Step 16, add it in `crates/lx-checker/src/diagnostics.rs` and give it a stable code.
19. Update `crates/lx-checker/src/lib.rs`, `crates/lx-linter/src/runner.rs`, and `crates/lx-checker/src/module_graph.rs` to work against the hardened `Program<Core>` accessor API by calling `program.stmts()`, `program.arena()`, `program.comments()`, `program.comment_map()`, and `program.file()` instead of direct field access.
20. Update the `lx-eval` files listed above only as compile fixes for the new `Program` accessor API and the `desugar(...) -> Result<...>` contract. Do not change evaluator behavior in this unit.
21. In `crates/lx-desugar/tests/surface_to_core_regressions.rs`, add one regression that builds invalid would-be core parts, calls `Program::try_new_core(...)`, and asserts it returns `Err(Vec<CoreValidationError>)` in normal test builds. The test must not construct `Program<Core>` through any unchecked external API.
22. Do not split the arena into separate surface/core node types in this unit. The hardening target here is validated construction and unconditional enforcement, not a generic arena rewrite.
23. Do not introduce a CST or trivia-preserving syntax layer in this unit.

## Verification
1. Run the Unit 01 tests added for parser, desugar, formatter, checker, and linter.
2. Run `cargo test -p lx-desugar --test surface_to_core_regressions`.
3. Run `cargo test -p lx-checker`.
4. Run `cargo test -p lx-linter`.
5. Run `cargo test -p lx-cli`.
6. Run `cargo test -p lx-eval`.
7. Run `just test`.

## Out Of Scope
- Splitting `Expr` and `Stmt` into separate surface/core enums
- Traversal API redesign
- Semantic-model expansion
- CST or lossless syntax work
