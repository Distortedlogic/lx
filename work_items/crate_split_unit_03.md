# Unit 3: Create lx-check crate and move checker, linter, formatter

## Scope

Create the `lx-check` crate and move `checker/`, `linter/`, `formatter/`. Set up its `Cargo.toml` with `lx-core` as a dependency (NOT lx-parse -- the checker does not depend on the parser). Rewrite all `use crate::X` imports in the moved files. Replace moved modules with re-export shims in `lx`.

## Why this position

lx-check depends only on lx-core (for ast, sym, visitor, source types) and not on lx-parse. It must come before lx-eval because the linter<->checker coupling needs to land in the same crate, and we need to verify there are no hidden deps on parse or eval modules. The one known cross-boundary reference is `checker/visit_stmt.rs` using `STDLIB_ROOT`, which was moved to lx-core in Unit 1.

## Dependency analysis

**checker/** imports from:
- `ast`, `sym`, `visitor` (all in lx-core)
- `linter::{RuleRegistry, lint}` (moving together into lx-check)
- `stdlib::STDLIB_ROOT` (moved to lx-core in Unit 1)

**linter/** imports from:
- `ast`, `sym`, `visitor` (all in lx-core)
- `checker::{Diagnostic, DiagLevel, semantic::*, diagnostics::*, types::Type}` (moving together)

**formatter/** imports from:
- `ast` only (in lx-core)

The checker<->linter mutual dependency resolves naturally: both land in lx-check, so `use crate::checker` and `use crate::linter` stay as `use crate::`.

## Concrete steps

1. Create `crates/lx-check/Cargo.toml` with:
   - `lx-core = { path = "../lx-core" }`
   - Workspace deps: `ena`, `itertools`, `la-arena`, `miette`, `num-bigint`, `similar`
2. Create `crates/lx-check/src/lib.rs` declaring `pub mod checker;`, `pub mod linter;`, `pub mod formatter;`
3. Move (git mv) from `crates/lx/src/` to `crates/lx-check/src/`:
   - `checker/` (25 files)
   - `linter/` (12 files)
   - `formatter/` (6 files)
4. Rewrite imports in moved files (89 `use crate::` lines total):
   - `use crate::ast::` -> `use lx_core::ast::`
   - `use crate::sym::` -> `use lx_core::sym::`
   - `use crate::visitor::` -> `use lx_core::visitor::`
   - `use crate::source::` -> `use lx_core::source::` (if any)
   - `use crate::stdlib::STDLIB_ROOT` -> `use lx_core::STDLIB_ROOT`
   - `use crate::checker::` stays as `use crate::checker::`
   - `use crate::linter::` stays as `use crate::linter::`
5. Handle `pub(crate)` widening:
   - `checker/semantic.rs` has `pub(crate)` fields on `SemanticModel` -- used by linter (same crate now). Stays `pub(crate)`.
   - `checker/mod.rs` has `pub(crate)` field `table` on `CheckerCtx` -- only used within checker. Stays `pub(crate)`.
   - If `lx-cli` accesses any `pub(crate)` checker items via `lx::checker::`, those need widening to `pub`. Check `lx-cli` imports: it uses `checker::diagnostics::Applicability`, `checker::{CheckResult, DiagLevel, Diagnostic, check}` -- all already `pub`.
6. In `crates/lx/Cargo.toml`: add `lx-check = { path = "../lx-check" }` dep
7. In `crates/lx/src/lib.rs`: replace moved modules with re-exports:
   ```rust
   pub use lx_check::checker;
   pub use lx_check::linter;
   pub use lx_check::formatter;
   ```
8. Update workspace `Cargo.toml`: add `"crates/lx-check"` to members

## Files touched

| Action | File |
|--------|------|
| CREATE | `crates/lx-check/Cargo.toml` |
| CREATE | `crates/lx-check/src/lib.rs` |
| MOVE   | `crates/lx/src/checker/` -> `crates/lx-check/src/checker/` |
| MOVE   | `crates/lx/src/linter/` -> `crates/lx-check/src/linter/` |
| MOVE   | `crates/lx/src/formatter/` -> `crates/lx-check/src/formatter/` |
| MODIFY | 41 Rust files (import rewrites) |
| MODIFY | `Cargo.toml` (workspace members) |
| MODIFY | `crates/lx/Cargo.toml` (add lx-check dep) |
| MODIFY | `crates/lx/src/lib.rs` (re-export shims) |

## Prerequisites

Unit 1 (lx-core exists with STDLIB_ROOT).

## Verification

`just diagnose` passes. `lx-cli` check command still compiles. All checker/linter/formatter references across the workspace work through re-exports.
