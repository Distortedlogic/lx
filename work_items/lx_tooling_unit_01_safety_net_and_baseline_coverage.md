---
unit: 1
title: Safety Net and Baseline Coverage
scope: lx-parser, lx-desugar, lx-fmt, lx-checker, lx-linter
depends_on: none
optional: false
---

## Goal
Add regression coverage for the current LX tooling pipeline before any AST, visitor, semantic, or checker refactors land. The baseline for later units is the current observable behavior of lexing, parsing, desugaring, formatting, checking, and linting.

## Dependency Contract
This unit has no prerequisites. It must land before Units 02-07. Do not change production behavior in this unit except for test-only dependency wiring.

## Verified Preconditions
- `just test` exists in `justfile` and runs `cargo test --workspace --exclude inference-server --exclude lx-desktop --all-targets --all-features -q` followed by `cargo run -p lx-cli -- test`.
- The current public pipeline entry points exist:
  - `lx_parser::lexer::lex` in `crates/lx-parser/src/lexer/mod.rs`
  - `lx_parser::parse` in `crates/lx-parser/src/lib.rs`
  - `lx_desugar::desugar` in `crates/lx-desugar/src/folder/desugar.rs`
  - `lx_fmt::format` in `crates/lx-fmt/src/formatter/mod.rs`
  - `lx_checker::check` and `lx_checker::check_with_imports` in `crates/lx-checker/src/lib.rs`
  - `lx_linter::lint` in `crates/lx-linter/src/runner.rs`
- `rg -n "#\\[cfg\\(test\\)\\]|mod tests" crates/lx-ast crates/lx-parser crates/lx-desugar crates/lx-checker crates/lx-linter crates/lx-fmt` returns no existing crate-local test modules for this area.
- The current surface-only/core-only boundary is still behaviorally important:
  - `Stmt::KeywordDecl` still parses in `crates/lx-parser/src/parser/stmt.rs`
  - `validate_core` still checks for `KeywordDecl`, `Pipe`, `Section`, `Ternary`, `Coalesce`, and `With(Binding)` in `crates/lx-desugar/src/folder/validate_core.rs`
- `RuleRegistry::default_rules()` in `crates/lx-linter/src/registry.rs` currently registers:
  - `empty_match`
  - `redundant_propagate`
  - `break_outside_loop`
  - `unreachable_code`
  - `unused_import`
  - `duplicate_record_field`
  - `single_branch_par`
- `mut_never_mutated` currently runs outside the registry through `check_unused_mut(...)` in `crates/lx-linter/src/runner.rs`.

## Files To Create Or Change
- `crates/lx-parser/tests/surface_parse_regressions.rs`
- `crates/lx-desugar/tests/surface_to_core_regressions.rs`
- `crates/lx-fmt/tests/format_regressions.rs`
- `crates/lx-checker/tests/checker_regressions.rs`
- `crates/lx-linter/Cargo.toml`
- `crates/lx-linter/tests/pipeline_regressions.rs`
- `crates/lx-linter/tests/lint_regressions.rs`

## Exact Modules And Functions To Exercise
- `lx_parser::lexer::lex`
- `lx_parser::parse`
- `lx_desugar::desugar`
- `lx_fmt::format`
- `lx_checker::check`
- `lx_linter::lint`
- `lx_linter::RuleRegistry`
- `lx_ast::ast::attach_comments`
- `lx_ast::ast::Program::leading_comments`
- `lx_ast::ast::Program::trailing_comments`
- `lx_ast::ast::Program::dangling_comments`

## Mechanical Task List
1. In `crates/lx-parser/tests/surface_parse_regressions.rs`, add helper code that runs `lex` and `parse` on inline LX source strings and fails the test if any parse errors are returned.
2. Add one parser regression per currently important surface syntax family:
   - keyword declaration with `Stmt::KeywordDecl`
   - trait declaration with `TraitEntry::Field`
   - class declaration with fields and methods
   - binding with `type_ann`
   - nested `TypeExpr` shapes parsed by `parser/type_ann.rs`
