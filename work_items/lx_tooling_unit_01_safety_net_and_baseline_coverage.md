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
- Binding-form `with` does not currently have a valid `lex -> parse` source-string path:
  - `with_binding` in `crates/lx-parser/src/parser/expr_compound.rs` parses the bound value as `expr.clone()` before it expects the body-opening `{`
  - `pratt_expr` in `crates/lx-parser/src/parser/expr_pratt.rs` still has empty-token application, so the following `{ ... }` can be greedily parsed as an argument to the binding value expression
  - representative fixtures such as `value = with x = 1 { x }` and `value = with mut x = 1 { x }` currently fail parse
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
- `crates/lx-desugar/Cargo.toml`
- `crates/lx-fmt/Cargo.toml`
- `crates/lx-checker/Cargo.toml`
- `crates/lx-parser/tests/surface_parse_regressions.rs`
- `crates/lx-desugar/tests/surface_to_core_regressions.rs`
- `crates/lx-fmt/tests/format_regressions.rs`
- `crates/lx-checker/tests/checker_regressions.rs`
- `crates/lx-checker/tests/semantic_signature_regressions.rs`
- `crates/lx-linter/Cargo.toml`
- `crates/lx-linter/tests/pipeline_regressions.rs`
- `crates/lx-linter/tests/lint_regressions.rs`

## Exact Modules And Functions To Exercise
- `lx_parser::lexer::lex`
- `lx_parser::parse`
- `lx_desugar::desugar`
- `lx_fmt::format`
- `lx_checker::check`
- `lx_checker::module_graph::extract_signature`
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
3. In the same parser test file, add one exact parser-backed comment regression named `comment_attachments_assign_leading_and_trailing_to_first_stmt` with this source string:
   ```lx
   -- leading on stmt
   x = 1 -- trailing on stmt
   ```
   Parse the source, define one local helper `fn first_stmt_id(program: &Program<Surface>) -> StmtId`, route the first-statement lookup through that helper, and assert all of the following on `NodeId::Stmt(first_stmt_id(&program))`:
   - `program.leading_comments(NodeId::Stmt(first_stmt_id(&program))).len() == 1`
   - `program.leading_comments(NodeId::Stmt(first_stmt_id(&program)))[0].text == "-- leading on stmt"`
   - `program.trailing_comments(NodeId::Stmt(first_stmt_id(&program))).len() == 1`
   - `program.trailing_comments(NodeId::Stmt(first_stmt_id(&program)))[0].text == "-- trailing on stmt"`
   The helper is the only place that may touch the first statement lookup in this test file. Implement it against the current `Program` layout for Unit 01, then update only that helper body to `program.stmts()[0]` when Unit 02 lands the `Program` accessor contract.
4. In the same parser test file, add one exact attachment-algorithm regression named `attach_comments_can_classify_dangling_for_a_synthetic_node_span`. Do not discover a dangling case from parsed LX source, because the current lexer only emits line comments. Build the fixture directly:
   - `source = "--slot;"`
   - allocate `expr_id = arena.alloc_expr(Expr::Ident(intern("slot")), (0, 6).into())`
   - allocate `stmt_id = arena.alloc_stmt(Stmt::Expr(expr_id), (0, 7).into())`
   - build `comments = CommentStore::from_vec(vec![Comment { span: (0, 2).into(), text: "--".into() }])`
   - call `attach_comments(&[stmt_id], &arena, &comments, source)`
   - build a `Program<Surface>` from that arena and returned `comment_map`
   - assert `program.dangling_comments(NodeId::Expr(expr_id)).len() == 1`
   - assert `program.leading_comments(NodeId::Expr(expr_id)).is_empty()`
   - assert `program.trailing_comments(NodeId::Expr(expr_id)).is_empty()`
5. In `crates/lx-parser/tests/surface_parse_regressions.rs`, add one exact current-behavior blocker regression named `with_binding_source_forms_currently_fail_to_parse`. It must run `lex -> parse` on all of these representative fixtures and assert that each one returns at least one parse error:
   - `value = with x = 1 { x }`
   - `value = (with x = 1 { x })`
   - `value = with mut x = 1 { x }`
   This regression documents the current parser limitation only. Do not work around it in this unit by constructing a manual `Program<Surface>` fixture for `Expr::With(Binding)`.
6. In `crates/lx-desugar/tests/surface_to_core_regressions.rs`, add a helper that runs `lex -> parse -> desugar` and fails the test if parse returns errors.
7. In the desugar test file, add one regression source string for each current surface-only construct that `validate_core` rejects and that the current parser can actually construct from source:
   - `Stmt::KeywordDecl`
   - `Expr::Pipe`
   - `Expr::Section`
   - `Expr::Ternary`
   - `Expr::Coalesce`
