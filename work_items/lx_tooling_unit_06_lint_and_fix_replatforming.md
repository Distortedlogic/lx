---
unit: 6
title: Lint and Fix Replatforming
scope: lx-linter, lx-checker
depends_on: lx_tooling_unit_05_semantic_index_expansion
optional: false
---

## Goal
Move lint rules onto the shared semantic queries from Unit 05, eliminate duplicated AST recursion for scope-sensitive rules, and keep automated fixes explicit about applicability.

## Dependency Contract
Assume Units 01-05 are merged. This unit consumes the semantic model through the query methods introduced in Unit 05. Do not add new `SemanticModel` fields in this unit.

## Verified Preconditions
- `LintRule` in `crates/lx-linter/src/rule.rs` currently mixes three execution styles:
  - whole-program `run`
  - per-expression `check_expr` / `leave_expr`
  - per-statement `check_stmt`
- `lint` in `crates/lx-linter/src/runner.rs` currently:
  - calls `rule.run(program, model)` for each rule
  - walks the AST with `LintWalker`
  - appends a separate `check_unused_mut` pass from `mut_never_mutated.rs`
- `mut_never_mutated.rs` currently bypasses the rule registry entirely and recursively scans AST shapes by hand.
- `unused_import.rs` already uses semantic definitions and references, but still re-scans top-level statements to rediscover import bindings.
- `lx_checker::diagnostics` already has `Fix` and `Applicability`.
- Unit 05 now guarantees these query methods without adding import-specific storage:
  - `definition(DefinitionId) -> &DefinitionInfo`
  - `import_definitions() -> Vec<DefinitionId>`
  - `references_to(DefinitionId) -> &[ExprId]`
  - `scope_of(NodeId) -> Option<ScopeId>`
  - `control_context(ExprId) -> Option<&ControlContext>`
  - `mutation_sites(DefinitionId) -> &[MutationSite]`

## Files To Create Or Change
- `crates/lx-linter/src/rule.rs`
- `crates/lx-linter/src/runner.rs`
- `crates/lx-linter/src/registry.rs`
- `crates/lx-linter/src/rules/mut_never_mutated.rs`
- `crates/lx-linter/src/rules/unused_import.rs`
- `crates/lx-linter/src/rules/break_outside_loop.rs`
- `crates/lx-linter/src/rules/unreachable_code.rs`
- `crates/lx-linter/src/rules/redundant_propagate.rs`
- `crates/lx-linter/src/rules/empty_match.rs`
- `crates/lx-linter/src/rules/single_branch_par.rs`
- `crates/lx-linter/src/rules/duplicate_record_field.rs`
- `crates/lx-linter/src/lib.rs`
- `crates/lx-linter/tests/lint_regressions.rs`
- `crates/lx-linter/tests/pipeline_regressions.rs`
- `crates/lx-checker/src/diagnostics.rs` only if a lint needs a stronger fix payload shape than the current `Fix`

## Exact Structs And Functions To Inspect Or Change
- `LintRule`
- `RuleContext`
- `RuleRegistry`
- `lint`
- `LintWalker`
- `check_unused_mut`
- `UnusedImport::run`
- `BreakOutsideLoop`
- `Diagnostic`
- `Fix`
- `Applicability`

## Mechanical Task List
1. In `crates/lx-linter/src/rule.rs`, replace the current mixed execution trait with this exact contract:
   - `fn name(&self) -> &'static str;`
   - `fn code(&self) -> &'static str;`
   - `fn category(&self) -> RuleCategory;`
   - `fn run(&mut self, ctx: &RuleContext);`
   - `fn take_diagnostics(&mut self) -> Vec<Diagnostic>;`
   Remove `check_expr`, `leave_expr`, and `check_stmt` from the trait entirely in this unit.
2. Add `RuleContext` in the same file. It must carry:
   - `&Program<Core>`
   - `&AstArena`
   - `&SemanticModel`
   - helpers for `program.stmts()`, `program.arena()`, `program.comments()`, `program.comment_map()`, and `program.file()`, plus definitions, references, parents, scopes, control context, and mutation sites
3. `RuleContext` is the only runner-owned input to `LintRule::run`. If a rule needs structural traversal, it must create and invoke its own local `AstVisitor` or direct AST iteration from inside `run(&RuleContext)`. The runner must not provide a separate walker callback channel after this refactor.
4. In `crates/lx-linter/src/runner.rs`, remove the current split between `rule.run(...)`, `LintWalker`, and `check_unused_mut(...)`.
5. Replace that split with one exact runner flow:
   - build one `RuleContext` from `program`, `program.arena()`, and `model`
   - iterate `registry.rules_mut()`
   - call `rule.run(&ctx)` once per rule
   - immediately collect `rule.take_diagnostics()` into the output vector
6. Delete the separate `check_unused_mut` pipeline hook. `mut_never_mutated` must become a normal registered rule.
7. In `crates/lx-linter/src/registry.rs`, register `mut_never_mutated` through the same mechanism as the other rules.
8. In `crates/lx-linter/src/rules/mut_never_mutated.rs`, rewrite the rule to use semantic definitions and semantic mutation-site queries from Unit 05. Remove the manual recursion helpers completely.
9. In `unused_import.rs`, stop rediscovering import definitions by scanning top-level statements and matching spans manually. Use `model.import_definitions()` plus `model.definition(def_id)` and `model.references_to(def_id)` directly. Do not add any new semantic storage for imports in this unit.
10. In `break_outside_loop.rs`, replace loop-depth bookkeeping with control-context queries from the semantic model.
11. In `unreachable_code.rs`, `empty_match.rs`, `single_branch_par.rs`, `redundant_propagate.rs`, and `duplicate_record_field.rs`, move any scope-sensitive or control-sensitive logic onto `RuleContext` queries. Purely structural checks may still use a local AST walk if that is simpler.
12. For every rule with an autofix, populate `Diagnostic.fix` and set `Applicability` explicitly. Use:
    - `MachineApplicable` only for deterministic edits
    - `MaybeIncorrect` for edits that depend on local intent
    - `DisplayOnly` for guidance-only messages
13. Keep rule codes and rule names stable. Do not renumber existing lint codes in this unit.
14. Update linter tests to assert both diagnostics and fix applicability where a fix exists.
15. Do not reintroduce rule-local scope, import, or mutation reconstruction after `RuleContext` exists.
16. Do not add a CST or source-preserving codemod layer in this unit.

## Verification
1. Run `cargo test -p lx-linter --test lint_regressions --test pipeline_regressions`.
2. Run the checker regression tests to ensure semantic queries still behave as expected.
3. Run `just test`.
4. Inspect the registered rule list and confirm `mut_never_mutated` is now emitted through the normal registry path.

## Out Of Scope
- Semantic-model storage redesign
- Checker pass splitting
- CST or lossless syntax work
