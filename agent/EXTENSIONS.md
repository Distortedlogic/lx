-- Memory: extension architecture spec. Tools + WASM plugins.
-- Implements the "thin core" principle: lx core is orchestration only, all capabilities are extensions.

# Extension Architecture

Design principle: **lx core knows how to orchestrate. Extensions know how to do things.**

## The Tool Interface

From the lx program's perspective, every capability is a Tool. One interface:

```lx
Trait Tool = {
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

Every tool call looks the same:

```lx
result <- Read.run {path: "src/main.rs"} ^
result <- Bash.run {command: "cargo test"} ^
result <- Claude.run {prompt: "analyze this"} ^
result <- WebSearch.run {query: "rust wasm plugins"} ^
result <- Grep.run {pattern: "TODO"  path: "src/"} ^
```

The lx program doesn't know or care how the tool is implemented. It calls `run`, gets a `Result` back.

### Tool Protocol Guarantees

All tools return `Result`:
- `Ok value` on success (value shape depends on tool)
- `Err {code: Int, message: Str}` on failure

Tools are **never** uncatchable. `^` propagation and `??` coalescing work on all tool results.

### Agent Wiring

Agents declare tools via `uses`:

```lx
Agent Worker = {
  uses Bash
  uses Claude
  uses Read
  uses WebFetch

  act = (task) {
    code <- Read.run {path: task.file} ^
    page <- WebFetch.run {url: task.reference} ^
    analysis <- Claude.run {prompt: "Review {code} against {page}"} ^
    analysis
  }
}
```

`uses` auto-connects the tool at agent initialization. Tool schemas are collected and passed to the LLM when the agent calls `self.think_with`.

### Tool Testing

Any tool can be mocked by providing a Tool with the same name:

```lx
MockBash = Tool {
  name: "bash"
  run = (args) { Ok {stdout: "mocked"  stderr: ""  code: 0} }
}

Agent TestWorker = {
  uses MockBash : Bash
}
```

## Tool Backings

The consumer sees one interface. The tool author picks an implementation. Four backing types:

### 1. CLI (subprocess)

Tool backed by `std::process::Command`. The simplest way to wrap any command-line program.

```lx
CLI Bash = {
  command: "bash"
  name: "bash"
  description: "Execute shell commands"
  params: {command: Str}
}

