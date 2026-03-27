# Browser Plugin Implementation

## Target

lx programs can automate browsers. The default ships with agent-browser, but any CLI that speaks `--json` over stdout can be swapped in.

```lx
-- default: uses agent-browser
snap = Browser.snapshot {interactive: true}
Browser.click "e2"
text = Browser.get_text "e5"

-- swap backend: user provides their own CLI tool
Tool MyBrowser = {
  name: "browser"
  description: "Browser automation via playwright-cli"
  params: {action: Str}
  run = (args) {
    r = Bash.run {command: "playwright-cli " ++ args.action ++ " --json"} ^
    json.parse r.stdout ^
  }
}
```

## Design: The Boundary Is the Tool Trait + CLI Protocol

lx already has three extension boundaries. The browser plugin uses two of them together:

### Boundary 1: The `Tool` trait (lx-level)

Every tool implements `run(args) -> Result`. Users swap implementations by rebinding the name:

```lx
-- stdlib ships this as the default
Tool Browser = { ... agent-browser impl ... }

-- user overrides in their program
Tool Browser = { ... playwright-cli impl ... }
```

The Tool trait is the **uniform interface** for all tooling in lx. Agents call `tool.run(args)` — they don't know or care what's behind it. This is already how Bash, Read, Write, Glob, Grep, WebSearch, and WebFetch work.

### Boundary 2: The `--json` CLI protocol (system-level)

The browser tool shells out to a CLI binary and parses JSON responses. The contract:

```
INPUT:  command-line args (action name + flags)
OUTPUT: JSON on stdout with consistent envelope
```

Any browser CLI that follows this contract is a drop-in replacement. agent-browser uses `{"success": true, "data": {...}}`. The lx tool normalizes this into `Ok data` / `Err message`.

### Why not a new plugin system

| Approach | Problem |
|----------|---------|
| **WASM plugin** | Can't do I/O. Would need `host_exec` which just reinvents `Bash.run` behind a WASM boundary |
| **Native Rust plugin** (dylib) | Platform-specific, ABI instability, security nightmare |
| **MCP server** | Works (`MCP Browser = {command: "..."}`) but 13.7K tokens of schema overhead, tool proliferation degrades agent accuracy |
| **New plugin protocol** | Inventing a new abstraction when Tool + CLI already works |

The Tool trait + CLI JSON protocol gives us:
- **Swappable**: rebind `Browser` to any implementation
- **Composable**: tools call other tools (`Browser.run` calls `Bash.run`)
- **Testable**: mock the CLI output in tests
- **No new infrastructure**: uses existing `Tool` keyword, `Bash.run`, `json.parse`

## Current State

**What exists:**
- `Tool` trait in `crates/lx/std/tool.lx` — `run(args)`, `schema()`, `validate()`
- 8 default tools in `crates/lx/std/tools/` — loaded by `default_tools.rs` via `include_str!()`
- `Bash.run` builtin returning `{stdout, stderr, code}` — `crates/lx/src/builtins/shell.rs`
- `json.parse` builtin — `crates/lx/src/builtins/register.rs:214`
- `CLI` keyword desugaring — auto-generates `run` that calls `bash(self.command ++ " " ++ args.command)`
- Tool sources registered in `crates/lx/src/stdlib/mod.rs:62-79` via `lx_std_module_source()`

**What's missing:**
- `crates/lx/std/tools/browser.lx` — the actual browser tool
- agent-browser binary on user's PATH (external dependency)

That's it. No new host functions, no new plugin systems, no build pipelines.

## Implementation

### 1. Create `crates/lx/std/tools/browser.lx`

The core tool file. All browser operations go through a single `run` method that dispatches on `args.action`.

```lx
-- default tool: browser automation via agent-browser CLI (--json protocol)

Tool Browser = {
  name: "browser"
  description: "Automate browsers via accessibility snapshots and ref-based interaction"
  params: {action: Str}
  session: "lx-default"

  run = (args) {
    cmd = self.build_cmd args
    r = Bash.run {command: cmd} ^
    r.code != 0 ? (Err r.stderr) : (self.parse_response r.stdout)
  }

  build_cmd = (args) {
    base = "agent-browser " ++ args.action
    flags = " --json --session " ++ self.session
    base ++ (args.flags ?? "") ++ flags
  }

  parse_response = (raw) {
    data = json.parse raw ^
    data.success ? (Ok (data.data ?? {})) : (Err (data.error ?? "unknown error"))
  }
}
```

But the raw `run({action: "..."})` interface is awkward for callers. They'd have to construct command strings. So add convenience methods:

