---
unit: 3
title: Middle Layer Removal
scope: lx-desugar, lx-fmt, lx-checker, lx-linter
depends_on: [1]
---

## File: crates/lx-desugar/src/folder/desugar_mcp_cli.rs
### Change:
Delete entire file (138 lines). All contents are MCP/CLI desugar logic that will no longer be callable after the desugar.rs changes below.

---

## File: crates/lx-desugar/src/folder/mod.rs
### Current (line 3):
```rust
mod desugar_mcp_cli;
```
### Change:
Delete line 3 entirely. The remaining file becomes:
```rust
pub mod desugar;
mod desugar_http;
mod desugar_schema;
mod desugar_uses;
pub(crate) mod gen_ast;
mod validate_core;

pub use desugar::desugar;
```

---

## File: crates/lx-desugar/src/folder/desugar.rs
### Current (lines 201-212):
```rust
  if data.keyword == KeywordKind::Mcp {
    eprintln!(
      "warning: MCP keyword is deprecated. Use `use tool \"{}\" as {}` instead.",
      data.fields.iter().find(|f| f.name.as_str() == "command").map(|_| "<command>").unwrap_or("<command>"),
      data.name,
    );
    return super::desugar_mcp_cli::desugar_mcp(data, span, arena);
  }
  if data.keyword == KeywordKind::Cli {
    eprintln!("warning: CLI keyword is deprecated. Use `use tool` with an MCP server instead of `CLI {}`.", data.name,);
    return super::desugar_mcp_cli::desugar_cli(data, span, arena);
  }
```
### Change:
Delete lines 201-212 entirely (both `if` blocks for Mcp and Cli). Do not replace with anything. The function proceeds to the `if data.keyword == KeywordKind::Http` check on line 213 (which becomes the new line 201).

Note: `KeywordKind` is still imported and used at lines 198, 217-224, so no import changes needed. `UseKind` is still used at line 231.

---

## File: crates/lx-desugar/src/folder/validate_core.rs
### Current (lines 17-19):
```rust
        | KeywordKind::Mcp
        | KeywordKind::Cli
