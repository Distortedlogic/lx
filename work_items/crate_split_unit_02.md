# Unit 2: lx-ast

## Scope

Extract the AST types, arena, comment attachment, visitor, and transformer into `lx-ast`. This crate depends on `lx-span` and `lx-macros`. After this unit, the `ast/` and `visitor/` directories are gone from `crates/lx/src/` and the remaining `source.rs` types (`Global*Id`, `AttachedComment`, `CommentMap`, `NodeId::in_file`) also move into `lx-ast`.

## Prerequisites

Unit 1 (lx-span) complete.

## Concrete Steps

### Step 1: Create crate skeleton

Create directory `crates/lx-ast/src/`.

Create `crates/lx-ast/Cargo.toml`:

```toml
[package]
edition.workspace = true
license.workspace = true
name = "lx-ast"
version = "0.1.0"

[dependencies]
lx-span = { path = "../lx-span" }
lx-macros = { path = "../lx-macros" }
la-arena.workspace = true
miette.workspace = true
num-bigint.workspace = true
smallvec.workspace = true
strum.workspace = true

[lints]
workspace = true
```

Dependencies rationale:
- `lx-span`: for `Sym`, `intern`, `FileId`, `Comment`, `CommentStore`, `CommentPlacement`
- `lx-macros`: for `#[derive(AstWalk)]`
- `la-arena`: for `Arena`, `Idx` used in `arena.rs`
- `miette`: for `SourceSpan`
- `num-bigint`: for `BigInt` used in `Literal::Int`
- `smallvec`: for `SmallVec` used in `walk_impls.rs`
- `strum`: for `#[derive(Display)]` used in `BinOp`, `UnaryOp`

### Step 2: Add to workspace

In `/home/entropybender/repos/lx/Cargo.toml`, add `"crates/lx-ast"` to the `members` list.

### Step 3: Add lx-ast as dependency of lx

In `/home/entropybender/repos/lx/crates/lx/Cargo.toml`, add:
```
lx-ast = { path = "../lx-ast" }
```

### Step 4: Move `ast/` directory

Move the entire `crates/lx/src/ast/` directory to `crates/lx-ast/src/ast/`.

Files moved:
- `ast/mod.rs`
- `ast/arena.rs`
- `ast/comment_attach.rs`
- `ast/display.rs`
- `ast/expr_types.rs`
- `ast/types.rs`
- `ast/walk_impls.rs`

### Step 5: Move `visitor/` directory

Move the entire `crates/lx/src/visitor/` directory to `crates/lx-ast/src/visitor/`.

Files moved:
- `visitor/mod.rs`
- `visitor/action.rs`
- `visitor/prelude.rs`
- `visitor/transformer.rs`
- `visitor/visitor_trait.rs`
- `visitor/walk/mod.rs`
- `visitor/walk/generated.rs`
- `visitor/walk/walk_pattern.rs`
- `visitor/walk/walk_type.rs`
- `visitor/walk_transform/mod.rs`

### Step 6: Create `crates/lx-ast/src/source.rs`

This file contains the AST-dependent types that were left behind in `crates/lx/src/source.rs` after unit 1. Move them here. These types reference AST node IDs and thus belong in `lx-ast`.

```rust
use std::collections::HashMap;

use lx_span::source::{CommentPlacement, FileId};

use crate::ast::{ExprId, NodeId, PatternId, StmtId, TypeExprId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalExprId {
    pub file: FileId,
    pub local: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalStmtId {
    pub file: FileId,
    pub local: StmtId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalPatternId {
    pub file: FileId,
    pub local: PatternId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalTypeExprId {
    pub file: FileId,
    pub local: TypeExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalNodeId {
    Expr(GlobalExprId),
    Stmt(GlobalStmtId),
    Pattern(GlobalPatternId),
    TypeExpr(GlobalTypeExprId),
}

impl GlobalExprId {
    pub fn new(file: FileId, local: ExprId) -> Self {
        Self { file, local }
    }
}

impl GlobalStmtId {
    pub fn new(file: FileId, local: StmtId) -> Self {
        Self { file, local }
    }
}

impl GlobalPatternId {
    pub fn new(file: FileId, local: PatternId) -> Self {
        Self { file, local }
    }
}

impl GlobalTypeExprId {
    pub fn new(file: FileId, local: TypeExprId) -> Self {
        Self { file, local }
    }
}

#[derive(Debug, Clone)]
pub struct AttachedComment {
    pub comment_idx: usize,
    pub placement: CommentPlacement,
}

pub type CommentMap = HashMap<NodeId, Vec<AttachedComment>>;

impl NodeId {
    pub fn in_file(self, file: FileId) -> GlobalNodeId {
        match self {
            NodeId::Expr(id) => GlobalNodeId::Expr(GlobalExprId::new(file, id)),
            NodeId::Stmt(id) => GlobalNodeId::Stmt(GlobalStmtId::new(file, id)),
            NodeId::Pattern(id) => GlobalNodeId::Pattern(GlobalPatternId::new(file, id)),
            NodeId::TypeExpr(id) => GlobalNodeId::TypeExpr(GlobalTypeExprId::new(file, id)),
        }
    }
}
```

