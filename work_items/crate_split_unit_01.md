# Unit 1: lx-span

## Scope

Extract the foundational span/symbol types into `lx-span`. This crate has zero in-workspace dependencies and provides: the interned symbol type (`Sym`), file identifiers (`FileId`), comment types (`Comment`, `CommentStore`, `CommentPlacement`), a standalone `ParseError` type, and the two manifest constants.

## Prerequisites

None. This is the first unit.

## Concrete Steps

### Step 1: Create crate skeleton

Create directory `crates/lx-span/src/`.

Create `crates/lx-span/Cargo.toml`:

```toml
[package]
edition.workspace = true
license.workspace = true
name = "lx-span"
version = "0.1.0"

[dependencies]
lasso.workspace = true
miette.workspace = true
thiserror.workspace = true

[lints]
workspace = true
```

Dependencies rationale:
- `lasso`: used by `sym.rs` for `ThreadedRodeo` and `Spur`
- `miette`: used by `source.rs` for `SourceSpan` and by `ParseError` for `Diagnostic`
- `thiserror`: used by `ParseError` for `#[derive(Error)]`

### Step 2: Add to workspace

In `/home/entropybender/repos/lx/Cargo.toml`, add `"crates/lx-span"` to the `members` list.

Change:
```
members = ["crates/lx", "crates/lx-api", "crates/lx-cli", "crates/lx-desktop", "crates/lx-macros", "crates/lx-mobile"]
```
To:
```
members = ["crates/lx", "crates/lx-api", "crates/lx-cli", "crates/lx-desktop", "crates/lx-macros", "crates/lx-mobile", "crates/lx-span"]
```

### Step 3: Add lx-span as dependency of lx

In `/home/entropybender/repos/lx/crates/lx/Cargo.toml`, add:
```
lx-span = { path = "../lx-span" }
```
Insert after the `lx-macros` line.

### Step 4: Create `crates/lx-span/src/sym.rs`

Copy `/home/entropybender/repos/lx/crates/lx/src/sym.rs` verbatim to `crates/lx-span/src/sym.rs`. The file is self-contained -- its only external dependency is `lasso`. No changes needed to the file content.

Contents (66 lines):
- `static INTERNER: OnceLock<ThreadedRodeo>`
- `fn interner() -> &'static ThreadedRodeo`
- `pub struct Sym(Spur)` with `Debug`, `Display`, `AsRef<str>`, `PartialEq<str>`, `PartialEq<&str>`, `From<&str>`, `From<String>` impls
- `pub fn intern(s: &str) -> Sym`

### Step 5: Create `crates/lx-span/src/source.rs`

This file contains `FileId`, `Comment`, `CommentStore`, and `CommentPlacement` -- the types that do NOT depend on AST node IDs. The types that DO depend on AST IDs (`GlobalExprId`, `GlobalStmtId`, `GlobalPatternId`, `GlobalTypeExprId`, `GlobalNodeId`, `AttachedComment`, `CommentMap`, and the `NodeId::in_file` impl) stay in `lx` and will move to `lx-ast` in unit 2.

File content for `crates/lx-span/src/source.rs`:

```rust
use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u32);

impl FileId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    pub fn index(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub span: SourceSpan,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct CommentStore {
    comments: Vec<Comment>,
}

impl CommentStore {
    pub fn from_vec(comments: Vec<Comment>) -> Self {
        Self { comments }
    }

    pub fn push(&mut self, comment: Comment) {
        self.comments.push(comment);
    }

    pub fn all(&self) -> &[Comment] {
        &self.comments
    }

    pub fn comments_in_range(&self, start: usize, end: usize) -> &[Comment] {
        let lo = self.comments.partition_point(|c| c.span.offset() < start);
        let hi = self.comments.partition_point(|c| c.span.offset() < end);
        &self.comments[lo..hi]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentPlacement {
    Leading,
    Trailing,
    Dangling,
}
```

