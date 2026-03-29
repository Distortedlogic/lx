# Unit 3: lx-parser

## Scope

Extract the lexer and parser into `lx-parser`. This crate depends on `lx-span` and `lx-ast`. After this unit, the `lexer/` and `parser/` directories are gone from `crates/lx/src/`.

## Prerequisites

Unit 2 (lx-ast) complete.

## Concrete Steps

### Step 1: Create crate skeleton

Create directory `crates/lx-parser/src/`.

Create `crates/lx-parser/Cargo.toml`:

```toml
[package]
edition.workspace = true
license.workspace = true
name = "lx-parser"
version = "0.1.0"

[dependencies]
lx-span = { path = "../lx-span" }
lx-ast = { path = "../lx-ast" }
chumsky.workspace = true
logos.workspace = true
miette.workspace = true
num-bigint.workspace = true

[lints]
workspace = true
```

Dependencies rationale:
- `lx-span`: for `Sym`, `intern`, `FileId`, `Comment`, `CommentStore`, `ParseError`
- `lx-ast`: for all AST types, `AstArena`, `attach_comments`, `Program`, `Surface`
- `chumsky`: parser combinator framework, used extensively in `parser/`
- `logos`: lexer generator, used in `lexer/raw_token.rs`
- `miette`: for `SourceSpan`, `SourceOffset`
- `num-bigint`: for `BigInt` in token and lexer

### Step 2: Add to workspace

In `/home/entropybender/repos/lx/Cargo.toml`, add `"crates/lx-parser"` to the `members` list.

### Step 3: Add lx-parser as dependency of lx

In `/home/entropybender/repos/lx/crates/lx/Cargo.toml`, add:
```
lx-parser = { path = "../lx-parser" }
```

### Step 4: Move `lexer/` directory

Move `crates/lx/src/lexer/` to `crates/lx-parser/src/lexer/`.

Files moved:
- `lexer/mod.rs`
- `lexer/helpers.rs`
- `lexer/raw_token.rs`
- `lexer/strings.rs`
- `lexer/token.rs`

### Step 5: Move `parser/` directory

Move `crates/lx/src/parser/` to `crates/lx-parser/src/parser/`.

Files moved:
- `parser/mod.rs`
- `parser/expr.rs`
- `parser/expr_compound.rs`
- `parser/expr_helpers.rs`
- `parser/expr_pratt.rs`
- `parser/pattern.rs`
- `parser/stmt.rs`
- `parser/stmt_class.rs`
- `parser/stmt_keyword.rs`
- `parser/type_ann.rs`

### Step 6: Create `crates/lx-parser/src/lib.rs`

```rust
pub mod lexer;
pub mod parser;
```

### Step 7: Rewrite imports in lexer files

**`crates/lx-parser/src/lexer/mod.rs`**:
- `use crate::error::LxError;` -> `use lx_span::error::ParseError;`
- `use crate::source::Comment;` -> `use lx_span::source::Comment;`
- All `LxError::parse(...)` calls must become `ParseError::new(...)`.
- The return type of `lex()` changes from `Result<..., LxError>` to `Result<..., ParseError>`.
- The `Lexer` struct field `comments: Vec<crate::source::Comment>` -> `comments: Vec<lx_span::source::Comment>`
- The return `crate::source::CommentStore::from_vec(...)` -> `lx_span::source::CommentStore::from_vec(...)`
- `crate::source::Comment` in `dispatch` method -> `lx_span::source::Comment`

Specifically in `lexer/mod.rs`:
- Line 6: `use crate::error::LxError;` -> `use lx_span::error::ParseError;`
- Line 18: `comments: Vec<crate::source::Comment>,` -> `comments: Vec<lx_span::source::Comment>,`
- Line 24: return type `Result<(Vec<Token>, crate::source::CommentStore), LxError>` -> `Result<(Vec<Token>, lx_span::source::CommentStore), ParseError>`
- Line 28: `crate::source::CommentStore::from_vec(...)` -> `lx_span::source::CommentStore::from_vec(...)`
- In `dispatch`, line 97-100: `LxError::parse(...)` -> `ParseError::new(...)`
- In `dispatch`, line 110: `crate::source::Comment` -> `lx_span::source::Comment`
- In `dispatch`, line 145: `LxError::parse(...)` -> `ParseError::new(...)`
- In method `run`, the return type `Result<(), LxError>` -> `Result<(), ParseError>`
- In `emit_int` (helpers.rs): `LxError::parse(...)` -> `ParseError::new(...)`

