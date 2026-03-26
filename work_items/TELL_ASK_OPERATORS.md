# Goal

Complete the `~>` (tell) and `~>?` (ask) agent messaging operators with tests, and fix the checker to use `unreachable!()` for all desugared surface nodes instead of `self.type_arena.todo()` / dead inference code.

# Why

The `~>` and `~>?` operators are lx's agent messaging primitives. All Rust implementation is done (lexer, parser, AST, desugarer, visitor, formatter, validate_core). Missing: tests proving they work.

Separately, the checker has dead code for desugared nodes. The validate_core.rs panics on 6 surface nodes that can't survive to Core AST: Pipe, Tell, Ask, Section, Ternary, Coalesce. The interpreter already uses `unreachable!()` for all 6. But the checker has:
- `Expr::Pipe(_) | Expr::Tell(_) | Expr::Ask(_)` → `self.type_arena.todo()` (should be unreachable)
- `Expr::Section(_)` → `self.type_arena.todo()` (should be unreachable)
- `Expr::Ternary(ternary)` → `self.synth_ternary_type(...)` (dead code — Ternary is desugared to Match before checker runs)
- `Expr::Coalesce(_)` → `self.type_arena.todo()` (should be unreachable)
- In check_expr.rs, Pipe/Tell/Ask/Section/Ternary/Coalesce are in the catch-all arm that calls `self.synth_expr`

The checker runs on Core AST. These nodes cannot exist in Core AST. The code is dead.

# What's Already Implemented

All Tell/Ask Rust code is done:

- **Lexer**: `~>` → `TildeArrow`, `~>?` → `TildeArrowQ` (raw_token.rs, token.rs, mod.rs)
- **AST**: `ExprTell { target, msg }`, `ExprAsk { target, msg }` (expr_types.rs, mod.rs)
- **Parser**: Infix at precedence 18, below Pipe's 19 (expr_pratt.rs)
- **Desugarer**: Tell → `agent.tell(target, msg)`, Ask → `agent.ask(target, msg)` (desugar.rs)
- **Visitor**: walk/dispatch functions + trait hooks (generated.rs, mod.rs, visitor_trait.rs)
- **Formatter**: Emits `target ~> msg` and `target ~>? msg` (emit_expr.rs)
- **Validate Core**: Panics if Tell/Ask survive to Core (validate_core.rs)
- **Interpreter**: `unreachable!()` (mod.rs)
- **Checker**: Tell/Ask in catch-all (check_expr.rs) and todo() (type_ops.rs)

# Files Affected

- `crates/lx/src/checker/type_ops.rs` — Pipe/Tell/Ask/Section/Ternary/Coalesce → `unreachable!()`
- `crates/lx/src/checker/check_expr.rs` — Move Pipe/Tell/Ask/Section/Ternary/Coalesce out of catch-all → `unreachable!()`
- `tests/tell_ask.lx` — New test file

# Task List

### Task 1: Fix checker for desugared nodes

**Subject:** Change all desugared surface nodes from todo()/dead code to unreachable!() in checker

**Description:**

**File: `crates/lx/src/checker/type_ops.rs`**

Line 46 — change:
```rust
Expr::Pipe(_) | Expr::Tell(_) | Expr::Ask(_) => self.type_arena.todo(),
```
to:
```rust
Expr::Pipe(_) | Expr::Tell(_) | Expr::Ask(_) => unreachable!(),
```

Line 48 — change:
```rust
Expr::Section(_) => self.type_arena.todo(),
```
to:
```rust
Expr::Section(_) => unreachable!(),
```

Line 86 — change:
```rust
Expr::Ternary(ternary) => self.synth_ternary_type(ternary.cond, ternary.then_, ternary.else_),
```
to:
```rust
Expr::Ternary(_) => unreachable!(),
```

Line 88 — change:
```rust
Expr::Coalesce(_) => self.type_arena.todo(),
```
to:
```rust
Expr::Coalesce(_) => unreachable!(),
```

Then delete the `synth_ternary_type` method — it's dead code (Ternary is desugared to Match before the checker runs, the interpreter already has `Expr::Ternary(_) => unreachable!()`). Find the method in type_ops.rs or a related checker file and remove it entirely.

**File: `crates/lx/src/checker/check_expr.rs`**

The catch-all arm starting at line 47 includes `Expr::Pipe(_)`, `Expr::Tell(_)`, `Expr::Ask(_)`, `Expr::Section(_)`, `Expr::Ternary(_)`, and `Expr::Coalesce(_)`. Remove all 6 from the catch-all arm. Add a separate arm above the catch-all:
```rust
Expr::Pipe(_) | Expr::Tell(_) | Expr::Ask(_) | Expr::Section(_) | Expr::Ternary(_) | Expr::Coalesce(_) => unreachable!(),
```

These 6 nodes match exactly the interpreter's unreachable set (mod.rs lines 136, 150, 159, 171) and the validate_core.rs panic set (lines 31-36).

**ActiveForm:** Fixing checker for desugared nodes

---

### Task 2: Write tell/ask test

**Subject:** Add test verifying ~> and ~>? operators parse and desugar

**Description:**

Create `tests/tell_ask.lx`:

```lx
-- tell/ask operator tests
-- verifies ~> and ~>? desugar to agent.tell/agent.ask

assert (type_of agent.tell == "Func")
assert (type_of agent.ask == "Func")

f = (target msg) { try agent.tell target msg }
g = (target msg) { try agent.ask target msg }
assert (type_of f == "Func")
assert (type_of g == "Func")
```

The operators `~>` and `~>?` desugar to `agent.tell target msg` and `agent.ask target msg`. Full end-to-end agent messaging requires a running agent subprocess. The test verifies: (1) the builtins exist and are Func, (2) functions wrapping the calls parse and compile. The formatter roundtrip test will cover parse→format→parse stability.

**ActiveForm:** Writing tell/ask test

---

### Task 3: Run full test suite

**Subject:** Verify no regressions

**Description:**

1. Run `just rust-diagnose` — must be 0 errors, 0 warnings. If `synth_ternary_type` removal causes unused import warnings or missing method errors, fix them.
2. Run `just test` — all tests must pass including the new `tell_ask.lx`.
3. Run `cargo test -p lx --test formatter_roundtrip` — must pass.

If any test triggers `unreachable!()` in the checker, it means the desugaring pipeline has a bug — a surface node survived to Core. Investigate the desugarer, don't revert to `todo()`.

**ActiveForm:** Running full test suite

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**
5. **Do not add code comments or doc strings.**
6. **Use `just rust-diagnose` not raw cargo commands** (exception: `cargo test -p lx --test formatter_roundtrip`).
7. **The 6 desugared nodes are: Pipe, Tell, Ask, Section, Ternary, Coalesce.** All 6 appear in validate_core.rs panic list and interpreter unreachable set. The checker must match.
8. **`synth_ternary_type` is dead code.** Ternary desugars to Match. The checker only sees Core AST where Ternary cannot exist. Delete the method and any helper it calls that has no other callers.
9. **Do NOT touch `todo()` calls for non-desugared nodes** (FieldAccess, With, Yield, etc.). Those represent genuinely unfinished type inference, not desugaring placeholders.

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/TELL_ASK_OPERATORS.md" })
```