This is lines 1-17 (FileId) and lines 75-111 (Comment, CommentStore, CommentPlacement) of the original `crates/lx/src/source.rs` (the parts that don't reference `crate::ast`).

### Step 6: Create `crates/lx-span/src/error.rs`

Extract a standalone `ParseError` type. This is the `LxError::Parse` variant broken out into its own diagnostic type. The full `LxError` enum stays in `lx` (will move to `lx-value` in unit 8) because other variants reference `value::LxVal`.

File content for `crates/lx-span/src/error.rs`:

```rust
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Clone, Error, Diagnostic)]
#[error("parse error: {msg}")]
#[diagnostic(code(lx::parse))]
pub struct ParseError {
    pub msg: String,
    #[label("{msg}")]
    pub span: SourceSpan,
    #[help]
    pub help: Option<String>,
}

impl ParseError {
    pub fn new(msg: impl Into<String>, span: SourceSpan, help: Option<String>) -> Self {
        Self { msg: msg.into(), span, help }
    }
}
```

### Step 7: Create `crates/lx-span/src/lib.rs`

```rust
pub const PLUGIN_MANIFEST: &str = "plugin.toml";
pub const LX_MANIFEST: &str = "lx.toml";

pub mod error;
pub mod source;
pub mod sym;
```

### Step 8: Delete original `crates/lx/src/sym.rs`

Remove the file. It is now in `lx-span`.

### Step 9: Rewrite `crates/lx/src/source.rs`

Replace the entire file with only the AST-dependent types, importing the span types from `lx-span` via re-exports. The file now contains:

```rust
use std::collections::HashMap;

use miette::SourceSpan;

use crate::ast::{ExprId, NodeId, PatternId, StmtId, TypeExprId};

pub use lx_span::source::{Comment, CommentPlacement, CommentStore, FileId};

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

### Step 10: Update `crates/lx/src/lib.rs`

Replace the entire file with:

```rust
pub use lx_span::source as span_source;
pub use lx_span::sym;

pub const PLUGIN_MANIFEST: &str = lx_span::PLUGIN_MANIFEST;
pub const LX_MANIFEST: &str = lx_span::LX_MANIFEST;

pub mod ast;
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
pub mod visitor;
```

The key lines:
- `pub use lx_span::sym;` -- makes `lx::sym::Sym` and `lx::sym::intern` available. All `use crate::sym` in remaining `lx` modules resolves through this.
- `pub use lx_span::source as span_source;` -- internal use; `source.rs` in lx re-exports the span-level types.
- Constants are re-exported as `pub const` delegating to `lx_span`.

### Step 11: Update `crates/lx/src/error.rs`

Add a `From<lx_span::error::ParseError>` conversion so the lexer/parser can throw `ParseError` that converts to `LxError::Parse`. Add after the existing `LxError` impl block:

```rust
impl From<lx_span::error::ParseError> for LxError {
    fn from(e: lx_span::error::ParseError) -> Self {
        Self::Parse { msg: e.msg, span: e.span, help: e.help }
    }
}
```

## Import Rewrite Patterns

No files in `lx` need import changes for this unit because:
- `crate::sym` resolves through `pub use lx_span::sym` in `lib.rs`
- `crate::source::FileId` etc. resolve through the `pub use` in `source.rs`
- All 66 files that `use crate::sym` continue to work unchanged
- All 7 files that `use crate::source` continue to work unchanged

## Files Touched

| Action | Path |
|--------|------|
| CREATE | `crates/lx-span/Cargo.toml` |
| CREATE | `crates/lx-span/src/lib.rs` |
| CREATE | `crates/lx-span/src/sym.rs` |
| CREATE | `crates/lx-span/src/source.rs` |
| CREATE | `crates/lx-span/src/error.rs` |
| MODIFY | `Cargo.toml` (workspace members) |
| MODIFY | `crates/lx/Cargo.toml` (add lx-span dep) |
| DELETE | `crates/lx/src/sym.rs` |
| MODIFY | `crates/lx/src/source.rs` (keep only AST-dependent types, re-export span types) |
| MODIFY | `crates/lx/src/lib.rs` (add re-exports, remove sym module decl) |
| MODIFY | `crates/lx/src/error.rs` (add From<ParseError> impl) |

## Verification

Run `just diagnose`. Expected: zero errors, zero warnings. All existing `use crate::sym` and `use crate::source` paths in `lx` resolve through the re-exports.