CLI Ripgrep = {
  command: "rg"
  name: "search"
  description: "Search files with ripgrep"
  params: {pattern: Str  path: Str}
}
```

The `CLI` keyword auto-generates a `run` that spawns the subprocess, captures output, and returns `Ok {stdout: Str, stderr: Str, code: Int}`. Non-zero exit codes are values, not exceptions — the lx program decides what to do.

**When to use:** Wrapping existing command-line tools. The lowest-friction option for agent-written tools. Agents reach for this first.

### 2. MCP (server protocol)

Tool backed by an MCP server over stdio. Richer than CLI: structured I/O, tool discovery, server lifecycle management.

```lx
MCP Weather = {
  server: "weather-server"
  name: "get_forecast"
  description: "Get weather forecast"
  params: {location: Str  days: Int}
}
```

The `MCP` keyword auto-generates a `run` that connects to the MCP server (spawning it if needed), calls `tools/call`, and returns the structured result.

**When to use:** When a tool server already exists. When the tool needs bidirectional state or rich structured I/O. When CLI isn't enough.

### 3. WASM

Tool backed by a WebAssembly function. Fast, sandboxed, in-process. See Part 2 for details on the WASM plugin system.

A WASM plugin can expose both functions (called via module syntax `json.parse(...)`) and tools (called via `Tool.run(...)`). The plugin manifest declares which is which.

**When to use:** Pure compute that needs to be fast. Sandboxed third-party code.

### 4. Pure lx

Tool implemented in lx code. For composition, glue, and virtual tools that combine other tools.

```lx
Tool Analyzer = {
  name: "analyze"
  description: "Search then summarize"
  params: {pattern: Str  path: Str}
  run = (args) {
    matches <- Ripgrep.run args ^
    Claude.run {prompt: "Summarize: {matches.stdout}"}
  }
}
```

**When to use:** Combining other tools into higher-level capabilities. Domain-specific workflows exposed as a single tool.

### Backing Comparison

| Backing | Implemented in | Runs | Typical latency | Sandbox |
|---------|---------------|------|-----------------|---------|
| CLI | Shell command (any language) | Subprocess | 10-100ms | OS-level |
| MCP | MCP server (any language) | Separate process | 10-100ms | Process isolation |
| WASM | Rust/C/Go/etc → .wasm | In-process (wasmtime) | 1-10ms | WASM sandbox |
| lx | lx code | In-process | Depends on body | lx runtime |

From the consumer: **all identical.** `Tool.run(args) -> Result`.

WASM is the Rust extension story. Developer writes Rust, compiles to `wasm32-unknown-unknown`, lx loads it via Extism. That's how you extend lx with Rust — not by editing the lx source and recompiling.

## Default Tools

lx ships with built-in tools that mirror what LLM coding agents expect:

| Tool | Description |
|------|-------------|
| `Bash` | Execute shell commands |
| `Read` | Read file contents |
| `Write` | Write file contents |
| `Edit` | Edit file (string replacement) |
| `Glob` | Find files by pattern |
| `Grep` | Search file contents |
| `WebSearch` | Search the web |
| `WebFetch` | Fetch URL → agent-friendly markdown |

How these are implemented internally (Rust compiled into the binary) is not the user's concern. They're tools. They have `run`. Additional tools (Claude, Git, database, etc.) are added per-project via `uses` declarations.

## What Moves Out of Core

### Becomes a Tool

| Current | Tool | Why |
|---------|------|-----|
| `llm.prompt` / `llm.prompt_with` | `Claude` (or any LLM tool) | Side effect: API call |
| `std/http` | `WebFetch` / custom HTTP tools | Side effect: network I/O |
| `std/fs` | `Read` / `Write` | Side effect: filesystem I/O |
| `$cmd` / `$^cmd` / `${}` | `Bash` | Side effect: subprocess |
| `std/git` | `Git` | Side effect: subprocess + filesystem |

### Becomes a WASM Plugin

| Current | Plugin | Why |
|---------|--------|-----|
| `std/json` | `json` | Pure compute: parsing/serialization |
| `std/re` | `regex` | Pure compute: pattern matching |
| `std/schema` | `schema` | Pure compute: validation |
| `std/md` | `markdown` | Pure compute: parsing |
| `std/math` (extended) | `math` | Pure compute: trigonometry, etc. |

### Stays in Core

Arithmetic, collections, control flow, pattern matching, pipes, closures, type system, agent spawn/messaging/channels, `par`/`sel`/`timeout`/`refine`/`meta`, `Store`, `with`/`emit`/`yield`/`assert`, module loading, the extension loading mechanisms themselves.

## Part 2: WASM Plugins

WASM plugins serve a different purpose than tools. Tools are capabilities agents invoke (`Tool.run`). WASM plugins are **compute modules** that expose functions (`module.func(args)`).

The distinction:
- `json.parse(text)` — a function call in an expression. WASM plugin.
- `Read.run({path: "file"})` — a capability invocation. Tool.

A WASM plugin CAN also register tools (declaring them in plugin.toml), but its primary interface is functions via `use`.

### Architecture: Extism

lx uses [Extism](https://github.com/extism/extism) as the WASM plugin runtime. Extism wraps wasmtime and provides:
- Simple `plugin.call::<Input, Output>("function", input)` host API
- `#[plugin_fn]` macro for plugin authors
- JSON serialization for complex types via `ToBytes`/`FromBytes`
- Host function callbacks (plugin can call back into lx runtime)
- WASI support for plugins that need filesystem/network access
- Fuel-based execution limits (prevents infinite loops)
- Plugin state persists across calls

### Plugin Structure

A WASM plugin is a Rust crate compiled to `wasm32-unknown-unknown`:

```
my-plugin/
├── Cargo.toml
├── src/
│   └── lib.rs
└── plugin.toml         -- lx plugin manifest
```

**Cargo.toml:**
```toml
[package]
name = "my-plugin"
version = "0.1.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**src/lib.rs:**
```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, FromBytes)]
#[encoding(Json)]
struct ParseInput {
    text: String,
}

#[derive(Serialize, ToBytes)]
#[encoding(Json)]
struct ParseOutput {
    data: serde_json::Value,
}

#[plugin_fn]
pub fn parse(input: ParseInput) -> FnResult<ParseOutput> {
    let data: serde_json::Value = serde_json::from_str(&input.text)
        .map_err(|e| Error::msg(e.to_string()))?;
    Ok(ParseOutput { data })
}

#[plugin_fn]
pub fn encode(input: Json<serde_json::Value>) -> FnResult<String> {
    Ok(serde_json::to_string(&input.0)?)
}
```

**plugin.toml:**
```toml
[plugin]
name = "json"
version = "0.1.0"
description = "JSON parsing and encoding"
wasm = "target/wasm32-unknown-unknown/release/my_plugin.wasm"

