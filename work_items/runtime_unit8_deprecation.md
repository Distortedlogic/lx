# Unit 8: MCP/CLI Keyword Removal

## Goal

Remove the `MCP` and `CLI` keywords since `use tool` replaces them. The codebase doesn't worry about backward compatibility (CLAUDE.md: "Do not worry about backward compatibility"), so this is a straight deletion, not a deprecation.

## Preconditions

- Unit 3 complete: `use tool` works as the replacement for MCP/CLI keywords
- `KeywordKind::Mcp` at `crates/lx/src/ast/types.rs:109`
- `KeywordKind::Cli` at `crates/lx/src/ast/types.rs:110`
- MCP desugaring at `crates/lx/src/folder/desugar_mcp_cli.rs`
- `McpKw` token at `crates/lx/src/lexer/token.rs:96`
- `CliKw` token at `crates/lx/src/lexer/token.rs:97`
- Keyword parser at `crates/lx/src/parser/stmt_keyword.rs:34-35` matches `McpKw` and `CliKw`
- Desugarer at `crates/lx/src/folder/desugar.rs:206-211` dispatches to `desugar_mcp` and `desugar_cli`

## Step 1: Remove tokens

File: `crates/lx/src/lexer/token.rs`

Remove `McpKw` (line 96) and `CliKw` (line 97) from the `TokenKind` enum.

## Step 2: Remove lexer keyword matching

File: `crates/lx/src/lexer/helpers.rs`

In `type_name_or_keyword()` (lines 35-52), remove the match arms for `"MCP"` and `"CLI"` that produce `TokenKind::McpKw` and `TokenKind::CliKw`. These strings will now lex as `TokenKind::TypeName(intern("MCP"))` and `TokenKind::TypeName(intern("CLI"))` — regular type names.

## Step 3: Remove KeywordKind variants

File: `crates/lx/src/ast/types.rs`

Remove `Mcp` (line 109) and `Cli` (line 110) from the `KeywordKind` enum.

## Step 4: Remove keyword parser branches

File: `crates/lx/src/parser/stmt_keyword.rs`

Remove lines 34-35:
```rust
just(TokenKind::McpKw).to(KeywordKind::Mcp),
just(TokenKind::CliKw).to(KeywordKind::Cli),
```

## Step 5: Remove desugar dispatch

File: `crates/lx/src/folder/desugar.rs`

Remove lines 206-211:
```rust
if data.keyword == KeywordKind::Mcp {
  return super::desugar_mcp_cli::desugar_mcp(data, span, arena);
}
if data.keyword == KeywordKind::Cli {
  return super::desugar_mcp_cli::desugar_cli(data, span, arena);
}
```

## Step 6: Delete desugar_mcp_cli.rs

File: `crates/lx/src/folder/desugar_mcp_cli.rs`

Delete this file entirely.

Remove the `mod desugar_mcp_cli;` declaration at `crates/lx/src/folder/mod.rs:3`.

## Step 7: Remove gen_ast helpers if only used by MCP/CLI desugar

File: `crates/lx/src/folder/gen_ast.rs`

Check if `gen_field_call`, `gen_self_field`, `gen_propagate` are used by anything other than `desugar_mcp_cli.rs`. They're also used by `desugar.rs:58` (Tell) and `desugar.rs:62` (Ask) and by `desugar_http.rs` and `desugar_uses.rs`.

So `gen_ast.rs` stays. Only `desugar_mcp_cli.rs` is deleted.

## Step 8: Update any test programs

Search for `.lx` test files that use the `MCP` or `CLI` keywords. Use ripgrep with case-insensitive pattern and broader matching:

```
rg -i '^\s*(export\s+)?(mcp|cli)\s+\w+' --type-add 'lx:*.lx' --type lx tests/ flows/
```

For each file found:
- If it's testing MCP/CLI keyword syntax, either delete the test or convert it to `use tool` syntax
- If it's using MCP/CLI as part of a larger program, convert to `use tool`

## Step 9: HTTP keyword stays

The architecture doc replaces only `MCP` and `CLI` with `use tool`. The `HTTP` keyword (`HttpKw`, `desugar_http.rs`) is unrelated and stays. No action needed.

## Step 10: Fix all compiler errors

Known match sites that need `Mcp`/`Cli` arms removed:

1. `crates/lx/src/folder/desugar.rs:206-211` — already removed in Step 5
2. `crates/lx/src/parser/stmt_keyword.rs:34-35` — already removed in Step 4
3. `crates/lx/src/lexer/helpers.rs` — already removed in Step 2
4. `crates/lx/src/ast/types.rs` — already removed in Step 3
5. `crates/lx/src/lexer/token.rs` — already removed in Step 1

Run `rg 'McpKw|CliKw|KeywordKind::Mcp|KeywordKind::Cli' --type rust crates/` to verify no references remain. Then run `just diagnose` to catch anything missed.

The `desugar.rs` file imports `UseKind` and `UseStmt` at line 8. After removing the MCP/CLI dispatch, the `KeywordKind` import is still needed for the remaining keywords (Agent, Tool, Prompt, etc.). No import cleanup needed for that line.

## Verification

1. Run `just diagnose` — no errors or warnings
2. Run `just test` — all tests pass
3. Verify that `MCP` and `CLI` are no longer recognized as keywords by writing a test that uses them as regular type names (they should lex as TypeName, not as keywords)
