# Goal

Encapsulate arena allocation behind a clean API boundary.

# Why

- Reduce coupling between interpreter and AST arena
- Prepare for future parallel compilation

# What Changes

**Modified `crates/lx-ast/src/ast/mod.rs`:** Extract arena methods into trait.
**Modified `crates/lx-eval/src/interpreter/mod.rs`:** Use arena trait instead of direct access.

# Files Affected

- `crates/lx-ast/src/ast/mod.rs` — Arena trait extraction
- `crates/lx-eval/src/interpreter/mod.rs` — Use new trait

# Task List

### Task 1: Extract arena trait

Define `ArenaAccess` trait in `lx-ast` with `alloc_expr` and `alloc_stmt` methods.

### Task 2: Update interpreter

Replace direct `AstArena` usage with `ArenaAccess` trait in the interpreter.
