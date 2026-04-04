# Goal

Extract AST analysis into a composable trait system.

# Why

- Decouple analysis passes from the AST
- Enable user-defined linting rules

# What Changes

**New `crates/lx-ast/src/analyzer.rs`:** Trait definition.
**Modified `crates/lx-checker/src/lib.rs`:** Implement trait.
**Modified `crates/lx-linter/src/lib.rs`:** Implement trait.
**New `crates/lx-ast/src/visitor/compose.rs`:** Composition helpers.

# Files Affected

- `crates/lx-ast/src/analyzer.rs` — New trait
- `crates/lx-checker/src/lib.rs` — Implement
- `crates/lx-linter/src/lib.rs` — Implement
- `crates/lx-ast/src/visitor/compose.rs` — New file

# Task List

### Task 1: Define Analyzer trait

Create the base `Analyzer` trait with `visit_expr`, `visit_stmt`, `report` methods.

### Task 2: Implement for checker

Refactor `lx-checker` to implement the `Analyzer` trait.

### Task 3: Implement for linter

Refactor `lx-linter` to implement the `Analyzer` trait.

### Task 4: Add composition

Create `ComposedAnalyzer` that runs multiple analyzers in sequence.

### Task 5: Integration test

Add test that composes checker + linter and runs on a sample program.