**`crates/lx-parser/src/lexer/token.rs`**:
- `use crate::sym::Sym;` -> `use lx_span::sym::Sym;`

**`crates/lx-parser/src/lexer/helpers.rs`**:
- `use crate::error::LxError;` -> `use lx_span::error::ParseError;`
- `LxError::parse(...)` -> `ParseError::new(...)`
- `crate::sym::intern(text)` -> `lx_span::sym::intern(text)`
- `crate::sym::intern(slice)` -> `lx_span::sym::intern(slice)` (in `ident_or_keyword`)

**`crates/lx-parser/src/lexer/strings.rs`**:
- `use crate::error::LxError;` -> `use lx_span::error::ParseError;`
- All `LxError::parse(...)` -> `ParseError::new(...)`
- Return types `Result<..., LxError>` -> `Result<..., ParseError>`

### Step 8: Rewrite imports in parser files

**`crates/lx-parser/src/parser/mod.rs`**:
- `use crate::ast::{AstArena, BinOp, ExprId, PatternId, Program, StmtId, Surface, TypeExprId};` -> `use lx_ast::ast::{AstArena, BinOp, ExprId, PatternId, Program, StmtId, Surface, TypeExprId};`
- `use crate::error::LxError;` -> `use lx_span::error::ParseError;`
- `use crate::lexer::token::{Token, TokenKind};` -> `use crate::lexer::token::{Token, TokenKind};` (no change, `crate` is now `lx-parser`)
- `use crate::source::{CommentStore, FileId};` -> `use lx_span::source::{CommentStore, FileId};`
- In `ParseResult`, change `errors: Vec<LxError>` to `errors: Vec<ParseError>`
- In `parse_with_recovery`, the error mapping line: `LxError::parse(format!(...), ss(*e.span()), None)` -> `ParseError::new(format!(...), ss(*e.span()), None)`
- `crate::ast::attach_comments(...)` -> `lx_ast::ast::attach_comments(...)`

**`crates/lx-parser/src/parser/stmt.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::lexer::token::TokenKind;` -> `use crate::lexer::token::TokenKind;` (no change)
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`

**`crates/lx-parser/src/parser/expr.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::lexer::token::TokenKind;` -> no change
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`

**`crates/lx-parser/src/parser/expr_compound.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::lexer::token::TokenKind;` -> no change
- `use crate::sym::intern;` -> `use lx_span::sym::intern;`

**`crates/lx-parser/src/parser/expr_helpers.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::lexer::token::TokenKind;` -> no change
- `use crate::sym::intern;` -> `use lx_span::sym::intern;`

**`crates/lx-parser/src/parser/expr_pratt.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::lexer::token::TokenKind;` -> no change

**`crates/lx-parser/src/parser/pattern.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::lexer::token::TokenKind;` -> no change
- `use crate::sym::intern;` -> `use lx_span::sym::intern;`

**`crates/lx-parser/src/parser/type_ann.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::lexer::token::TokenKind;` -> no change
- `use crate::sym::{Sym, intern};` -> `use lx_span::sym::{Sym, intern};`

**`crates/lx-parser/src/parser/stmt_class.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::lexer::token::TokenKind;` -> no change
- `use crate::sym::Sym;` -> `use lx_span::sym::Sym;`

**`crates/lx-parser/src/parser/stmt_keyword.rs`**:
- `use crate::ast::{...}` -> `use lx_ast::ast::{...}`
- `use crate::lexer::token::TokenKind;` -> no change
- `use crate::sym::Sym;` -> `use lx_span::sym::Sym;`

### Step 9: Delete `crates/lx/src/lexer/` and `crates/lx/src/parser/`

Remove both directories and all files within.

### Step 10: Update `crates/lx/src/lib.rs`

Replace `pub mod lexer;` with `pub use lx_parser::lexer;`
Replace `pub mod parser;` with `pub use lx_parser::parser;`

Full updated file:

```rust
pub use lx_ast::ast;
pub use lx_ast::visitor;
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
pub mod folder;
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

### Step 11: Update remaining lx files that use lexer/parser

Check for `use crate::lexer` and `use crate::parser` in remaining `lx` source files.

**`crates/lx/src/folder/`** files: None of the folder files import from lexer or parser.

The `crate::lexer` and `crate::parser` paths resolve through `pub use lx_parser::lexer` and `pub use lx_parser::parser`.

The `crate::error::LxError` type is still in `lx`, so any code in `lx` that calls `LxError::parse(...)` is unaffected. The `From<ParseError> for LxError` impl added in unit 1 handles conversion at the boundary where `lx` code calls the parser.

### Step 12: Update error conversion at call sites

The main entry point that calls `lex()` and `parse()` is in the interpreter/runtime. Those files use `LxError`. Since the parser now returns `ParseError` instead of `LxError`, any call site that does `lexer::lex(source)?` in `lx` code will need the `?` to trigger the `From<ParseError> for LxError` conversion. This already works due to the impl added in unit 1.

Verify that `lx_parser::parser::ParseResult.errors` is now `Vec<ParseError>`. Any code in `lx` that maps these errors to `LxError` must use `.into()` or `From`. Find these sites:

Search for uses of `parser::parse` or `parser::ParseResult` in `lx` crate modules (interpreter, runtime, etc.) and ensure they convert `ParseError` -> `LxError` via `.into()` or by mapping with `LxError::from(e)`.

## Import Rewrite Patterns

| Old (in moved files) | New |
|---|---|
| `use crate::sym::Sym` | `use lx_span::sym::Sym` |
| `use crate::sym::{Sym, intern}` | `use lx_span::sym::{Sym, intern}` |
| `use crate::sym::intern` | `use lx_span::sym::intern` |
| `use crate::error::LxError` | `use lx_span::error::ParseError` |
| `LxError::parse(msg, span, help)` | `ParseError::new(msg, span, help)` |
| `use crate::ast::{...}` | `use lx_ast::ast::{...}` |
| `use crate::source::{CommentStore, FileId}` | `use lx_span::source::{CommentStore, FileId}` |
| `crate::source::Comment` | `lx_span::source::Comment` |
| `crate::source::CommentStore` | `lx_span::source::CommentStore` |
| `crate::ast::attach_comments` | `lx_ast::ast::attach_comments` |

For remaining `lx` modules (not moved), no import changes needed -- `crate::lexer` and `crate::parser` resolve through `pub use`.

## Files Touched

| Action | Path |
|--------|------|
| CREATE | `crates/lx-parser/Cargo.toml` |
| CREATE | `crates/lx-parser/src/lib.rs` |
| MOVE | `crates/lx/src/lexer/` -> `crates/lx-parser/src/lexer/` (5 files) |
| MOVE | `crates/lx/src/parser/` -> `crates/lx-parser/src/parser/` (10 files) |
| MODIFY | `crates/lx-parser/src/lexer/mod.rs` (rewrite error/source imports) |
| MODIFY | `crates/lx-parser/src/lexer/token.rs` (rewrite sym import) |
| MODIFY | `crates/lx-parser/src/lexer/helpers.rs` (rewrite error/sym imports) |
| MODIFY | `crates/lx-parser/src/lexer/strings.rs` (rewrite error import) |
| MODIFY | `crates/lx-parser/src/parser/mod.rs` (rewrite ast/error/source imports) |
| MODIFY | `crates/lx-parser/src/parser/stmt.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-parser/src/parser/expr.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-parser/src/parser/expr_compound.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-parser/src/parser/expr_helpers.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-parser/src/parser/expr_pratt.rs` (rewrite ast import) |
| MODIFY | `crates/lx-parser/src/parser/pattern.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-parser/src/parser/type_ann.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-parser/src/parser/stmt_class.rs` (rewrite ast/sym imports) |
| MODIFY | `crates/lx-parser/src/parser/stmt_keyword.rs` (rewrite ast/sym imports) |
| MODIFY | `Cargo.toml` (workspace members) |
| MODIFY | `crates/lx/Cargo.toml` (add lx-parser dep) |
| DELETE | `crates/lx/src/lexer/` (entire directory) |
| DELETE | `crates/lx/src/parser/` (entire directory) |
| MODIFY | `crates/lx/src/lib.rs` (replace mod with pub use) |

## Verification

Run `just diagnose`. Expected: zero errors, zero warnings.
