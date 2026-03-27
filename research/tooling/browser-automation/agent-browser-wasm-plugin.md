# Agent Browser WASM Plugin for lx: Technical Context & Task List

## Goal

Ship a WASM plugin (`lx-browser`) that gives any lx agent full browser automation via Vercel's agent-browser. An lx program should be able to:

```lx
use wasm/lx_browser

url = "https://example.com"
lx_browser.open url                          -- navigate
snap = lx_browser.snapshot ()                -- get interactive elements with refs
lx_browser.click snap.refs.e2               -- click a ref
lx_browser.fill {ref: "e3", value: "hello"} -- fill a form field
text = lx_browser.get_text "e5"             -- extract text
lx_browser.close ()                          -- cleanup
```

## Architecture Decision: Why WASM, Not CLI Keyword

lx already has a `CLI` keyword that shells out. A `CLI Browser = { command: "agent-browser" }` would work today. The WASM plugin approach is better because:

1. **Structured I/O** — the plugin parses agent-browser's JSON responses and returns typed lx Records, not raw stdout strings. Callers get `snap.refs.e2` instead of parsing text.
2. **Session management** — the plugin tracks the daemon socket path and session lifecycle internally. No caller bookkeeping.
3. **Ref tracking** — the plugin maintains a ref map across snapshots, enabling `lx_browser.click "e2"` without the caller re-parsing the snapshot.
4. **Batch optimization** — the plugin can coalesce multiple actions into a single `batch` command over the socket, cutting per-action overhead.
5. **Sandbox integration** — WASM plugins run under lx's fuel limits and sandbox policy. A CLI tool has no fuel metering.

## How It Works

```
lx program
  |
  | use wasm/lx_browser
  |
  v
lx runtime (Extism host)
  |
  | JSON marshaling (LxVal <-> JSON)
  |
  v
lx_browser.wasm (Extism guest, wasm32-unknown-unknown)
  |
  | host_exec("agent-browser open https://example.com --json --session lx-default")
  |
  v
host_exec host function (new, added to wasm.rs)
  |
  | std::process::Command (sandboxed by lx sandbox policy)
  |
  v
agent-browser CLI binary
  |
  | Unix socket (newline-delimited JSON)
  |
  v
agent-browser daemon (Rust, self-forking)
  |
  | CDP over WebSocket (localhost)
  |
  v
Chrome (headless, local)
```

### Key architectural constraint

WASM plugins (wasm32-unknown-unknown) cannot do process spawning or socket I/O — even with WASI enabled, socket support is incomplete. The plugin must delegate all I/O to the host via a new `host_exec` host function that runs shell commands and returns stdout.

This is the same pattern as lx's default tools (`Bash.run` calls a Rust builtin). The WASM boundary adds fuel metering and type marshaling.

### Alternative considered: direct socket communication

The plugin could speak agent-browser's newline-delimited JSON protocol over a Unix socket, bypassing the CLI. This would be faster (no process spawn per command) but requires:
- A `host_socket_connect` / `host_socket_send` / `host_socket_recv` host function set
- Socket path discovery logic
- Daemon liveness checking

This is the right long-term path but `host_exec` is simpler to ship first. The plugin's exported API stays the same either way — the transport is an internal detail.

## Existing lx Infrastructure

### WASM Plugin System

| Component | Location | What It Does |
|-----------|----------|-------------|
| Plugin loader | `crates/lx/src/stdlib/wasm.rs` | Reads plugin.toml, builds Extism manifest, registers host fns, caches in global HashMap |
| JSON marshaling | `crates/lx/src/stdlib/wasm_marshal.rs` | LxVal <-> JSON for all types (Int, Float, Bool, Str, List, Record, Ok/Err, Tagged) |
| Plugin CLI | `crates/lx-cli/src/plugin.rs` | `lx plugin new/install/list/remove` |
| Module resolver | `crates/lx/src/interpreter/modules.rs` | `use wasm/plugin_name` triggers `load_wasm_plugin()` |
| Test plugin | `tests/fixtures/plugins/test_upper/` | Reference: plugin.toml + .wasm, exports `upper(str) -> str` |

### Plugin manifest format

```toml
[plugin]
name = "lx_browser"
version = "0.1.0"
description = "Browser automation via agent-browser"
wasm = "target/wasm32-unknown-unknown/release/lx_browser.wasm"

[exports]
open = { arity = 1 }
snapshot = { arity = 1 }
click = { arity = 1 }
fill = { arity = 1 }
type_text = { arity = 1 }
press = { arity = 1 }
get_text = { arity = 1 }
get_url = { arity = 1 }
screenshot = { arity = 1 }
scroll = { arity = 1 }
wait = { arity = 1 }
eval = { arity = 1 }
back = { arity = 1 }
forward = { arity = 1 }
close = { arity = 1 }
batch = { arity = 1 }

[sandbox]
wasi = true
```

### Current host functions available to WASM plugins

| Function | Signature | What It Does |
|----------|-----------|-------------|
| `plugin_log` | `(level: u32, msg: String) -> ()` | Log to stderr with level label |
| `plugin_get_config` | `(key: String) -> String` | Read environment variable |