```
### Change:
Delete lines 18-19 (the `| KeywordKind::Mcp` and `| KeywordKind::Cli` arms). The remaining match arm list is:
```rust
        KeywordKind::Agent
        | KeywordKind::Tool
        | KeywordKind::Prompt
        | KeywordKind::Store
        | KeywordKind::Session
        | KeywordKind::Guard
        | KeywordKind::Workflow
        | KeywordKind::Schema
        | KeywordKind::Http => {
```
Note: This change depends on Unit 1 removing the `Mcp` and `Cli` variants from `KeywordKind` in lx-ast. If Unit 1 removes those variants, this exhaustive match will compile without them. If Unit 1 does NOT remove them, leaving them here causes a non-exhaustive match error. Coordinate accordingly.

---

## File: crates/lx-fmt/src/formatter/emit_stmt.rs
### Current (lines 194-198):
```rust
  fn emit_use(&mut self, u: &UseStmt) {
    if let UseKind::Tool { command, alias } = &u.kind {
      self.write(&format!("use tool \"{}\" as {}", command.as_str(), alias.as_str()));
      return;
    }
```
### Change:
Remove lines 195-198 (the `if let UseKind::Tool` early-return block). The function body starts directly with `self.write("use ");`. Result:
```rust
  fn emit_use(&mut self, u: &UseStmt) {
    self.write("use ");
```

### Current (line 222):
```rust
      UseKind::Tool { .. } => unreachable!(),
```
### Change:
Delete line 222 entirely. The `match &u.kind` block becomes:
```rust
    match &u.kind {
      UseKind::Whole => {},
      UseKind::Alias(alias) => {
        self.write(" : ");
        self.write(alias.as_str());
      },
      UseKind::Selective(names) => {
        self.write(" { ");
        for (i, n) in names.iter().enumerate() {
          if i > 0 {
            self.write("; ");
          }
          self.write(n.as_str());
        }
        self.write(" }");
      },
    }
```
Note: This depends on Unit 1 removing `UseKind::Tool` from the enum. If the variant still exists, the compiler will warn about a non-exhaustive match.

Additionally, the `UseKind` import on line 1 is still needed (used by `Whole`, `Alias`, `Selective` in the match).

---

## File: crates/lx-fmt/src/formatter/emit_stmt_keyword.rs
### Current (lines 19-20):
```rust
      KeywordKind::Mcp => "MCP",
      KeywordKind::Cli => "CLI",
```
### Change:
Delete lines 19-20 entirely. The remaining match becomes:
```rust
    let kw = match data.keyword {
      KeywordKind::Agent => "Agent",
      KeywordKind::Tool => "Tool",
      KeywordKind::Prompt => "Prompt",
      KeywordKind::Store => "Store",
      KeywordKind::Session => "Session",
      KeywordKind::Guard => "Guard",
      KeywordKind::Workflow => "Workflow",
      KeywordKind::Schema => "Schema",
      KeywordKind::Http => "HTTP",
    };
```
Note: Depends on Unit 1 removing `Mcp`/`Cli` from `KeywordKind`.

---

## File: crates/lx-checker/src/visit_stmt.rs
### Current (lines 174-177):
```rust
      UseKind::Tool { alias, .. } => {
        let def_id = self.sem.add_definition(*alias, DefKind::Import, span, false);
        self.sem.set_definition_type(def_id, unknown);
      },
```
### Change:
Delete lines 174-177 entirely. The `match &u.kind` in `resolve_use` will then end with the `UseKind::Selective` arm closing brace at line 173 (current numbering).

Note: Depends on Unit 1 removing `UseKind::Tool` from the enum. The `UseKind` import on line 3 is still needed (used by `Whole`, `Alias`, `Selective`).

---

## File: crates/lx-linter/src/rules/unused_import.rs
### Current (line 50):
```rust
        UseKind::Tool { alias, .. } => vec![*alias],
```
### Change:
Delete line 50 entirely. The match becomes:
```rust
      let names_to_check: Vec<_> = match &use_stmt.kind {
        UseKind::Whole => use_stmt.path.last().map(|n| vec![*n]).unwrap_or_default(),
        UseKind::Alias(alias) => vec![*alias],
        UseKind::Selective(names) => names.clone(),
      };
```
Note: Depends on Unit 1 removing `UseKind::Tool` from the enum. The `UseKind` import on line 4 is still needed.

---

## Downstream impacts

- **Hard dependency on Unit 1 (lx-ast):** Every file in this unit that matches on `UseKind` or `KeywordKind` will fail to compile if Unit 1 has not already removed the `Tool` variant from `UseKind` and the `Mcp`/`Cli` variants from `KeywordKind`. Unit 1 must land first.
- **lx-parser (NOT in this unit's scope):** `crates/lx-parser/src/parser/stmt.rs:98` constructs `UseKind::Tool { command, alias }`. `crates/lx-parser/src/parser/stmt_keyword.rs:34-35` maps `McpKw`/`CliKw` tokens to `KeywordKind::Mcp`/`Cli`. `crates/lx-parser/src/parser/expr.rs:52-53` maps `McpKw`/`CliKw` tokens to idents. `crates/lx-parser/src/lexer/helpers.rs:50-51` and `token.rs:98-99` define these tokens. These must be handled in a separate parser unit (presumably Unit 2) or this unit will not compile in isolation.
- **lx-eval (NOT in this unit's scope):** `crates/lx-eval/src/interpreter/modules.rs:25` and `:85` reference `UseKind::Tool`. Must be handled in a separate unit.
- **No new compile errors within Unit 3 itself** as long as Units 1 and 2 have landed. All remaining `UseKind` and `KeywordKind` imports stay valid since other variants are still in use.
