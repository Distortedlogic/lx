# Unit 5: lx-fmt

## Scope

Extract the formatter into `lx-fmt`. This crate depends on `lx-span` and `lx-ast`. After this unit, the `formatter/` directory is gone from `crates/lx/src/`.

## Prerequisites

Unit 4 (lx-desugar) complete.

## Concrete Steps

### Step 1: Create crate skeleton

Create directory `crates/lx-fmt/src/`.

Create `crates/lx-fmt/Cargo.toml`:

```toml
[package]
edition.workspace = true
license.workspace = true
name = "lx-fmt"
version = "0.1.0"

[dependencies]
lx-span = { path = "../lx-span" }
lx-ast = { path = "../lx-ast" }

[lints]
workspace = true
```

Dependencies rationale:
- `lx-span`: for `Sym` (used via `.as_str()` on `Sym` fields throughout emit functions)
- `lx-ast`: for all AST types (`AstArena`, `Program`, `Stmt`, `Expr`, `Pattern`, `TypeExpr`, and all sub-types)

No external crates are needed. The formatter uses only AST types, `Sym`, and `std::fmt`/`std::string`.

### Step 2: Add to workspace

In `/home/entropybender/repos/lx/Cargo.toml`, add `"crates/lx-fmt"` to the `members` list.

### Step 3: Add lx-fmt as dependency of lx

In `/home/entropybender/repos/lx/crates/lx/Cargo.toml`, add:
```
lx-fmt = { path = "../lx-fmt" }
```

### Step 4: Move `formatter/` directory

Move `crates/lx/src/formatter/` to `crates/lx-fmt/src/formatter/`.

Files moved:
- `formatter/mod.rs`
- `formatter/emit_expr.rs`
- `formatter/emit_expr_helpers.rs`
- `formatter/emit_pattern.rs`
- `formatter/emit_stmt.rs`
- `formatter/emit_type.rs`

### Step 5: Create `crates/lx-fmt/src/lib.rs`

```rust
pub mod formatter;

pub use formatter::format;
```

### Step 6: Rewrite imports in all moved formatter files

**`crates/lx-fmt/src/formatter/mod.rs`**:
- `use crate::ast::{AstArena, Program};` -> `use lx_ast::ast::{AstArena, Program};`

**`crates/lx-fmt/src/formatter/emit_stmt.rs`**:
- `use crate::ast::{BindTarget, Binding, ClassDeclData, KeywordDeclData, KeywordKind, Stmt, StmtFieldUpdate, StmtId, StmtTypeDef, TraitDeclData, TraitEntry, TraitUnionDef, UseKind, UseStmt};` -> `use lx_ast::ast::{BindTarget, Binding, ClassDeclData, KeywordDeclData, KeywordKind, Stmt, StmtFieldUpdate, StmtId, StmtTypeDef, TraitDeclData, TraitEntry, TraitUnionDef, UseKind, UseStmt};`

The type params helper `emit_type_params` uses `crate::sym::Sym` on line 290: `pub(super) fn emit_type_params(&mut self, params: &[crate::sym::Sym])`. Change to `pub(super) fn emit_type_params(&mut self, params: &[lx_span::sym::Sym])`.

**`crates/lx-fmt/src/formatter/emit_expr.rs`**:
- `use crate::ast::{BinOp, Expr, ExprBlock, ExprBreak, ExprId, ExprLoop, ExprPar, ExprPropagate, ExprTuple, FieldKind, ListElem, MapEntry, RecordField};` -> `use lx_ast::ast::{BinOp, Expr, ExprBlock, ExprBreak, ExprId, ExprLoop, ExprPar, ExprPropagate, ExprTuple, FieldKind, ListElem, MapEntry, RecordField};`

The `emit_field_access` method references `crate::ast::ExprFieldAccess` (line 98): change to `lx_ast::ast::ExprFieldAccess`.

The `emit_block` method references `crate::ast::StmtId` (line 112): change to `lx_ast::ast::StmtId`.

The `emit_func` method references `crate::ast::ExprFunc` (line 197): change to `lx_ast::ast::ExprFunc`.

The `emit_match` method references `crate::ast::ExprMatch` (line 227): change to `lx_ast::ast::ExprMatch`.

The `emit_assert` method references `crate::ast::ExprAssert` (line 254): change to `lx_ast::ast::ExprAssert`.

The `emit_sel` method references `crate::ast::SelArm` (line 263): change to `lx_ast::ast::SelArm`.

The `emit_timeout` method references `crate::ast::ExprTimeout` (line 277): change to `lx_ast::ast::ExprTimeout`.

