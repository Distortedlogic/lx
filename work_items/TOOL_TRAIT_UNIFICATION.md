# Goal

Rename `Connector` to `Tool` and merge with the existing `Tool` trait into a single abstraction. Every capability an agent can call is a Tool with one interface: `run(args) -> Result`.

# Why

lx currently has two overlapping abstractions: `Tool` (in `std/tool.lx`: `run`/`schema`/`validate`) and `Connector` (in `std/connector.lx`: `connect`/`disconnect`/`call`/`tools`). They're the same concept — "capability an agent invokes." The `MCP`, `CLI`, `HTTP` keywords desugar to `Class : [Connector]` but should desugar to `Class : [Tool]`.

# Design Decision: Lifecycle Methods

The current Connector trait has `connect()`/`disconnect()` for session lifecycle (MCP servers need this). The merged Tool trait keeps `run` as the consumer-facing method. For MCP-backed tools, the desugared `run` wraps lifecycle internally — `connect` on first call, `call` the server, keep session alive. The consumer never calls `connect`/`disconnect` directly.

For tools that need explicit lifecycle (long-lived MCP sessions), they can add `connect`/`disconnect` as extra methods. But `run` is the universal interface every tool has.

# Exact Files That Reference Connector

Rust source (10 files):
- `crates/lx/src/lexer/token.rs:91` — `ConnectorKw` token variant
- `crates/lx/src/lexer/helpers.rs:42` — `"Connector" => TokenKind::ConnectorKw`
- `crates/lx/src/parser/stmt_keyword.rs:30` — `just(TokenKind::ConnectorKw).to(KeywordKind::Connector)`
- `crates/lx/src/parser/expr.rs:45` — `TokenKind::ConnectorKw => intern("Connector")`
- `crates/lx/src/ast/types.rs:104` — `KeywordKind::Connector` enum variant
- `crates/lx/src/folder/desugar.rs:213` — `KeywordKind::Connector => (vec!["std", "connector"], "Connector")`
- `crates/lx/src/folder/desugar_mcp_cli.rs` — `intern("Connector")` on lines 24, 129; `vec![intern("std"), intern("connector")]` on lines 25, 130
- `crates/lx/src/folder/desugar_http.rs` — same pattern as desugar_mcp_cli
- `crates/lx/src/folder/validate_core.rs:13` — `KeywordKind::Connector` in desugared assertion
- `crates/lx/src/formatter/emit_stmt.rs:191` — `KeywordKind::Connector => "Connector"`

lx source (1 file):
- `crates/lx/std/connector.lx` — the Connector trait definition

No references in `pkg/` or `programs/` — those already use `use std/connector {Connector}` or go through keyword desugaring.

# What Changes

**`crates/lx/std/tool.lx`** — merged Tool trait replaces both files:

```lx
+Trait Tool = {
  name: Str = ""
  description: Str = ""
  params: Record = {}

  run = (args) { Err "Tool.run not implemented" }
  schema = () { self.params }
  validate = (args) {
    missing = self.params | keys | filter (k) { (args | keys | contains? k) == false }
    (missing | len) == 0 ? Ok args : Err {missing: missing}
  }
}
```

**Delete** `crates/lx/std/connector.lx`.

**Lexer:** Rename `ConnectorKw` → `ToolKw` in `token.rs`. Update `helpers.rs` mapping. Note: `ToolKw` already exists in the lexer for the `Tool` keyword — so `ConnectorKw` is simply removed, and existing `ToolKw` handling stays. Verify there's no collision.

**AST:** Remove `KeywordKind::Connector` from the enum in `types.rs`. The existing `KeywordKind::Tool` stays.

**Parser:** Remove the `ConnectorKw` match in `stmt_keyword.rs`. Remove `ConnectorKw => intern("Connector")` in `expr.rs`.

**Desugarer:** In `desugar.rs`, remove the `KeywordKind::Connector` branch. In `desugar_mcp_cli.rs`, change `intern("Connector")` → `intern("Tool")` and `vec![intern("std"), intern("connector")]` → `vec![intern("std"), intern("tool")]`. Same for `desugar_http.rs`.

**MCP desugarer method generation:** Currently generates `connect`/`disconnect`/`call`/`tools`. Change to generate a single `run` method that wraps the lifecycle: auto-connect on first call if session is None, call the MCP server, return result. Keep `session` as an internal field. The generated `run` body:

```lx
run = (args) {
  self.session == None ? { self.session <- mcp.connect {command: self.command  args: self.args} ^ } : ()
  mcp.call self.session args.tool args.args ^
}
```

**CLI desugarer method generation:** Currently generates `connect`/`disconnect`/`call`/`tools`. Change to generate `run` that builds a command string and executes via subprocess. The generated `run` body:

```lx
run = (args) {
  cmd_str = self.command ++ " " ++ (args.command ?? "")
  bash cmd_str ^
}
```

**Formatter:** Change `KeywordKind::Connector => "Connector"` to remove the branch (Connector keyword no longer exists).

**validate_core.rs:** Remove `KeywordKind::Connector` from the desugared assertion list.

**`lx_std_module_source()` in `stdlib/mod.rs`:** Remove the `"connector"` entry. The `"tool"` entry already exists and now returns the merged trait.

# Task List

### Task 1: Delete Connector trait, update Tool trait
Delete `crates/lx/std/connector.lx`. Update `crates/lx/std/tool.lx` with the merged trait (add `name` field). Remove `"connector"` from `lx_std_module_source()` in `stdlib/mod.rs`.

### Task 2: Update lexer and parser
Remove `ConnectorKw` from `token.rs`, `helpers.rs`. Remove `ConnectorKw` matches from `stmt_keyword.rs` and `expr.rs`. Remove `KeywordKind::Connector` from `types.rs`.

### Task 3: Update desugarers
In `desugar.rs`: remove `KeywordKind::Connector` branch. In `desugar_mcp_cli.rs`: change `intern("Connector")` → `intern("Tool")`, change `vec![intern("std"), intern("connector")]` → `vec![intern("std"), intern("tool")]`. Rewrite generated methods: replace `connect`/`disconnect`/`call`/`tools` with a single `run` method that wraps lifecycle. Same changes in `desugar_http.rs`.

### Task 4: Update formatter and validate_core
Remove `KeywordKind::Connector` from `emit_stmt.rs` and `validate_core.rs`.

### Task 5: Update tests and verify
Update any tests that reference Connector. Run `just test`. Grep entire codebase for remaining "Connector" references and fix.

### Task 6: Update design docs
Update AGENTS.md, INVENTORY.md, STDLIB.md, EXTENSIONS.md — Connector → Tool everywhere.

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/TOOL_TRAIT_UNIFICATION.md" })
```
