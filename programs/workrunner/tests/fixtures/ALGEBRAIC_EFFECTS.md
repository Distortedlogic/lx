# Goal

Add algebraic effect handlers to the lx language runtime.

# Why

- Enable structured concurrency patterns
- Allow composable error recovery

# What Changes

**Modified `crates/lx-eval/src/interpreter/mod.rs`:** Add effect handler stack.
**New `crates/lx-eval/src/interpreter/effects.rs`:** Effect handler implementation.
**Modified `crates/lx-parser/src/parser/expr.rs`:** Parse perform/handle syntax.

# Files Affected

- `crates/lx-eval/src/interpreter/mod.rs` — Effect handler stack
- `crates/lx-eval/src/interpreter/effects.rs` — New file
- `crates/lx-parser/src/parser/expr.rs` — Parse syntax

# Task List

### Task 1: Define effect types

Create `Effect` and `EffectHandler` types in a new `effects.rs` module.

### Task 2: Add handler stack to interpreter

Add a stack of effect handlers to the `Interpreter` struct.

### Task 3: Implement perform expression

Parse and evaluate `perform` expressions that trigger effects.

### Task 4: Implement handle expression

Parse and evaluate `handle` blocks that install effect handlers.
