-- Memory: extension architecture spec. Two extension mechanisms: Tools and WASM plugins.
-- Implements the "thin core" principle: lx core is orchestration only, all capabilities are extensions.

# Extension Architecture

lx has two extension mechanisms for two different purposes:

- **Tools** — I/O and side effects (LLM, HTTP, filesystem, shell, databases). The Tool trait is the universal interface. Multiple backing implementations: CLI (subprocess), MCP (protocol), HTTP (REST), native Rust, or pure lx. Different scenarios call for different backings — an agent reaches for CLI first, MCP when richer integration is needed.
- **WASM plugins** — fast pure compute (JSON parsing, regex, schema validation, hashing). Compiled to WebAssembly, loaded by the lx runtime. Sandboxed by default.

Design principle: **lx core knows how to orchestrate. Extensions know how to do things.**

## What Moves Out of Core

### Becomes a Tool

| Current | Tool Name | Why |
|---------|-----------|-----|
| `llm.prompt` / `llm.prompt_with` | `Claude` (or any LLM) | Side effect: API call to external service |
| `std/http` | `Http` | Side effect: network I/O |
| `std/fs` | `Fs` | Side effect: filesystem I/O |
| `$cmd` / `$^cmd` / `${}` | `Bash` | Side effect: subprocess execution |
| `std/git` | `Git` | Side effect: subprocess + filesystem |

### Becomes a WASM Plugin

| Current | Plugin Name | Why |
|---------|-------------|-----|
| `std/json` (json.parse, json.encode) | `json` | Pure compute: parsing/serialization |
| `std/re` | `regex` | Pure compute: pattern matching |
| `std/schema` | `schema` | Pure compute: validation |
| `std/md` | `markdown` | Pure compute: parsing |
| `std/math` (extended) | `math` | Pure compute: trigonometry, etc. |

### Stays in Core

Arithmetic, collections, control flow, pattern matching, pipes, closures, type system, agent spawn/messaging/channels, `par`/`sel`/`timeout`/`refine`/`meta`, `Store`, `with`/`emit`/`yield`/`assert`, module loading, the extension loading mechanisms themselves.

## Part 1: Tools

### Connector → Tool Rename

The current `Connector` trait is renamed to `Tool`. The current `Tool` trait (which only defines schema/validate) merges into it. One abstraction: "thing an agent can call."

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

### Tool Backings

`Tool` is the interface. How it runs is a detail. The `MCP`, `CLI`, `HTTP` keywords each provide a different default `run` implementation:

```lx
-- CLI tool: backed by std::process::Command
-- Simplest. Agent writes a command, gets stdout/stderr/code back.
-- This is the default reach — agents write CLI tools first.
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

-- HTTP tool: backed by generic HTTP requests (reqwest)
-- For any URL. Agent specifies base_url, headers, tool builds requests.
-- This is plain HTTP, not MCP-over-HTTP.
HTTP GitHubAPI = {
  base_url: "https://api.github.com"
  headers: {accept: "application/vnd.github.v3+json"}
  name: "github"
  description: "GitHub REST API"
  params: {method: Str = "GET"  path: Str  body: Record = {:}}
}

-- MCP tool: backed by MCP server over stdio
-- Richer integration: tool discovery, structured I/O, server lifecycle.
-- Used when a tool server already exists or needs bidirectional state.
MCP Weather = {
  server: "weather-server"
  name: "get_forecast"
  description: "Get weather forecast"
  params: {location: Str  days: Int}
}

-- Pure lx tool: backed by lx code
-- For composed/virtual tools that combine other tools.
Tool Analyzer = {
  name: "analyze"
  description: "Run ripgrep then summarize"
  params: {pattern: Str  path: Str}
  run = (args) {
    matches <- Ripgrep.run args ^
    Claude.run {prompt: "Summarize: {matches.stdout}"}
  }
}
```

Each backing has a Rust implementation that provides the default `run`:

