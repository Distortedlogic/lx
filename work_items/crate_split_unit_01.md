# Unit 1: Create lx-core crate and move all core modules

## Scope

Create the `lx-core` crate and move all modules assigned to it: `sym`, `source`, `ast/`, `error`, `env`, `value/`, `visitor/`, `runtime/`, `event_stream`, `mcp_client`, `tool_module`. Set up its `Cargo.toml` with only the external deps these modules actually use. Rewrite all `use crate::X` imports within the moved files to `use crate::X` (same -- they stay intra-crate since they all land in lx-core together). Move the `PLUGIN_MANIFEST` and `LX_MANIFEST` constants to `lx-core`'s `lib.rs`. Move `STDLIB_ROOT` from `stdlib/mod.rs` to `lx-core`'s `lib.rs` as well (the checker needs it, and checker is in lx-check which depends on lx-core).

In the `lx` crate, replace every moved module with a re-export shim so that all downstream code (`lx-cli`, `lx-check`, `lx-eval`, and internal `use crate::X` within `lx`) keeps compiling unchanged. The `lx` crate's `lib.rs` changes from `pub mod sym;` to `pub use lx_core::sym;`, etc.

## Why this position

lx-core is the leaf of the dependency DAG. Every other new crate depends on it. It must exist first. Moving all lx-core modules in one unit is practical because: (a) their internal `use crate::` imports reference each other and don't change since they all land in the same crate, (b) the re-export shims in `lx` mean nothing else needs to change yet.

## Concrete steps

1. Create `crates/lx-core/Cargo.toml` with these workspace deps:
   - `lx-macros` (used by ast/)
   - `dashmap`, `futures`, `indexmap`, `la-arena`, `miette`, `num-bigint`, `num-traits`, `parking_lot`, `serde`, `serde_json`, `smallvec`, `smart-default`, `strum`, `thiserror`, `tokio`, `tokio-tungstenite`, `derive_more`, `lasso`
2. Create `crates/lx-core/src/lib.rs` declaring all moved modules plus the constants
3. Move (git mv) these directories/files from `crates/lx/src/` to `crates/lx-core/src/`:
   - `sym.rs`, `source.rs`, `error.rs`, `env.rs`, `event_stream.rs`, `mcp_client.rs`, `tool_module.rs`
   - `ast/`, `value/`, `visitor/`, `runtime/`
4. Within moved files: imports like `use crate::sym` stay unchanged (they're all in lx-core now). The only rewrite needed is removing refs to modules NOT in lx-core (there are none -- verified by dependency analysis).
5. Handle `$crate` paths in macros:
   - `record!` in `value/impls.rs` uses `$crate::sym::intern` and `$crate::value::LxVal` -- correct since both are in lx-core
   - `typed_field_methods!` and similar in `value/mod.rs` use `crate::sym::intern` -- correct
6. In `crates/lx/Cargo.toml`: add `lx-core = { path = "../lx-core" }` dep
7. Rewrite `crates/lx/src/lib.rs` to re-export everything from lx-core:
   ```rust
   pub use lx_core::ast;
   pub use lx_core::env;
   pub use lx_core::error;
   pub use lx_core::event_stream;
   pub use lx_core::mcp_client;
   pub use lx_core::runtime;
   pub use lx_core::source;
   pub use lx_core::sym;
   pub use lx_core::tool_module;
   pub use lx_core::value;
   pub use lx_core::visitor;
   pub use lx_core::{LX_MANIFEST, PLUGIN_MANIFEST, STDLIB_ROOT};
   ```
   The remaining `pub mod` declarations for modules still in `lx` (lexer, parser, folder, checker, linter, formatter, interpreter, builtins, stdlib) stay as-is.
8. In `crates/lx/src/checker/visit_stmt.rs`: change `use crate::stdlib::STDLIB_ROOT` to `use crate::STDLIB_ROOT` (since it's now re-exported at crate root)
9. Update workspace `Cargo.toml`: add `"crates/lx-core"` to members
10. Delete the now-empty source files/dirs from `crates/lx/src/` (sym.rs, source.rs, etc.)

## Key detail: pub(crate) visibility

Several items in the moved modules use `pub(crate)` visibility:
- `ast/arena.rs`: 4 fields on `AstArena` (exprs, stmts, patterns, type_exprs)
- `value/impls.rs`: 2 methods (structural_eq, hash_value)
- `lexer/mod.rs`: `Lexer` struct (but lexer stays in lx, not moving here)

The `pub(crate)` items in ast/ and value/ are only used by other lx-core modules (parser accesses arena fields but via public methods, not the fields directly -- need to verify). If any `pub(crate)` item is accessed from outside lx-core, widen it to `pub`.

## Files touched

| Action | File |
|--------|------|
| CREATE | `crates/lx-core/Cargo.toml` |
| CREATE | `crates/lx-core/src/lib.rs` |
| MOVE   | `crates/lx/src/sym.rs` -> `crates/lx-core/src/sym.rs` |
| MOVE   | `crates/lx/src/source.rs` -> `crates/lx-core/src/source.rs` |
| MOVE   | `crates/lx/src/error.rs` -> `crates/lx-core/src/error.rs` |
| MOVE   | `crates/lx/src/env.rs` -> `crates/lx-core/src/env.rs` |
| MOVE   | `crates/lx/src/event_stream.rs` -> `crates/lx-core/src/event_stream.rs` |
| MOVE   | `crates/lx/src/mcp_client.rs` -> `crates/lx-core/src/mcp_client.rs` |
| MOVE   | `crates/lx/src/tool_module.rs` -> `crates/lx-core/src/tool_module.rs` |
| MOVE   | `crates/lx/src/ast/` -> `crates/lx-core/src/ast/` |
| MOVE   | `crates/lx/src/value/` -> `crates/lx-core/src/value/` |
| MOVE   | `crates/lx/src/visitor/` -> `crates/lx-core/src/visitor/` |
| MOVE   | `crates/lx/src/runtime/` -> `crates/lx-core/src/runtime/` |
| MODIFY | `Cargo.toml` (workspace members) |
| MODIFY | `crates/lx/Cargo.toml` (add lx-core dep) |
| MODIFY | `crates/lx/src/lib.rs` (re-export shims) |
| MODIFY | `crates/lx/src/checker/visit_stmt.rs` (STDLIB_ROOT import path) |
| MODIFY | `crates/lx/src/stdlib/mod.rs` (remove STDLIB_ROOT const, keep functions) |

## Prerequisites

None.

## Verification

`just diagnose` passes. All `use crate::` and `use lx::` imports across the workspace compile unchanged.