3. In the same parser test file, add a comment-attachment regression that parses a source string containing comments before the first statement, after a statement, and inside a node span, then asserts non-empty results from `leading_comments`, `trailing_comments`, and `dangling_comments` for concrete `NodeId` values.
4. In `crates/lx-desugar/tests/surface_to_core_regressions.rs`, add a helper that runs `lex -> parse -> desugar` and fails the test if parse returns errors.
5. In the desugar test file, add one regression source string for each current surface-only construct that `validate_core` rejects:
   - `Stmt::KeywordDecl`
   - `Expr::Pipe`
   - `Expr::Section`
   - `Expr::Ternary`
   - `Expr::Coalesce`
   - `Expr::With(Binding)`
6. In each desugar regression, walk the resulting `Program<Core>` and assert that none of the rejected surface-only nodes remain. Reuse the same visitor shape as `validate_core` so later units can compare behavior directly.
7. In `crates/lx-fmt/tests/format_regressions.rs`, add format baselines for:
   - trait declarations with field defaults and method signatures
   - class declarations
   - keyword declarations
   - bindings with `TypeExprId` annotations
   - nested type expressions formatted through `emit_type.rs`
8. Make the formatter tests compare exact output strings, including trailing newline behavior from `Formatter::format_program`.
9. In `crates/lx-checker/tests/checker_regressions.rs`, add a helper that runs `lex -> parse -> desugar -> check` and returns `CheckResult`.
10. Add checker regressions for:
    - binding annotation mismatch
    - function parameter annotation resolution
    - match exhaustiveness warning
    - imported names resolved through `check_with_imports`
    - pattern bindings entering scope
11. In checker tests, assert exact diagnostic codes instead of only diagnostic counts. Use the current codes from `crates/lx-checker/src/diagnostics.rs`.
12. In `crates/lx-linter/Cargo.toml`, add the dev-dependencies needed for full-pipeline tests from the linter crate:
    - `lx-parser = { path = "../lx-parser" }`
    - `lx-desugar = { path = "../lx-desugar" }`
    - `lx-fmt = { path = "../lx-fmt" }`
13. In `crates/lx-linter/tests/pipeline_regressions.rs`, add an end-to-end helper that runs `lex -> parse -> desugar -> format -> check -> lint`.
14. In that pipeline test file, add one smoke test that asserts:
    - no parse errors on a valid sample
    - formatted output is non-empty
    - checker diagnostics are stable for the fixture
    - linter diagnostics are stable for the fixture
15. In `crates/lx-linter/tests/lint_regressions.rs`, add one dedicated source string per default-registered lint rule, plus one source string for the separate `mut_never_mutated` pipeline check:
    - `break_outside_loop`
    - `redundant_propagate`
    - `duplicate_record_field`
    - `unused_import`
    - `empty_match`
    - `single_branch_par`
    - `mut_never_mutated`
    - `unreachable_code`
16. Assert exact lint codes and the presence of the expected `rule_name` text in each lint diagnostic.
17. Do not add CST, trivia-preserving rewrite infrastructure, or any new production AST APIs in this unit.

## Verification
1. Run `cargo test -p lx-parser --test surface_parse_regressions`.
2. Run `cargo test -p lx-desugar --test surface_to_core_regressions`.
3. Run `cargo test -p lx-fmt --test format_regressions`.
4. Run `cargo test -p lx-checker --test checker_regressions`.
5. Run `cargo test -p lx-linter --test pipeline_regressions --test lint_regressions`.
6. Run `just test`.
7. Record any fixture strings added in the tests only. Do not leave expected output in comments or external scratch files.

## Out Of Scope
- Any AST shape changes
- Any visitor or transformer API changes
- Any semantic-model expansion
- Any CST or lossless syntax work
