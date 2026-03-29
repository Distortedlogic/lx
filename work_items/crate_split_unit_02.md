# Unit 2: Create lx-parse crate and move lexer, parser, folder

## Scope

Create the `lx-parse` crate and move the three parsing-pipeline modules: `lexer/`, `parser/`, `folder/`. Set up its `Cargo.toml` with `lx-core` as a dependency plus the external deps these modules use. Rewrite all `use crate::X` imports within the moved files to point at `lx_core::X` for core types (sym, ast, source, error, visitor). In the `lx` crate, replace the moved modules with re-export shims.

## Why this position

lx-parse depends only on lx-core. lx-core was created in Unit 1, so lx-parse can now be extracted. lx-check and lx-eval both depend on lx-parse (lx-eval directly uses `parser::parse`, `lexer::lex`, `folder::desugar`), so this must come before those.

## Concrete steps

1. Create `crates/lx-parse/Cargo.toml` with:
   - `lx-core = { path = "../lx-core" }`
   - Workspace deps: `chumsky`, `logos`, `miette`, `num-bigint`
2. Create `crates/lx-parse/src/lib.rs` declaring `pub mod lexer;`, `pub mod parser;`, `pub mod folder;`
3. Move (git mv) from `crates/lx/src/` to `crates/lx-parse/src/`:
   - `lexer/` (5 files)
   - `parser/` (10 files)
   - `folder/` (8 files)
4. Rewrite imports in moved files (53 `use crate::` lines total):
   - `use crate::ast::*` -> `use lx_core::ast::*`
   - `use crate::sym::*` -> `use lx_core::sym::*`
   - `use crate::source::*` -> `use lx_core::source::*`
   - `use crate::error::*` -> `use lx_core::error::*`
   - `use crate::visitor::*` -> `use lx_core::visitor::*`
   - `use crate::lexer::*` -> `use crate::lexer::*` (stays same -- lexer is in lx-parse)
   - `use crate::folder::*` -> `use crate::folder::*` (stays same)
5. Handle the `Lexer` struct visibility: currently `pub(crate)` in lexer/mod.rs. Since parser (now in the same crate) uses `Token`/`TokenKind` from `lexer::token` (which are pub), and not the `Lexer` struct itself, this stays `pub(crate)` within lx-parse. If anything outside lx-parse needs it, widen to `pub`.
6. In `crates/lx/Cargo.toml`: add `lx-parse = { path = "../lx-parse" }` dep
7. In `crates/lx/src/lib.rs`: replace the remaining module declarations with re-exports:
   ```rust
   pub use lx_parse::lexer;
   pub use lx_parse::parser;
   pub use lx_parse::folder;
   ```
8. Update workspace `Cargo.toml`: add `"crates/lx-parse"` to members

## Import rewrite detail

The 53 imports break down as:

**lexer/** (2 imports):
- `use crate::error::LxError` -> `use lx_core::error::LxError`
- `use crate::sym::Sym` -> `use lx_core::sym::Sym`
- (also references `crate::source::Comment` in a non-import line -- needs checking)

**parser/** (~24 imports):
- All `use crate::ast::` -> `use lx_core::ast::`
- All `use crate::sym::` -> `use lx_core::sym::`
- All `use crate::error::` -> `use lx_core::error::`
- All `use crate::source::` -> `use lx_core::source::`
- `use crate::lexer::token::` stays as `use crate::lexer::token::`

**folder/** (~27 imports):
- All `use crate::ast::` -> `use lx_core::ast::`
- All `use crate::sym::` -> `use lx_core::sym::`
- All `use crate::visitor::` -> `use lx_core::visitor::`
- `use crate::folder::` stays as `use crate::folder::`

## Inline path references

Check for non-import `crate::` references in lexer/:
- `lexer/mod.rs` line 18: `comments: Vec<crate::source::Comment>` -- becomes `comments: Vec<lx_core::source::Comment>`

## Files touched

| Action | File |
|--------|------|
| CREATE | `crates/lx-parse/Cargo.toml` |
| CREATE | `crates/lx-parse/src/lib.rs` |
| MOVE   | `crates/lx/src/lexer/` -> `crates/lx-parse/src/lexer/` |
| MOVE   | `crates/lx/src/parser/` -> `crates/lx-parse/src/parser/` |
| MOVE   | `crates/lx/src/folder/` -> `crates/lx-parse/src/folder/` |
| MODIFY | 23 Rust files (import rewrites) |
| MODIFY | `Cargo.toml` (workspace members) |
| MODIFY | `crates/lx/Cargo.toml` (add lx-parse dep) |
| MODIFY | `crates/lx/src/lib.rs` (re-export shims) |

## Prerequisites

Unit 1 (lx-core exists).

## Verification

`just diagnose` passes. All `use crate::lexer`, `use crate::parser`, `use crate::folder`, and `use lx::lexer`, etc. compile unchanged across the workspace.
