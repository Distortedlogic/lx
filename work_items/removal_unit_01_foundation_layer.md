---
unit: 1
title: Foundation Layer Removal
scope: lx-span, lx-ast, lx-parser
depends_on: none
---

## File: crates/lx-span/src/lib.rs

### Current (line 1):
```rust
pub const PLUGIN_MANIFEST: &str = "plugin.toml";
```

### Change:
Delete line 1 entirely. The `LX_MANIFEST` constant on the next line is kept.

Result:
```rust
pub const LX_MANIFEST: &str = "lx.toml";

pub mod error;
pub mod source;
pub mod sym;
```

---

## File: crates/lx-ast/src/ast/types.rs

### Current (lines 15-21) — UseKind enum:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
  Whole,
  Alias(Sym),
  Selective(Vec<Sym>),
  Tool { command: Sym, alias: Sym },
}
```

### Change:
Remove the `Tool { command: Sym, alias: Sym }` variant (line 20). The enum becomes:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
  Whole,
  Alias(Sym),
  Selective(Vec<Sym>),
}
```

### Current (lines 102-115) — KeywordKind enum:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordKind {
  Agent,
  Tool,
  Prompt,
  Store,
  Session,
  Guard,
  Workflow,
  Schema,
  Mcp,
  Cli,
  Http,
}
```

### Change:
Remove `Mcp` (line 112) and `Cli` (line 113). The enum becomes:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordKind {
  Agent,
  Tool,
  Prompt,
  Store,
  Session,
  Guard,
  Workflow,
  Schema,
  Http,
}
```

---

## File: crates/lx-parser/src/lexer/token.rs

### Current (lines 98-99):
```rust
  McpKw,
  CliKw,
```

### Change:
Delete both lines. `HttpKw` on line 100 remains. The surrounding context becomes:
```rust
  SchemaKw,
  HttpKw,
  ChannelKw,
```

---

## File: crates/lx-parser/src/lexer/helpers.rs

### Current (lines 50-51 in type_name_or_keyword):
```rust
    "MCP" => TokenKind::McpKw,
    "CLI" => TokenKind::CliKw,
```

### Change:
Delete both match arms. The surrounding context becomes:
```rust
    "Schema" => TokenKind::SchemaKw,
    "HTTP" => TokenKind::HttpKw,
    _ => TokenKind::TypeName(lx_span::sym::intern(text)),
```

---

## File: crates/lx-parser/src/parser/stmt.rs

### Current (lines 91-101) — use_tool combinator inside use_parser:
```rust
  let arena_tool = arena.clone();
  let raw_string = just(TokenKind::StrStart).ignore_then(select! { TokenKind::StrChunk(s) => s }).then_ignore(just(TokenKind::StrEnd));

  let use_tool =
    just(TokenKind::Use).ignore_then(just(TokenKind::ToolKw)).ignore_then(raw_string).then_ignore(just(TokenKind::As)).then(name_or_type()).map_with(
      move |(command_str, alias), e| {
        let command = intern(&command_str);
        let stmt = Stmt::Use(UseStmt { path: vec![], kind: UseKind::Tool { command, alias } });
        arena_tool.borrow_mut().alloc_stmt(stmt, ss(e.span()))
      },
    );
```

### Change:
Delete lines 91-101 entirely (the `arena_tool` clone, the `raw_string` parser, and the entire `use_tool` combinator).

### Current (line 131) — the final combinator in use_parser:
```rust
  use_tool.or(use_path)
```

### Change:
Replace with just `use_path`:
```rust
  use_path
```

### Import cleanup (line 7):
```rust
use lx_ast::ast::{AstArena, BindTarget, Binding, Expr, ExprFieldAccess, FieldKind, Stmt, StmtFieldUpdate, StmtTypeDef, UseKind, UseStmt};
```
Remove `UseKind` from the import (it is no longer referenced in this file — `UseKind::Whole` and `UseKind::Selective` and `UseKind::Alias` are used in use_path via the `kind` variable which references `UseKind` by its short constructors imported on this line — **keep UseKind**, it is still used on line 116 (`UseKind::Alias`) and line 122 (`UseKind::Selective`) and line 124 (`UseKind::Whole`)).

Also remove `UseStmt` if no longer referenced — **keep UseStmt**, it is still used on line 128.

`intern` is still used on lines 105-106 (`dotdot_prefix` and `dot_prefix`). **Keep the `lx_span::sym` import as-is.**

---

## File: crates/lx-parser/src/parser/stmt_keyword.rs

### Current (lines 34-35 inside `other_kw` choice):
```rust
    just(TokenKind::McpKw).to(KeywordKind::Mcp),
    just(TokenKind::CliKw).to(KeywordKind::Cli),
```

### Change:
Delete both lines. The `other_kw` choice becomes:
```rust
  let other_kw = choice((
    just(TokenKind::AgentKw).to(KeywordKind::Agent),
    just(TokenKind::ToolKw).to(KeywordKind::Tool),
    just(TokenKind::PromptKw).to(KeywordKind::Prompt),
    just(TokenKind::StoreKw).to(KeywordKind::Store),
    just(TokenKind::SessionKw).to(KeywordKind::Session),
    just(TokenKind::GuardKw).to(KeywordKind::Guard),
    just(TokenKind::WorkflowKw).to(KeywordKind::Workflow),
    just(TokenKind::HttpKw).to(KeywordKind::Http),
  ));
```