### What's missing: `host_exec`

A new host function that executes a shell command and returns stdout. This is the critical gap.

```rust
// Proposed signature:
host_exec(command: String) -> String
// Returns JSON: {"code": 0, "stdout": "...", "stderr": "..."}
```

## agent-browser Internals (Plugin-Relevant)

### Wire protocol

CLI to daemon: newline-delimited JSON over Unix socket (`~/.agent-browser/<session>.sock`).

```json
// Request:
{"id":"r123","action":"navigate","url":"https://example.com"}

// Response:
{"id":"r123","success":true,"data":{"url":"https://example.com","title":"Example"}}
```

### CLI invocation pattern (what the plugin will call via host_exec)

Every command supports `--json` for structured output and `--session <name>` for session isolation.

```bash
# Navigate
agent-browser open https://example.com --json --session lx

# Snapshot (interactive elements only)
agent-browser snapshot -i --json --session lx

# Click a ref
agent-browser click @e2 --json --session lx

# Fill a form field
agent-browser fill @e3 "search query" --json --session lx

# Get text content
agent-browser get text @e5 --json --session lx

# Batch (multiple commands, one invocation)
echo '[{"action":"click","selector":"@e2"},{"action":"snapshot","interactive":true}]' | agent-browser batch --json --session lx
```

### Snapshot format (what the plugin parses)

Interactive snapshot (`snapshot -i --json`) returns:

```json
{
  "success": true,
  "data": {
    "snapshot": "- heading \"Example Domain\" [level=1, ref=e1]\n- button \"Submit\" [ref=e2]\n- textbox \"Email\" [ref=e3]",
    "origin": "https://example.com",
    "refs": {
      "e1": {"role": "heading", "name": "Example Domain"},
      "e2": {"role": "button", "name": "Submit"},
      "e3": {"role": "textbox", "name": "Email"}
    }
  }
}
```

The plugin returns this as an lx Record:

```lx
{
  snapshot: "- heading \"Example Domain\" ...",
  origin: "https://example.com",
  refs: {
    e1: {role: "heading", name: "Example Domain"},
    e2: {role: "button", name: "Submit"},
    e3: {role: "textbox", name: "Email"}
  }
}
```

### Ref lifecycle

Refs are invalidated on every `snapshot` call (the daemon clears and rebuilds the ref map). After navigation or significant DOM changes, the agent must re-snapshot to get fresh refs.

### Session model

Each session gets its own daemon process, socket, PID file, and Chrome instance. The plugin uses a fixed session name (`lx-<pid>` or `lx-default`) to isolate from other agent-browser users. `close` terminates the daemon + browser.

### Daemon auto-start

The CLI automatically spawns the daemon if it's not running (`ensure_daemon()` re-spawns the binary with `AGENT_BROWSER_DAEMON=1`). The plugin doesn't need to manage daemon lifecycle — just call commands.

## lx-desktop Integration (Future)

When an lx agent running in lx-desktop uses this plugin, the desktop should be able to attach a `Browser` pane to the same Chrome instance. The path:

1. Plugin calls `agent-browser get cdp-url --session lx --json` to get the CDP WebSocket URL
2. lx-desktop's `BrowserView` connects to that same CDP endpoint via `common_cdp`
3. User sees the agent's browser actions in real-time via screencast

This is a future concern — the plugin works headlessly first, desktop integration follows.

---

## Task List

### Phase 1: Host Function — `host_exec`

**1.1** Add `host_exec` host function to `crates/lx/src/stdlib/wasm.rs`
- Signature: `host_exec(command: String) -> String`
- Implementation: `std::process::Command::new("sh").arg("-c").arg(&command)`, capture stdout+stderr, return JSON `{"code": N, "stdout": "...", "stderr": "..."}`
- Register alongside existing `plugin_log` and `plugin_get_config` in the `PluginBuilder`
- Respect sandbox policy: if `policy.agent == false` or shell access is restricted, return error JSON instead of executing

**1.2** Add sandbox gating for `host_exec`
- Check the thread-local `POLICY_STACK` in `sandbox_scope.rs`
- If no policy is active, allow (matches current behavior for builtins)
- If policy active and shell is disallowed, return `{"code": -1, "stdout": "", "stderr": "sandbox: shell execution denied"}`

**1.3** Test `host_exec` with a minimal plugin
- Create `tests/fixtures/plugins/test_exec/` with a plugin that calls `host_exec("echo hello")` and returns the stdout
- Add test in `tests/wasm_plugin.lx` or equivalent

### Phase 2: Plugin Scaffold

**2.1** Scaffold the plugin project
- `lx plugin new lx-browser` (or manually create the structure)
- Crate at `plugins/lx-browser/` in the repo (or a separate repo, TBD)
- Dependencies: `extism-pdk = "1.4.1"`, `serde`, `serde_json`

**2.2** Define the host function import in the plugin
```rust
#[host_fn]
extern "ExtismHost" {
    fn host_exec(command: &str) -> String;
}
```