8. In each desugar regression, walk the resulting `Program<Core>` and assert that none of the rejected surface-only nodes remain. Reuse the same visitor shape as `validate_core` so later units can compare behavior directly.
9. In `crates/lx-fmt/tests/format_regressions.rs`, add format baselines for:
   - trait declarations with field defaults and method signatures
   - class declarations
   - keyword declarations
   - bindings with `TypeExprId` annotations
   - nested type expressions formatted through `emit_type.rs`
10. Make the formatter tests compare exact output strings, including trailing newline behavior from `Formatter::format_program`.
11. In `crates/lx-checker/tests/checker_regressions.rs`, add a helper that runs `lex -> parse -> desugar -> check` and returns `CheckResult`.
12. Add checker regressions for:
   - binding annotation mismatch
   - function parameter annotation resolution
   - match exhaustiveness warning
   - imported names resolved through `check_with_imports`
   - pattern bindings entering scope
13. In checker tests, assert exact diagnostic codes instead of only diagnostic counts. Use the current codes from `crates/lx-checker/src/diagnostics.rs`.
14. In `crates/lx-checker/tests/semantic_signature_regressions.rs`, add one exact semantic-model baseline named `semantic_model_baseline_for_import_and_binding_resolution` with this source string:
   ```lx
   use std/time
   answer = 1
   value = answer
   ```
   Run `lex -> parse -> desugar -> check` and assert:
   - `model.definitions.len() == 3`
   - definition order is `Import(time)`, `Binding(answer)`, `Binding(value)`
   - `model.references_to(answer_def).len() == 1`
   - `model.display_type(model.type_of_def(answer_def).unwrap()) == "Int"`
   - `model.display_type(model.type_of_expr(value_expr_id).unwrap()) == "Int"`
15. In the same checker test file, add one exact module-signature baseline named `module_signature_baseline_for_exported_binding_and_trait` with this source string:
   ```lx
   +answer = 1
   +Trait Pair = { left: Int; right: Int }
   ```
   After `check`, call `extract_signature(&program, &result.semantic)` and assert:
   - `signature.bindings` contains only `answer`
   - `signature.traits` contains only `Pair`
   - `signature.types` is empty
   - `signature.traits[Pair].len() == 2`
16. In `crates/lx-linter/Cargo.toml`, add the dev-dependencies needed for full-pipeline tests from the linter crate:
   - `lx-parser = { path = "../lx-parser" }`
   - `lx-desugar = { path = "../lx-desugar" }`
   - `lx-fmt = { path = "../lx-fmt" }`
17. In `crates/lx-desugar/Cargo.toml`, `crates/lx-fmt/Cargo.toml`, and `crates/lx-checker/Cargo.toml`, add the exact dev-dependencies needed for the new integration tests in this unit:
   - `crates/lx-desugar/Cargo.toml` must add `lx-parser = { path = "../lx-parser" }` under `[dev-dependencies]`
   - `crates/lx-fmt/Cargo.toml` must add `lx-parser = { path = "../lx-parser" }` and `lx-desugar = { path = "../lx-desugar" }` under `[dev-dependencies]`
   - `crates/lx-checker/Cargo.toml` must add `lx-parser = { path = "../lx-parser" }` and `lx-desugar = { path = "../lx-desugar" }` under `[dev-dependencies]`
18. In `crates/lx-linter/tests/pipeline_regressions.rs`, add an end-to-end helper that runs `lex -> parse -> desugar -> format -> check -> lint`.
19. In that pipeline test file, add one smoke test that asserts:
   - no parse errors on a valid sample
   - formatted output is non-empty
   - checker diagnostics are stable for the fixture
   - linter diagnostics are stable for the fixture
20. In `crates/lx-linter/tests/lint_regressions.rs`, add one dedicated source string per default-registered lint rule, plus one source string for the separate `mut_never_mutated` pipeline check:
   - `break_outside_loop`
   - `redundant_propagate`
   - `duplicate_record_field`
   - `unused_import`
   - `empty_match`
   - `single_branch_par`
   - `mut_never_mutated`
   - `unreachable_code`
21. Assert exact lint codes and the presence of the expected `rule_name` text in each lint diagnostic.
22. Do not add CST, trivia-preserving rewrite infrastructure, any new production AST APIs, or any parser grammar changes in this unit.

## Verification
1. Run `cargo test -p lx-parser --test surface_parse_regressions`.
2. Run `cargo test -p lx-desugar --test surface_to_core_regressions`.
3. Run `cargo test -p lx-fmt --test format_regressions`.
4. Run `cargo test -p lx-checker --test checker_regressions --test semantic_signature_regressions`.
5. Run `cargo test -p lx-linter --test pipeline_regressions --test lint_regressions`.
6. Run `just test`.
7. Record any fixture strings added in the tests only. Do not leave expected output in comments or external scratch files.

## Out Of Scope
- Any AST shape changes
- Any visitor or transformer API changes
- Any semantic-model expansion
- Fixing the parser grammar so binding-form `with` parses from source
- Any CST or lossless syntax work
