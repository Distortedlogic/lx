# Unit 4: lx-desugar

## Scope

Extract the desugaring/folder module into `lx-desugar`. This crate depends on `lx-span` and `lx-ast`. After this unit, the `folder/` directory is gone from `crates/lx/src/`.

## Prerequisites

Unit 3 (lx-parser) complete.

## Concrete Steps

### Step 1: Create crate skeleton

Create directory `crates/lx-desugar/src/`.

Create `crates/lx-desugar/Cargo.toml`:

```toml
[package]
edition.workspace = true
license.workspace = true
name = "lx-desugar"
version = "0.1.0"

[dependencies]
lx-span = { path = "../lx-span" }
lx-ast = { path = "../lx-ast" }
miette.workspace = true

[lints]
workspace = true
```

Dependencies rationale:
- `lx-span`: for `Sym`, `intern`
- `lx-ast`: for all AST types, `AstArena`, `Program`, `Surface`, `Core`, visitor/transformer traits
- `miette`: for `SourceSpan` used in gen_ast and desugar functions

No other external crates are needed. The folder code only uses `std` types, `miette::SourceSpan`, AST types, and `Sym`/`intern`.

### Step 2: Add to workspace

In `/home/entropybender/repos/lx/Cargo.toml`, add `"crates/lx-desugar"` to the `members` list.

### Step 3: Add lx-desugar as dependency of lx

In `/home/entropybender/repos/lx/crates/lx/Cargo.toml`, add:
```
lx-desugar = { path = "../lx-desugar" }
```

### Step 4: Move `folder/` directory

Move `crates/lx/src/folder/` to `crates/lx-desugar/src/folder/`.

Files moved:
- `folder/mod.rs`
- `folder/desugar.rs`
- `folder/desugar_http.rs`
- `folder/desugar_mcp_cli.rs`
- `folder/desugar_schema.rs`
- `folder/desugar_uses.rs`
- `folder/gen_ast.rs`
- `folder/validate_core.rs`

### Step 5: Create `crates/lx-desugar/src/lib.rs`

```rust
pub mod folder;

pub use folder::desugar;
```

### Step 6: Rewrite imports in all moved folder files

**`crates/lx-desugar/src/folder/mod.rs`**: No changes needed -- it only references `super::` and submodule paths.

**`crates/lx-desugar/src/folder/desugar.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`
- `use crate::visitor::transformer::AstTransformer;` -> `use lx_ast::visitor::transformer::AstTransformer;`
- `use crate::visitor::walk_transform::walk_transform_stmt;` -> `use lx_ast::visitor::walk_transform::walk_transform_stmt;`

**`crates/lx-desugar/src/folder/desugar_schema.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::sym::intern;` -> `use lx_span::sym::intern;`

**`crates/lx-desugar/src/folder/desugar_mcp_cli.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::folder::gen_ast::{...}` -> `use crate::folder::gen_ast::{...}` (no change -- `crate` now refers to `lx-desugar`, and `folder` is in `lx-desugar`)
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`

**`crates/lx-desugar/src/folder/desugar_http.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::folder::gen_ast::{...}` -> `use crate::folder::gen_ast::{...}` (no change)
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`