# Functions exposed as module members (use wasm/json → json.parse, json.encode)
[exports]
parse = { params = { text = "Str" }, returns = "Record" }
encode = { params = { value = "Any" }, returns = "Str" }

# Optional: also register as a Tool (Tool.run interface)
# [tools]
# json_validator = { description = "Validate JSON against schema", params = { text = "Str", schema = "Str" } }
```

### Plugin Loading

Plugins live in `~/.lx/plugins/` or project-local `.lx/plugins/`:

```
~/.lx/plugins/
├── json/
│   ├── plugin.toml
│   └── json.wasm
├── regex/
│   ├── plugin.toml
│   └── regex.wasm
└── schema/
    ├── plugin.toml
    └── schema.wasm
```

Loaded via `use`:

```lx
use wasm/json
use wasm/regex

data = json.parse `{"key": "value"}`
matches = regex.find_all `\d+` "abc 123 def 456"
```

### Module Resolution

The interpreter's `eval_use()` gains a new resolution path:

```rust
// In interpreter/modules.rs, inside eval_use()
if str_path.starts_with("wasm/") {
    let plugin_name = &str_path[5..];
    return self.load_wasm_plugin(plugin_name, span);
}
```

`load_wasm_plugin` does:
1. Find plugin directory (`~/.lx/plugins/{name}/` or `.lx/plugins/{name}/`)
2. Read `plugin.toml` manifest
3. Load `.wasm` file via Extism
4. Create `ModuleExports` with one `BuiltinFunc` per exported function
5. If `[tools]` section exists, also register tools in the tool registry
6. Cache the loaded plugin (don't reload on subsequent `use`)

### Host-Side Plugin Manager

```rust
// In stdlib/wasm.rs (new file)
use extism::*;
use std::collections::HashMap;
use parking_lot::RwLock;

static PLUGINS: LazyLock<RwLock<HashMap<String, Plugin>>> = LazyLock::new(Default::default);

pub fn load_plugin(name: &str, wasm_path: &Path) -> Result<ModuleExports, LxError> {
    let manifest = Manifest::new([Wasm::file(wasm_path)])
        .with_wasi(false);  // deny by default
    let plugin = Plugin::new(&manifest, [], false)
        .map_err(|e| LxError::runtime(format!("WASM load failed: {e}"), span))?;

    let manifest_toml = read_plugin_manifest(name)?;
    let mut bindings = IndexMap::new();

    for (fn_name, fn_meta) in &manifest_toml.exports {
        let plugin_name = name.to_string();
        let fn_name_owned = fn_name.to_string();
        let arity = fn_meta.params.len();

        let builtin = mk_async(&fn_name, arity, move |args, span, _ctx| {
            let pname = plugin_name.clone();
            let fname = fn_name_owned.clone();
            Box::pin(async move {
                let input = lxval_to_json(&args)?;
                let mut plugins = PLUGINS.write();
                let plugin = plugins.get_mut(&pname)
                    .ok_or_else(|| LxError::runtime("plugin not loaded", span))?;
                let result = plugin.call::<&str, &str>(&fname, &input)
                    .map_err(|e| LxError::runtime(format!("WASM call failed: {e}"), span))?;
                json_to_lxval(result)
            })
        });

        bindings.insert(sym!(fn_name), builtin);
    }

    PLUGINS.write().insert(name.to_string(), plugin);

    Ok(ModuleExports { bindings, variant_ctors: vec![] })
}
```

### Data Marshaling: lx ↔ WASM

All data crosses the WASM boundary as JSON:

| lx Type | JSON | WASM Plugin (Rust) |
|---------|------|-------------------|
| `Int` | number | `i64` or `BigInt` string |
| `Float` | number | `f64` |
| `Bool` | boolean | `bool` |
| `Str` | string | `String` |
| `List` | array | `Vec<T>` |
| `Record` | object | `HashMap<String, Value>` or struct |
| `None` | null | `Option::None` |
| `Ok v` | `{"Ok": v}` | `Result::Ok(v)` |
| `Err e` | `{"Err": e}` | `Result::Err(e)` |

JSON serialization adds ~1-5μs per call. Acceptable for compute plugins called dozens of times per workflow, not thousands per second.

### Host Functions (Plugin → lx Runtime)

Plugins can call back into the lx runtime via Extism host functions:

```rust
host_fn!(plugin_log(level: u32, msg: String) {
    match level {
        0 => log::debug!("{msg}"),
        1 => log::info!("{msg}"),
        2 => log::warn!("{msg}"),
        _ => log::error!("{msg}"),
    }
    Ok(())
});

