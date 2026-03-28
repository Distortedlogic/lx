# Work Item 12: Deprecate MCP/CLI Keywords

Add deprecation warnings when MCP or CLI keywords are encountered. Update all `.lx` example and test files to use `use tool` instead. Do NOT remove the keywords -- they continue to work, just with a warning.

## Prerequisites

- **Unit 2** (Parser) must be complete -- provides `UseKind::Tool { command, alias }` variant
- **Unit 3** (Tool Module) must be complete -- `use tool "command" as Name` spawns MCP servers and dispatches calls

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify

## Current State

### Keyword Flow

1. Lexer: `"MCP"` lexes as `TokenKind::McpKw` (`crates/lx/src/lexer/helpers.rs` line 47). `"CLI"` lexes as `TokenKind::CliKw` (line 48).
2. Parser: `keyword_parser` in `crates/lx/src/parser/stmt_keyword.rs` (lines 26-37) matches `McpKw` to `KeywordKind::Mcp` and `CliKw` to `KeywordKind::Cli`. The parser produces `Stmt::KeywordDecl(KeywordDeclData)` with `keyword: KeywordKind::Mcp` or `KeywordKind::Cli`.
3. Desugaring: `crates/lx/src/folder/desugar.rs` (lines 207-211) routes `KeywordKind::Mcp` to `desugar_mcp` and `KeywordKind::Cli` to `desugar_cli` in `crates/lx/src/folder/desugar_mcp_cli.rs`.
4. `desugar_mcp` (lines 23-46) produces: a `use std/tool {Tool}` statement + a `ClassDecl` with a `run` method that calls `mcp.connect` then `mcp.call`.
5. `desugar_cli` (lines 92-137) produces: a `use std/tool {Tool}` statement + a `ClassDecl` with a `run` method that calls `bash()`.

### Files Using MCP/CLI Keywords

- `tests/keywords.lx` -- lines 60 (`CLI TestCli`) and 80 (`MCP TestServer`)
- `pkg/git/git.lx` -- lines 4 (`CLI +Git`) and 20 (`CLI +Gh`)
- `programs/brain/tools.lx` -- line 14 (`MCP CognitiveTools`)

### Existing `use tool` syntax

`use tool "command" as Name` -- the parser produces `UseKind::Tool { command: String, alias: Sym }`. At runtime, the interpreter spawns the command as an MCP server, connects, discovers tools, and creates a module binding. Method calls on the module dispatch to MCP `tools/call`.

## Step 1: Add deprecation warning in desugaring

File: `crates/lx/src/folder/desugar.rs`

Before calling `desugar_mcp` and `desugar_cli`, emit a deprecation warning to stderr.

Current (lines 207-211):
```rust
if data.keyword == KeywordKind::Mcp {
    return super::desugar_mcp_cli::desugar_mcp(data, span, arena);
}
if data.keyword == KeywordKind::Cli {
    return super::desugar_mcp_cli::desugar_cli(data, span, arena);
}
```