---

## File: crates/lx-parser/src/parser/expr.rs

### Current (lines 52-53 inside keyword_as_type_name):
```rust
      TokenKind::McpKw => intern("Mcp"),
      TokenKind::CliKw => intern("Cli"),
```

### Change:
Delete both lines. The `keyword_as_type_name` select becomes:
```rust
  select! {
      TokenKind::AgentKw => intern("Agent"),
      TokenKind::ToolKw => intern("Tool"),
      TokenKind::PromptKw => intern("Prompt"),
      TokenKind::StoreKw => intern("Store"),
      TokenKind::SessionKw => intern("Session"),
      TokenKind::GuardKw => intern("Guard"),
      TokenKind::WorkflowKw => intern("Workflow"),
      TokenKind::SchemaKw => intern("Schema"),
      TokenKind::HttpKw => intern("Http"),
  }
```

---

## Downstream impacts

Every item below is a compile error caused by these removals. They are **out of scope for this unit** but must be resolved in subsequent units.

### 1. `UseKind::Tool` removal (from lx-ast)

| Crate | File | Line(s) | Error |
|---|---|---|---|
| lx-eval | `crates/lx-eval/src/interpreter/modules.rs` | 25 | `if let UseKind::Tool { command, alias } = &use_stmt.kind` — pattern match on removed variant |
| lx-eval | `crates/lx-eval/src/interpreter/modules.rs` | 85 | `UseKind::Tool { .. } => unreachable!()` — match arm for removed variant |
| lx-checker | `crates/lx-checker/src/visit_stmt.rs` | 174 | `UseKind::Tool { alias, .. } =>` — match arm for removed variant |
| lx-linter | `crates/lx-linter/src/rules/unused_import.rs` | 50 | `UseKind::Tool { alias, .. } => vec![*alias]` — match arm for removed variant |
| lx-fmt | `crates/lx-fmt/src/formatter/emit_stmt.rs` | 195 | `if let UseKind::Tool { command, alias } = &u.kind` — pattern match on removed variant |
| lx-fmt | `crates/lx-fmt/src/formatter/emit_stmt.rs` | 222 | `UseKind::Tool { .. } => unreachable!()` — match arm for removed variant |

### 2. `KeywordKind::Mcp` and `KeywordKind::Cli` removal (from lx-ast)

| Crate | File | Line(s) | Error |
|---|---|---|---|
| lx-fmt | `crates/lx-fmt/src/formatter/emit_stmt_keyword.rs` | 19-20 | `KeywordKind::Mcp => "MCP"` and `KeywordKind::Cli => "CLI"` — match arms for removed variants |
| lx-desugar | `crates/lx-desugar/src/folder/validate_core.rs` | 18-19 | `KeywordKind::Mcp` and `KeywordKind::Cli` in match pattern — match arms for removed variants |
| lx-desugar | `crates/lx-desugar/src/folder/desugar.rs` | 201-211 | `if data.keyword == KeywordKind::Mcp` and `if data.keyword == KeywordKind::Cli` — comparisons against removed variants; also calls `desugar_mcp_cli::desugar_mcp` and `desugar_mcp_cli::desugar_cli` |
| lx-desugar | `crates/lx-desugar/src/folder/desugar_mcp_cli.rs` | entire file | The `desugar_mcp` and `desugar_cli` functions exist solely to handle MCP/CLI keywords. The entire file becomes dead code once the variants are removed. |
| lx-desugar | `crates/lx-desugar/src/folder/mod.rs` | 3 | `mod desugar_mcp_cli;` — module for dead file |

### 3. `TokenKind::McpKw` and `TokenKind::CliKw` removal (from lx-parser)

No downstream impact outside lx-parser. All uses of `McpKw`/`CliKw` are within the parser crate itself (helpers.rs, stmt_keyword.rs, expr.rs) and are handled in this unit.

### 4. `PLUGIN_MANIFEST` removal (from lx-span)

| Crate | File | Line(s) | Error |
|---|---|---|---|
| lx-eval | `crates/lx-eval/src/lib.rs` | 1 | `pub use lx_span::{LX_MANIFEST, PLUGIN_MANIFEST};` — re-export of removed constant |
| lx-eval | `crates/lx-eval/src/stdlib/wasm.rs` | 46 | `crate::PLUGIN_MANIFEST` — reference to re-exported constant |
| lx-eval | `crates/lx-eval/src/interpreter/modules.rs` | 109, 115 | `crate::PLUGIN_MANIFEST` — reference to re-exported constant |
| lx-cli | `crates/lx-cli/src/plugin.rs` | 37, 230 | `lx_span::PLUGIN_MANIFEST` — direct reference to removed constant |

### 5. `.lx` program files using removed syntax

These files use `MCP` or `CLI` keywords and will fail to parse after this removal:

| File | Syntax |
|---|---|
| `tests/keywords.lx` | `CLI TestCli = { ... }` (line 49), `MCP TestServer = { ... }` (line 69) |
| `programs/brain/tools.lx` | `MCP CognitiveTools = { ... }` (line 14) |
| `pkg/git/git.lx` | `CLI +Git = { ... }` (line 4), `CLI +Gh = { ... }` (line 20) |