host_fn!(plugin_get_config(key: String) -> String {
    let val = std::env::var(&key).unwrap_or_default();
    Ok(val)
});
```

### Sandboxing

WASM plugins are sandboxed by default:
- **No filesystem access** (WASI disabled unless manifest opts in)
- **No network access**
- **Fuel limits** — configurable max instructions per call (prevents infinite loops)
- **Memory limits** — configurable max WASM linear memory

Plugins that need I/O should be Tools, not WASM plugins. WASM is for pure compute.

### Plugin Installation

```bash
lx plugin install json              # from registry (future)
lx plugin install ./my-plugin       # from local path
lx plugin list                      # show installed
lx plugin remove json               # uninstall
```

### Hot Reload

WASM makes hot reload trivial:
1. Watch `.wasm` file for changes
2. Drop old `Plugin` instance
3. Create new `Plugin` from new bytes
4. Update `PLUGINS` map

No ABI concerns — WASM is a stable binary format.

## Part 3: Unified Extension Model

### How It All Fits Together

```
┌──────────────────────────────────────────────────────────┐
│                      lx program                           │
│  Agent Worker = {                                         │
│    uses Bash                                              │
│    uses Claude                                            │
│    uses Read                                              │
│    act = (task) {                                         │
│      code <- Read.run {path: task.file} ^                │
│      json.parse code | schema.validate spec ^            │
│      Claude.run {prompt: "review {code}"} ^              │
│    }                                                      │
│  }                                                        │
├──────────────────────────────────────────────────────────┤
│           Uniform Interface Layer                         │
│  Tool.run(args) -> Result    module.func(args) -> value  │
├──────────────┬──────────────┬────────────────────────────┤
│ CLI          │ MCP          │ WASM                        │
│ subprocess   │ server       │ wasmtime (Extism)           │
│              │              │                              │
│ Bash         │ Weather      │ json.parse                  │
│ Ripgrep      │ Database     │ regex.match                 │
│ curl         │ Langfuse     │ schema.validate             │
│ git          │              │ md.parse                    │
│              │              │                              │
│ 10-100ms     │ 10-100ms     │ 1-10ms                     │
│ Subprocess   │ Ext process  │ In-process, sandboxed       │
└──────────────┴──────────────┴────────────────────────────┘

Built-in tools (Read, Write, Grep, WebFetch, etc.) sit above
this layer — they're compiled into lx, not user-extensible.
```

The consumer never sees the bottom row. They see `Tool.run` and `module.func`.

### Extension Discovery

All extensions (regardless of backing) are found through a unified path:

1. **Built-in** — native Rust tools and stdlib modules compiled into `lx`
2. **Project-local** — `.lx/plugins/` and `.lx/tools/` in the project directory
3. **User-global** — `~/.lx/plugins/` and `~/.lx/tools/`
4. **Registry** — `lx plugin install` / `lx tool install` (future)

### Extension Manifest

Both tools and plugins use `plugin.toml`:

```toml
[plugin]
name = "my-extension"
version = "0.1.0"
description = "Does something useful"

# WASM plugin: expose functions as a module
[exports]
parse = { params = { text = "Str" }, returns = "Record" }

# Tool: expose capabilities via Tool.run
[tools.my_tool]
description = "Does the thing"
params = { input = "Str", count = "Int" }
backing = "wasm"  # or "cli", "mcp", "lx"

# CLI tool config (only if backing = "cli")
[tools.my_tool.cli]
command = "my-command"

# MCP tool config (only if backing = "mcp")
[tools.my_tool.mcp]
server = "my-server"
```

### Migration Path

Phase 1: **Tool trait + native backings**
- Rename `Connector` → `Tool`, merge with existing `Tool` trait
- Implement native Rust backing for built-in tools (Read, Write, Grep, Bash, WebFetch)
- CLI keyword desugars to Tool with subprocess `run`
- MCP keyword desugars to Tool with JSON-RPC `run`
- Move `llm.*` from global builtins to a Tool
- All tools return `Result` — no uncatchable errors

Phase 2: **WASM plugin infrastructure**
- Add Extism dependency
- Implement plugin loading, manifest parsing, module resolution
- Create `json` and `regex` as first WASM plugins
- `lx plugin` CLI subcommand

Phase 3: **Migrate stdlib**
- Move `std/json`, `std/re`, `std/schema`, `std/md`, `std/math` to WASM plugins
- Keep Rust implementations as fallback for environments without WASM
- Remove `$cmd` / `$^cmd` / `${}` shell syntax from parser (Bash tool replaces it)

Phase 4: **Ecosystem**
- Plugin registry
- Plugin/tool templates (`lx plugin new`, `lx tool new`)
- Documentation generation from plugin.toml