**2.3** Implement internal helpers
- `fn run_ab(args: &str) -> Result<serde_json::Value, String>` — calls `host_exec("agent-browser {args} --json --session lx-default")`, parses JSON response, checks `success` field
- `fn format_ref(r: &str) -> String` — normalizes ref format (prepends `@` if needed, e.g. `"e2"` -> `"@e2"`)
- `fn unwrap_data(response: Value) -> Value` — extracts `.data` from success responses, returns `.error` string on failure

### Phase 3: Core Exports

**3.1** `open(url: String) -> Result`
- Calls `agent-browser open {url} --json --session lx-default`
- Returns `Ok {url, title}` or `Err "message"`

**3.2** `snapshot(config: Record) -> Result`
- Default: `agent-browser snapshot -i --json --session lx-default`
- Config options: `{interactive: Bool, content: Bool, detailed: Bool}`
- `-i` if `interactive` (default true), `-c` if `content`, `-d` if `detailed`
- Returns `Ok {snapshot, origin, refs}` — refs is a Record of ref_id -> {role, name}

**3.3** `click(ref_or_config: String|Record) -> Result`
- String: `agent-browser click @{ref} --json --session lx-default`
- Record: supports `{ref, new_tab: Bool}`
- Returns `Ok {}` or `Err "message"`

**3.4** `fill(config: Record) -> Result`
- `config: {ref: String, value: String}`
- Calls `agent-browser fill @{ref} "{value}" --json --session lx-default`
- Returns `Ok {}` or `Err`

**3.5** `type_text(config: Record) -> Result`
- `config: {ref: String, value: String}`
- Calls `agent-browser type @{ref} "{value}" --json --session lx-default`
- Returns `Ok {}` or `Err`

**3.6** `press(key: String) -> Result`
- Calls `agent-browser press {key} --json --session lx-default`
- Supports: Enter, Tab, Escape, ArrowDown, etc.

**3.7** `get_text(ref: String) -> Result`
- Calls `agent-browser get text @{ref} --json --session lx-default`
- Returns `Ok "text content"` or `Err`

**3.8** `get_url(unit: Any) -> Result`
- Calls `agent-browser get url --json --session lx-default`
- Returns `Ok "https://..."` or `Err`

**3.9** `screenshot(config: Record) -> Result`
- Config: `{path: String?, full: Bool?, annotate: Bool?}`
- Calls `agent-browser screenshot --json --session lx-default` with flags
- Returns `Ok {path: "/tmp/screenshot.png"}` or `Err`

**3.10** `close(unit: Any) -> Result`
- Calls `agent-browser close --json --session lx-default`
- Returns `Ok {}` or `Err`

### Phase 4: Extended Exports

**4.1** `scroll(config: Record) -> Result` — `{ref?: String, direction?: "up"|"down", amount?: Int}`

**4.2** `wait(config: String|Record) -> Result` — wait for selector/text/time: `{selector?: String, text?: String, ms?: Int}`

**4.3** `eval(js: String) -> Result` — execute JavaScript on page, return result

**4.4** `back(unit) -> Result` / `forward(unit) -> Result` — navigation history

**4.5** `select(config: Record) -> Result` — `{ref: String, value: String}` for dropdowns

**4.6** `batch(commands: List) -> Result` — send multiple commands in one invocation via stdin pipe to `agent-browser batch --json`

### Phase 5: Build & Distribution

**5.1** Add build recipe to justfile
- `just build-plugin lx-browser` → `cargo build --release --target wasm32-unknown-unknown -p lx-browser`
- Copy .wasm to plugin directory

**5.2** Add install recipe
- `just install-plugin lx-browser` → `lx plugin install plugins/lx-browser`

**5.3** Add agent-browser binary check
- On first `open` call, check if `agent-browser` is in PATH
- If not, return `Err "agent-browser not found. Install: npm install -g agent-browser && agent-browser install"`

**5.4** Pre-build .wasm and commit to `plugins/lx-browser/`
- So users can `lx plugin install plugins/lx-browser` without needing the Rust WASM toolchain

### Phase 6: Tests

**6.1** Unit tests in the plugin crate
- Test JSON parsing of snapshot responses
- Test ref normalization
- Test command string construction
- Test error response handling

**6.2** Integration test `.lx` file
- `tests/browser_plugin.lx` (or `tests/suite/browser/` if test directories are used)
- Tests require agent-browser installed; skip with guard if not available
- Basic flow: open → snapshot → click → get_text → close

**6.3** Sandbox test
- Verify that `host_exec` is denied when sandbox policy restricts shell access

### Phase 7: Desktop Bridge (Future)

**7.1** Add `get_cdp_url` export to the plugin
- Calls `agent-browser get cdp-url --json --session lx-default`
- Returns the WebSocket CDP URL

**7.2** Wire lx-desktop's `BrowserView` to accept a CDP URL from an lx agent
- When agent runtime emits a browser event, `BrowserView` connects to that CDP session
- Screencast + input forwarding over the shared session

**7.3** Add pane auto-creation
- When an lx agent calls `lx_browser.open(url)` inside lx-desktop, auto-create a `DesktopPane::Browser` pane connected to the same session