| Keyword | Backing | Default `run` | Returns |
|---------|---------|---------------|---------|
| `CLI` | `std::process::Command` | Spawns subprocess, captures output | `{stdout: Str, stderr: Str, code: Int}` |
| `HTTP` | `reqwest` | Generic HTTP request to any URL | `{status: Int, body: Str, headers: Record}` |
| `MCP` | MCP client (JSON-RPC over stdio) | Calls MCP server's `tools/call` | Tool-defined structured output |
| `Tool` | lx code | User-defined `run` method | Whatever `run` returns |

An agent writing lx code picks the backing that fits:
- **CLI** for anything that has a command-line interface (most things)
- **HTTP** for REST APIs
- **MCP** when a tool server exists with rich capabilities
- **Tool** (pure lx) for composition and glue

### Tool Discovery and Wiring

Agents declare tools via `uses`:

```lx
Agent Worker = {
  uses Bash
  uses Claude
  uses Weather

  act = (task) {
    files <- Bash.run {command: "ls src/"} ^
    forecast <- Weather.run {location: "SF"  days: 3} ^
    analysis <- Claude.run {prompt: "Analyze: {files.stdout}"} ^
    analysis
  }
}
```

`uses` auto-connects the tool at agent initialization and makes it available as a bound name in the agent's scope. Tool schemas are collected and passed to the LLM when the agent calls `self.think_with`.

### Rust Backing Implementations

Each keyword gets a Rust-backed default `run`. These live in `crates/lx/src/stdlib/tools/`:

**CLI** (`tools/cli.rs`):
```rust
fn bi_cli_run(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let command = args[0].require_str("command", span)?;
    let tool_args = args[1]; // Record of named args
    let output = Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| LxError::runtime(format!("CLI failed: {e}"), span))?;
    Ok(LxVal::ok(LxVal::record(indexmap! {
        sym!("stdout") => LxVal::str(String::from_utf8_lossy(&output.stdout)),
        sym!("stderr") => LxVal::str(String::from_utf8_lossy(&output.stderr)),
        sym!("code")   => LxVal::int(output.status.code().unwrap_or(-1)),
    })))
}
```

**HTTP** (`tools/http.rs`): generic HTTP via `reqwest`. Builds request from `base_url` + `path` + `method` + `headers` + `body`. Returns `{status, body, headers}`. Not MCP — plain HTTP to any URL.

**MCP** (`tools/mcp.rs`): JSON-RPC client over stdio. Handles server lifecycle (spawn, connect, call, close) and connection pooling.

### LLM as a Tool

`llm.prompt` is no longer a global builtin — it's a tool that agents explicitly `uses`. The backing can be CLI (calling `claude` CLI), HTTP (calling the API directly), or MCP (connecting to a model server):

```lx
-- Simplest: CLI backing, calls the claude CLI
CLI Claude = {
  command: "claude"
  name: "prompt"
  description: "Send prompt to Claude"
  params: {prompt: Str  json_schema: Str = ""  max_turns: Int = 1}
}

-- Or: HTTP backing, calls the API directly
HTTP Claude = {
  base_url: "https://api.anthropic.com"
  name: "prompt"
  description: "Send prompt to Claude"
  params: {prompt: Str  model: Str = "claude-sonnet-4-20250514"}
}

Agent Analyst = {
  uses Claude

  think = (prompt) {
    Claude.run {prompt: prompt} ^
  }
}
```

### Tool Protocol Guarantees

All tools return `Result`:
- `Ok value` on success (value shape depends on tool and backing)
- `Err {code: Int, message: Str}` on failure

Tools are **never** uncatchable. `^` propagation and `??` coalescing work on all tool results. This fixes the `$^` bug by design — there is no `$^`, there's `Bash.run` which returns a Result.

### Tool Testing

Tools can be mocked for testing:

```lx
MockBash = Tool {
  name: "bash"
  run = (args) { Ok {stdout: "mocked output"  stderr: ""  code: 0} }
}

Agent TestWorker = {
  uses MockBash : Bash    -- alias mock as Bash
  -- rest of agent code unchanged
}
```

## Part 2: WASM Plugins

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