### Step 7: Create `crates/lx-ast/src/lib.rs`

```rust
pub mod ast;
pub mod source;
pub mod visitor;
```

### Step 8: Rewrite imports in moved ast files

All `use crate::source::` and `use crate::sym::` references in the ast and visitor files must be rewritten to use `lx_span`.

**`crates/lx-ast/src/ast/mod.rs`**: Change:
- `use crate::source::{Comment, CommentMap, CommentPlacement, CommentStore, FileId};` to `use lx_span::source::{Comment, CommentPlacement, CommentStore, FileId};` and add `use crate::source::CommentMap;`
- `use crate::sym::Sym;` to `use lx_span::sym::Sym;`

**`crates/lx-ast/src/ast/comment_attach.rs`**: Change:
- `use crate::source::{AttachedComment, CommentMap, CommentPlacement, CommentStore};` to `use lx_span::source::{CommentPlacement, CommentStore};` and add `use crate::source::{AttachedComment, CommentMap};`

**`crates/lx-ast/src/ast/types.rs`**: Change:
- `use crate::sym::Sym;` to `use lx_span::sym::Sym;`

**`crates/lx-ast/src/ast/expr_types.rs`**: Change:
- `use crate::sym::Sym;` to `use lx_span::sym::Sym;`

**`crates/lx-ast/src/ast/walk_impls.rs`**: Change:
- `use crate::visitor::prelude::*;` to `use crate::visitor::prelude::*;` (no change -- `crate` now refers to `lx-ast`)
- `use crate::visitor::transformer::AstTransformer;` to `use crate::visitor::transformer::AstTransformer;` (no change)
- `use crate::visitor::walk_transform::walk_transform_expr;` to `use crate::visitor::walk_transform::walk_transform_expr;` (no change)

These are fine because `crate` in the moved file now refers to `lx-ast`, and `visitor` is at `lx-ast::visitor`.

**`crates/lx-ast/src/visitor/visitor_trait.rs`**: Change:
- `use crate::ast::{...};` -- no change needed, `crate` is `lx-ast`
- `use crate::sym::Sym;` to `use lx_span::sym::Sym;`

**`crates/lx-ast/src/visitor/prelude.rs`**: Change:
- `use crate::ast::{...};` -- no change, `crate` is `lx-ast`
- `use crate::visitor::{...};` -- no change

**`crates/lx-ast/src/visitor/walk/mod.rs`**: Change:
- `use crate::ast::{...};` -- no change

**`crates/lx-ast/src/visitor/walk/walk_pattern.rs`**: Change:
- `use crate::ast::{...};` -- no change
- `use crate::sym::Sym;` to `use lx_span::sym::Sym;`
- `use crate::visitor::{...};` -- no change

**`crates/lx-ast/src/visitor/walk/walk_type.rs`**: Change:
- `use crate::ast::{...};` -- no change
- `use crate::sym::Sym;` to `use lx_span::sym::Sym;`
- `use crate::visitor::{...};` -- no change

**`crates/lx-ast/src/visitor/walk/generated.rs`**: Change:
- `use crate::ast::{...};` -- no change

**`crates/lx-ast/src/visitor/walk_transform/mod.rs`**: Change:
- `use crate::ast::{...};` -- no change