**`crates/lx-desugar/src/folder/desugar_uses.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::folder::gen_ast::{...}` -> `use crate::folder::gen_ast::{...}` (no change)
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`

**`crates/lx-desugar/src/folder/gen_ast.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`

**`crates/lx-desugar/src/folder/validate_core.rs`**:
- `use crate::ast::{Core, KeywordKind, WithKind};` -> `use lx_ast::ast::{Core, KeywordKind, WithKind};`
- `use crate::visitor::prelude::*;` -> `use lx_ast::visitor::prelude::*;`

### Step 7: Fix cross-module references in desugar_mcp_cli.rs and desugar_http.rs

These files reference sibling modules via `super::`:
- `super::desugar::desugar_ternary` -- still works because `super` is `folder`
- `super::desugar::desugar_coalesce` -- still works
- `super::desugar_schema::desugar_schema` -- still works
- `super::desugar_mcp_cli::desugar_mcp` -- still works
- `super::desugar_http::desugar_http` -- still works
- `super::validate_core::validate_core` -- still works
- `super::desugar_uses::generate_uses_wiring` -- still works

All `super::` references within the folder remain valid because the entire `folder/` directory moved as a unit.

### Step 8: Handle `gen_ast` visibility

In the original `crates/lx/src/folder/mod.rs`, `gen_ast` was declared as `pub(crate) mod gen_ast;`. Since it's now in `lx-desugar`, change it to `pub mod gen_ast;` so it's accessible to the other modules within the crate. Actually, checking the original: `desugar_mcp_cli.rs` and `desugar_http.rs` and `desugar_uses.rs` all import from `crate::folder::gen_ast::...`, which works because they were in the same crate. Now that they're still in the same crate (`lx-desugar`), the `pub(crate)` visibility is sufficient. No change needed.

### Step 9: Delete `crates/lx/src/folder/`

Remove the entire directory and all files within.

### Step 10: Update `crates/lx/src/lib.rs`

Replace `pub mod folder;` with `pub use lx_desugar::folder;`.

Full updated file:

```rust
pub use lx_ast::ast;
pub use lx_ast::visitor;
pub use lx_desugar::folder;
pub use lx_parser::lexer;
pub use lx_parser::parser;
pub use lx_span::sym;

pub const PLUGIN_MANIFEST: &str = lx_span::PLUGIN_MANIFEST;
pub const LX_MANIFEST: &str = lx_span::LX_MANIFEST;

pub mod builtins;
pub mod checker;
pub mod env;
pub mod error;
pub mod event_stream;
pub mod formatter;
pub mod interpreter;
pub mod linter;
pub mod mcp_client;
pub mod runtime;
pub mod source;
pub mod stdlib;
pub mod tool_module;
pub mod value;
```

## Import Rewrite Patterns

| Old (in moved files) | New |
|---|---|
| `use crate::ast::{...}` | `use lx_ast::ast::{...}` |
| `use crate::sym::{Sym, intern}` | `use lx_span::sym::{Sym, intern}` |
| `use crate::sym::intern` | `use lx_span::sym::intern` |
| `use crate::visitor::transformer::AstTransformer` | `use lx_ast::visitor::transformer::AstTransformer` |
| `use crate::visitor::walk_transform::walk_transform_stmt` | `use lx_ast::visitor::walk_transform::walk_transform_stmt` |
| `use crate::visitor::prelude::*` | `use lx_ast::visitor::prelude::*` |

For remaining `lx` modules that `use crate::folder`, no changes needed -- `crate::folder` resolves through `pub use lx_desugar::folder`.

Specific files in `lx` that import from folder:
- `crates/lx/src/interpreter/modules.rs` (if it uses `crate::folder::desugar`) -- resolves through re-export
- Any other interpreter file that calls `desugar()` -- resolves through re-export

## Files Touched

| Action | Path |
|--------|------|
| CREATE | `crates/lx-desugar/Cargo.toml` |
| CREATE | `crates/lx-desugar/src/lib.rs` |
| MOVE | `crates/lx/src/folder/` -> `crates/lx-desugar/src/folder/` (8 files) |
| MODIFY | `crates/lx-desugar/src/folder/desugar.rs` (rewrite ast/sym/visitor imports) |
| MODIFY | `crates/lx-desugar/src/folder/desugar_schema.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-desugar/src/folder/desugar_mcp_cli.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-desugar/src/folder/desugar_http.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-desugar/src/folder/desugar_uses.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-desugar/src/folder/gen_ast.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-desugar/src/folder/validate_core.rs` (rewrite ast/visitor imports) |
| MODIFY | `Cargo.toml` (workspace members) |
| MODIFY | `crates/lx/Cargo.toml` (add lx-desugar dep) |
| DELETE | `crates/lx/src/folder/` (entire directory) |
| MODIFY | `crates/lx/src/lib.rs` (replace mod with pub use) |

## Verification

Run `just diagnose`. Expected: zero errors, zero warnings.