[exports]
parse = { params = { text = "Str" }, returns = "Record" }
encode = { params = { value = "Any" }, returns = "Str" }
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
use wasm/json              -- loads ~/.lx/plugins/json/
use wasm/regex             -- loads ~/.lx/plugins/regex/

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
5. Cache the loaded plugin (don't reload on subsequent `use`)

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

    // Discover exports from plugin.toml
    let manifest_toml = read_plugin_manifest(name)?;
    let mut bindings = IndexMap::new();

    for (fn_name, fn_meta) in &manifest_toml.exports {
        let plugin_name = name.to_string();
        let fn_name_owned = fn_name.to_string();
        let arity = fn_meta.params.len();

        // Create a builtin that calls into the WASM plugin
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
// Host-side: register callbacks available to all WASM plugins
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

Plugin-side:
```rust
#[host_fn("extism:host/user")]
extern "ExtismHost" {
    fn plugin_log(level: u32, msg: String);
    fn plugin_get_config(key: String) -> String;
}
```

### Sandboxing

WASM plugins are sandboxed by default:
- **No filesystem access** (WASI disabled unless manifest opts in)
- **No network access**
- **Fuel limits** — configurable max instructions per call (prevents infinite loops)
- **Memory limits** — configurable max WASM linear memory

Plugins that need I/O should be Tools (CLI/HTTP/MCP), not WASM plugins. WASM is for pure compute.

### Plugin Installation

```bash
lx plugin install json              # from registry (future)
lx plugin install ./my-plugin       # from local path
lx plugin list                      # show installed
lx plugin remove json               # uninstall
```

Or manually: copy the plugin directory to `~/.lx/plugins/`.

### Hot Reload

WASM makes hot reload trivial:
1. Watch `.wasm` file for changes
2. Drop old `Plugin` instance
3. Create new `Plugin` from new bytes
4. Update `PLUGINS` map

No ABI concerns — WASM is a stable binary format.

## Part 3: Integration — How Tools and WASM Work Together

### Agent Using Both

```lx
use wasm/json
use wasm/schema

Agent Analyst : [Agent] = {
  uses Claude
  uses Bash
  uses Http

  act = (task) {
    -- WASM: pure compute (fast, in-process)
    input = json.parse (task.data)
    valid = schema.validate task_schema input

    valid ? {
      Err e -> Err "Invalid input: {e}"
      Ok _ -> {
        -- Tool: I/O (MCP/CLI, out-of-process)
        raw <- Http.run {url: task.source  method: "GET"} ^
        code <- Bash.run {command: "wc -l src/*.rs"} ^
        analysis <- Claude.run {
          prompt: "Analyze {raw.body} against {code.stdout}"
        } ^

        -- WASM: pure compute on the result
        json.encode {result: analysis  input: input}
      }
    }
  }
}
```

### The Clean Split

```
┌──────────────────────────────────────────────────┐
│                   lx program                      │
│  agents, pipes, pattern matching, control flow    │
│  par/sel/timeout/refine/meta, channels, stores    │
├────────────────────┬─────────────────────────────┤
│   WASM plugins     │         Tools                │
│   (in-process)     │     (out-of-process)         │
│                    │                               │
│   json.parse       │   CLI: Bash.run (shell)      │
│   regex.match      │   CLI: Git.run (vcs)         │
│   schema.validate  │   HTTP: Claude.run (LLM)     │
│   md.parse         │   HTTP: Api.run (REST)       │
│   math.sin         │   MCP: Weather.run (server)  │
│                    │   lx: Composed.run (glue)    │
│   Pure compute     │   Side effects               │
│   <1ms per call    │   10ms-10s per call          │
│   Sandboxed        │   Capability-controlled      │
└────────────────────┴─────────────────────────────┘
```

### Migration Path

Phase 1: **Tool infrastructure** (CLI/HTTP/MCP backings + Tool trait rename)
- Implement CLI tool backing via `std::process::Command`
- Implement HTTP tool backing via existing `ReqwestHttpBackend`
- Implement MCP tool backing (JSON-RPC client over stdio)
- Rename `Connector` → `Tool`, merge traits
- Create `Bash`, `Http`, `Fs` as built-in CLI Tools
- Move `llm.*` from global builtins to a Tool (CLI or HTTP backing)

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
- Plugin registry (like crates.io but for lx plugins)
- Plugin template (`lx plugin new my-plugin`)
- Documentation generation from plugin.toml