**`crates/lx-ast/src/visitor/transformer.rs`**: Change:
- `use crate::ast::{...};` -- no change

### Step 9: Delete `crates/lx/src/ast/` and `crates/lx/src/visitor/`

Remove both directories and all files within.

### Step 10: Replace `crates/lx/src/source.rs`

The file now only re-exports everything from both `lx-span` and `lx-ast`:

```rust
pub use lx_ast::source::{
    AttachedComment, CommentMap, GlobalExprId, GlobalNodeId, GlobalPatternId, GlobalStmtId, GlobalTypeExprId,
};
pub use lx_span::source::{Comment, CommentPlacement, CommentStore, FileId};
```

### Step 11: Update `crates/lx/src/lib.rs`

Replace with:

```rust
pub use lx_ast::ast;
pub use lx_ast::visitor;
pub use lx_span::sym;

pub const PLUGIN_MANIFEST: &str = lx_span::PLUGIN_MANIFEST;
pub const LX_MANIFEST: &str = lx_span::LX_MANIFEST;

pub mod builtins;
pub mod checker;
pub mod env;
pub mod error;
pub mod event_stream;
pub mod folder;
pub mod formatter;
pub mod interpreter;
pub mod lexer;
pub mod linter;
pub mod mcp_client;
pub mod parser;
pub mod runtime;
pub mod source;
pub mod stdlib;
pub mod tool_module;
pub mod value;
```

Key changes from unit 1:
- `pub use lx_ast::ast;` replaces `pub mod ast;`
- `pub use lx_ast::visitor;` replaces `pub mod visitor;`
- `pub mod source;` remains (it's now just re-exports)
- `pub use lx_span::sym;` remains from unit 1

## Import Rewrite Patterns

Modules remaining in `lx` that imported from `ast` or `visitor` continue unchanged because `crate::ast` and `crate::visitor` resolve through `pub use lx_ast::ast` and `pub use lx_ast::visitor`.

Modules remaining in `lx` that imported from `source` continue unchanged because `crate::source` is still a module in `lx` (now a re-export module).

Example -- `crates/lx/src/folder/desugar.rs` has `use crate::ast::{...}` and `use crate::sym::{Sym, intern}` -- both still resolve through the re-exports. No change needed.

The only files that need import changes are the ones that moved into `lx-ast` (covered in step 8).

## Files Touched

| Action | Path |
|--------|------|
| CREATE | `crates/lx-ast/Cargo.toml` |
| CREATE | `crates/lx-ast/src/lib.rs` |
| CREATE | `crates/lx-ast/src/source.rs` |
| MOVE | `crates/lx/src/ast/` -> `crates/lx-ast/src/ast/` (7 files) |
| MOVE | `crates/lx/src/visitor/` -> `crates/lx-ast/src/visitor/` (10 files) |
| MODIFY | `crates/lx-ast/src/ast/mod.rs` (rewrite imports) |
| MODIFY | `crates/lx-ast/src/ast/comment_attach.rs` (rewrite imports) |
| MODIFY | `crates/lx-ast/src/ast/types.rs` (rewrite imports) |
| MODIFY | `crates/lx-ast/src/ast/expr_types.rs` (rewrite imports) |
| MODIFY | `crates/lx-ast/src/visitor/visitor_trait.rs` (rewrite Sym import) |
| MODIFY | `crates/lx-ast/src/visitor/walk/walk_pattern.rs` (rewrite Sym import) |
| MODIFY | `crates/lx-ast/src/visitor/walk/walk_type.rs` (rewrite Sym import) |
| MODIFY | `Cargo.toml` (workspace members) |
| MODIFY | `crates/lx/Cargo.toml` (add lx-ast dep) |
| DELETE | `crates/lx/src/ast/` (entire directory) |
| DELETE | `crates/lx/src/visitor/` (entire directory) |
| MODIFY | `crates/lx/src/source.rs` (replace with re-exports) |
| MODIFY | `crates/lx/src/lib.rs` (replace mod with pub use) |

## Verification

Run `just diagnose`. Expected: zero errors, zero warnings. All `use crate::ast`, `use crate::visitor`, and `use crate::source` paths in remaining `lx` modules resolve through the re-exports.