Change to:
```rust
if data.keyword == KeywordKind::Mcp {
    eprintln!(
        "warning: MCP keyword is deprecated. Use `use tool \"{}\" as {}` instead.",
        data.fields.iter()
            .find(|f| f.name.as_str() == "command")
            .map(|_| "<command>")
            .unwrap_or("<command>"),
        data.name,
    );
    return super::desugar_mcp_cli::desugar_mcp(data, span, arena);
}
if data.keyword == KeywordKind::Cli {
    eprintln!(
        "warning: CLI keyword is deprecated. Use `use tool` with an MCP server instead of `CLI {}`.",
        data.name,
    );
    return super::desugar_mcp_cli::desugar_cli(data, span, arena);
}
```

This produces visible deprecation warnings at desugaring time (before execution). The desugaring still runs unchanged -- existing behavior is preserved.

## Step 2: Update tests/keywords.lx

File: `tests/keywords.lx`

The file tests MCP and CLI keyword desugaring. After this work item, these keywords still work (with warnings), but the test file should demonstrate the new `use tool` syntax.

### Locate the CLI section (around line 60):

Current:
```lx
-- CLI
CLI TestCli = {
  command: "echo"
}
```

Change to:
```lx
-- CLI (deprecated, use `use tool` instead)
CLI TestCli = {
  command: "echo"
}
```

Keep the existing `CLI` usage to verify it still works with the deprecation warning.

### Locate the MCP section (around line 80):

Current:
```lx
-- MCP
MCP TestServer = {
  command: "echo"
  args: []
}
```

Change to:
```lx
-- MCP (deprecated, use `use tool` instead)
MCP TestServer = {
  command: "echo"
  args: []
}
```

Keep the existing `MCP` usage to verify it still works with the deprecation warning.

Do NOT remove these test cases -- they verify backward compatibility. The deprecation warnings print to stderr, which does not affect test assertions.

## Step 3: Update pkg/git/git.lx

File: `pkg/git/git.lx`

Current (lines 4-5):
```lx
CLI +Git = {
  command: "git"
```

And (lines 20-21):
```lx
CLI +Gh = {
  command: "gh"
```

The `CLI` keyword desugars to a class that shells out via `bash()`. The `use tool` equivalent requires an MCP server wrapping the CLI tool. Since no such MCP server exists yet, keep the `CLI` keyword for now. The deprecation warning alerts users.

Do NOT change `pkg/git/git.lx`. The `CLI` keyword continues to work.

## Step 4: Update programs/brain/tools.lx

File: `programs/brain/tools.lx`

Current (line 14):
```lx
MCP CognitiveTools = {
  Read {path: Str} -> {content: Str}
```

The `MCP` keyword creates a class that connects to an MCP server. The `use tool` equivalent is:
```lx
use tool "cognitive-tools" as CognitiveTools
```

However, the `MCP` keyword declaration also includes tool schema definitions (`Read {path: Str} -> {content: Str}`) that are embedded in the class. `use tool` discovers tools dynamically via `tools/list`. The schema definitions in the MCP keyword form serve as documentation in the source code.

Read the full file to determine the actual command and args used:

The `MCP CognitiveTools` block contains `command` and `args` fields that specify the MCP server binary. If the command can be extracted, the `use tool` replacement is straightforward. If the block contains custom methods or schema annotations beyond the standard MCP pattern, keep the `MCP` keyword.

Do NOT change `programs/brain/tools.lx` unless the replacement is mechanically equivalent. The deprecation warning alerts users.

## Step 5: Verify deprecation output

After changes, running any `.lx` file that uses `MCP` or `CLI` keywords prints a warning to stderr:

```
warning: MCP keyword is deprecated. Use `use tool "<command>" as TestServer` instead.
warning: CLI keyword is deprecated. Use `use tool` with an MCP server instead of `CLI TestCli`.
```

The program continues to execute normally. The warning appears during desugaring (before interpretation begins).

## Step 6: Verify no keyword removal

Confirm that these files are NOT modified (the keywords must remain functional):

- `crates/lx/src/lexer/helpers.rs` -- `McpKw` and `CliKw` tokens remain
- `crates/lx/src/lexer/token.rs` -- `McpKw` and `CliKw` variants remain
- `crates/lx/src/parser/stmt_keyword.rs` -- `McpKw` and `CliKw` parser branches remain
- `crates/lx/src/folder/desugar_mcp_cli.rs` -- `desugar_mcp` and `desugar_cli` functions remain
- `crates/lx/src/folder/desugar.rs` -- routing for `KeywordKind::Mcp` and `KeywordKind::Cli` remains (with added warnings)
- `crates/lx/src/folder/validate_core.rs` -- `KeywordKind::Mcp` and `KeywordKind::Cli` validation remains

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. Run `just test` -- all existing tests pass. The deprecation warnings appear on stderr but do not affect test results (test assertions check stdout/return values, not stderr).
3. Manually run a file using `MCP` keyword -- confirm warning appears on stderr, program executes normally
4. Manually run a file using `CLI` keyword -- confirm warning appears on stderr, program executes normally