The `emit_block_keyword` method references `crate::ast::StmtId` (line 226): change to `lx_ast::ast::StmtId`.

To simplify: add all these types to the main import statement at the top. The full import becomes:
```rust
use lx_ast::ast::{
    BinOp, Expr, ExprAssert, ExprBlock, ExprBreak, ExprFieldAccess, ExprFunc, ExprId, ExprLoop,
    ExprMatch, ExprPar, ExprPropagate, ExprTimeout, ExprTuple, FieldKind, ListElem, MapEntry,
    RecordField, SelArm, StmtId,
};
```

**`crates/lx-fmt/src/formatter/emit_expr_helpers.rs`**:
- `use crate::ast::{ExprApply, ExprBinary, ExprCoalesce, ExprId, ExprNamedArg, ExprPipe, ExprSlice, ExprTernary, ExprUnary, ExprWith, Literal, Section, StrPart, WithKind};` -> `use lx_ast::ast::{ExprApply, ExprBinary, ExprCoalesce, ExprId, ExprNamedArg, ExprPipe, ExprSlice, ExprTernary, ExprUnary, ExprWith, Literal, Section, StrPart, WithKind};`

The `emit_with` method references `crate::ast::StmtId` at line 226: add `StmtId` to the import.

**`crates/lx-fmt/src/formatter/emit_pattern.rs`**:
- `use crate::ast::{Pattern, PatternId};` -> `use lx_ast::ast::{Pattern, PatternId};`

**`crates/lx-fmt/src/formatter/emit_type.rs`**:
- `use crate::ast::{TypeExpr, TypeExprId};` -> `use lx_ast::ast::{TypeExpr, TypeExprId};`

### Step 7: Delete `crates/lx/src/formatter/`

Remove the entire directory and all files within.

### Step 8: Update `crates/lx/src/lib.rs`

Replace `pub mod formatter;` with `pub use lx_fmt::formatter;`.

Full updated file:

```rust
pub use lx_ast::ast;
pub use lx_ast::visitor;
pub use lx_desugar::folder;
pub use lx_fmt::formatter;
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
| `crate::ast::ExprFieldAccess` | `lx_ast::ast::ExprFieldAccess` |
| `crate::ast::StmtId` | `lx_ast::ast::StmtId` |
| `crate::ast::ExprFunc` | `lx_ast::ast::ExprFunc` |
| `crate::ast::ExprMatch` | `lx_ast::ast::ExprMatch` |
| `crate::ast::ExprAssert` | `lx_ast::ast::ExprAssert` |
| `crate::ast::SelArm` | `lx_ast::ast::SelArm` |
| `crate::ast::ExprTimeout` | `lx_ast::ast::ExprTimeout` |
| `crate::sym::Sym` | `lx_span::sym::Sym` |

For remaining `lx` modules that `use crate::formatter`, no changes needed -- `crate::formatter` resolves through `pub use lx_fmt::formatter`.

Files in `lx` that reference the formatter:
- `crates/lx/src/stdlib/diag/` files use the formatter to pretty-print AST -- they continue to use `crate::formatter::format` which resolves through the re-export.

## Files Touched

| Action | Path |
|--------|------|
| CREATE | `crates/lx-fmt/Cargo.toml` |
| CREATE | `crates/lx-fmt/src/lib.rs` |
| MOVE | `crates/lx/src/formatter/` -> `crates/lx-fmt/src/formatter/` (6 files) |
| MODIFY | `crates/lx-fmt/src/formatter/mod.rs` (rewrite ast import) |
| MODIFY | `crates/lx-fmt/src/formatter/emit_stmt.rs` (rewrite ast/sym imports, inline crate::ast refs) |
| MODIFY | `crates/lx-fmt/src/formatter/emit_expr.rs` (rewrite ast imports, inline crate::ast refs) |
| MODIFY | `crates/lx-fmt/src/formatter/emit_expr_helpers.rs` (rewrite ast imports) |
| MODIFY | `crates/lx-fmt/src/formatter/emit_pattern.rs` (rewrite ast import) |
| MODIFY | `crates/lx-fmt/src/formatter/emit_type.rs` (rewrite ast import) |
| MODIFY | `Cargo.toml` (workspace members) |
| MODIFY | `crates/lx/Cargo.toml` (add lx-fmt dep) |
| DELETE | `crates/lx/src/formatter/` (entire directory) |
| MODIFY | `crates/lx/src/lib.rs` (replace mod with pub use) |

## Verification

Run `just diagnose`. Expected: zero errors, zero warnings.