```lx
Tool Browser = {
  name: "browser"
  description: "Automate browsers via accessibility snapshots and ref-based interaction"
  params: {action: Str}
  session: "lx-default"

  run = (args) {
    cmd = "agent-browser " ++ args.action ++ " --json --session " ++ self.session
    r = Bash.run {command: cmd} ^
    r.code != 0 ? (Err r.stderr) : {
      data = json.parse r.stdout ^
      data.success ? (Ok (data.data ?? {})) : (Err (data.error ?? "unknown error"))
    }
  }

  open = (url) {
    self.run {action: "open " ++ url}
  }

  snapshot = (config) {
    flags = config.interactive ?? true ? " -i" : ""
    flags = flags ++ (config.content ?? false ? " -c" : "")
    flags = flags ++ (config.detailed ?? false ? " -d" : "")
    self.run {action: "snapshot" ++ flags}
  }

  click = (ref) {
    r = ref | starts_with? "@" ? ref : ("@" ++ ref)
    self.run {action: "click " ++ r}
  }

  fill = (config) {
    r = config.ref | starts_with? "@" ? config.ref : ("@" ++ config.ref)
    self.run {action: "fill " ++ r ++ " \"" ++ config.value ++ "\""}
  }

  type_text = (config) {
    r = config.ref | starts_with? "@" ? config.ref : ("@" ++ config.ref)
    self.run {action: "type " ++ r ++ " \"" ++ config.value ++ "\""}
  }

  press = (key) {
    self.run {action: "press " ++ key}
  }

  get_text = (ref) {
    r = ref | starts_with? "@" ? ref : ("@" ++ ref)
    self.run {action: "get text " ++ r}
  }

  get_url = () {
    self.run {action: "get url"}
  }

  screenshot = (config) {
    flags = config.full ?? false ? " --full" : ""
    flags = flags ++ (config.annotate ?? false ? " --annotate" : "")
    flags = flags ++ (config.path ? (" --output " ++ config.path) : "")
    self.run {action: "screenshot" ++ flags}
  }

  scroll = (config) {
    target = config.ref ? (" @" ++ config.ref) : ""
    dir = config.direction ?? "down"
    self.run {action: "scroll" ++ target ++ " --direction " ++ dir}
  }

  wait = (config) {
    -- config is either Str (selector) or Record {text?, ms?, selector?}
    self.run {action: "wait " ++ (config.selector ?? config.text ?? (to_str (config.ms ?? 1000)))}
  }

  eval = (js) {
    self.run {action: "eval \"" ++ js ++ "\""}
  }

  back = () { self.run {action: "back"} }
  forward = () { self.run {action: "forward"} }
  close = () { self.run {action: "close"} }
}
```

### 2. Register in stdlib module source map

**File:** `crates/lx/src/stdlib/mod.rs`

Add to the `lx_std_module_source` match:

```rust
"tools/browser" => Some(include_str!("../../std/tools/browser.lx")),
```

### 3. Add to default tool loading

**File:** `crates/lx/src/interpreter/default_tools.rs`

Add `"tools/browser"` to `DEFAULT_TOOL_SOURCES`:

```rust
const DEFAULT_TOOL_SOURCES: &[&str] = &[
    "tools/bash", "tools/read", "tools/write", "tools/edit",
    "tools/glob", "tools/grep", "tools/web_search", "tools/web_fetch",
    "tools/browser",
];
```

### 4. Test: unit test for browser tool

**File:** `tests/suite/tools/browser.lx` (or wherever tool tests live)

```lx
-- test browser tool exists and has expected methods
assert (Browser.name) == "browser"
assert (Browser.description | len) > 0

-- test build_cmd constructs correct command strings
-- (actual browser tests need agent-browser installed)
```

### 5. Test: integration test (requires agent-browser)

**File:** `tests/integration/browser.lx`

```lx
-- integration test: requires agent-browser installed
-- skip if not available
check = Bash.run {command: "command -v agent-browser"}
(check is Err) ? (emit "SKIP: agent-browser not installed") : {

  result = Browser.open "https://example.com"
  assert result is Ok

  snap = Browser.snapshot {interactive: true}
  assert snap is Ok
  assert snap.unwrap.snapshot | len > 0

  url = Browser.get_url ()
  assert url is Ok

  Browser.close ()
}
```

### 6. Swappability documentation by example

Users override the default Browser tool in their own `.lx` files:

```lx
-- override default Browser with playwright-mcp via MCP keyword
MCP Browser = {
  command: "npx"
  args: ["@playwright/mcp@latest", "--headless"]
}

-- or override with a custom CLI
Tool Browser = {
  name: "browser"
  params: {action: Str}
  run = (args) {
    r = Bash.run {command: "my-browser-tool " ++ args.action ++ " --json"} ^
    json.parse r.stdout ^
  }
}
```

Because lx uses lexical scoping with rebinding, a later `Tool Browser = {...}` in the same scope shadows the default. Agents spawned after the rebinding use the new implementation.

## Execution Order

1. Write `crates/lx/std/tools/browser.lx`
2. Add `include_str!` to `stdlib/mod.rs`
3. Add to `DEFAULT_TOOL_SOURCES` in `default_tools.rs`
4. Write unit test
5. Write integration test
6. Verify the tool works: `just test`

## Files Changed

- `crates/lx/src/stdlib/mod.rs` — add one line to `lx_std_module_source`
- `crates/lx/src/interpreter/default_tools.rs` — add `"tools/browser"` to array

## Files Created

- `crates/lx/std/tools/browser.lx` — the tool implementation
- `tests/suite/tools/browser.lx` — unit test (no agent-browser needed)
- `tests/integration/browser.lx` — integration test (needs agent-browser)

## What WASM Plugins Are Good For

For the record, the WASM plugin system is still valuable — just not for I/O-heavy tools. Good WASM plugin use cases:

| Use Case | Why WASM Works |
|----------|---------------|
| JSON schema validation | Pure computation, sandboxed |
| Regex engine | Pure computation, fuel-limited |
| Markdown/HTML parsing | Pure transform |
| Template rendering | Pure transform |
| Data encoding (base64, etc.) | Pure transform |
| Custom lx DSL transforms | Pure computation |

The pattern: **WASM for compute, Tool+CLI for I/O.** The browser plugin is I/O.
