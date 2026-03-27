# Browser Plugin Implementation

## Target

lx programs can automate browsers via agent-browser:

```lx
use wasm/lx_browser

lx_browser.open "https://example.com"
snap = lx_browser.snapshot {interactive: true}
lx_browser.click "e2"
text = lx_browser.get_text "e5"
lx_browser.close ()
```

## Current State

**What exists:**
- WASM plugin infrastructure is complete (Extism 1.20.0, JSON marshaling, plugin.toml manifest, `use wasm/` module resolution, `lx plugin` CLI)
- One test plugin (`test_upper`) proves the system works end-to-end
- Sandbox policy system with capability flags (fs_read, fs_write, net_allow, agent, mcp, llm)
- `bash` builtin in `crates/lx/src/builtins/shell.rs` — `std::process::Command::new("bash").arg("-c")` returning `{stdout, stderr, code}`

**What's missing:**
- `host_exec` host function — WASM plugins can't execute commands (only `plugin_log` and `plugin_get_config` exist)
- No `shell` capability in sandbox policy — sandbox can't gate whether a WASM plugin is allowed to run commands
- The lx-browser plugin itself (Rust cdylib crate targeting wasm32-unknown-unknown)
- agent-browser binary (users install separately: `npm install -g agent-browser && agent-browser install`)

## Architecture

```
lx program calls lx_browser.open("https://example.com")
    |
    v
Extism host (wasm.rs) — marshals LxVal to JSON, calls WASM export
    |
    v
lx_browser.wasm — constructs command string, calls host_exec
    |
    v
host_exec host function (wasm.rs) — runs std::process::Command, returns {code, stdout, stderr}
    |
    v
agent-browser CLI (--json --session lx) — talks to daemon over Unix socket
    |
    v
agent-browser daemon — CDP over localhost WebSocket
    |
    v
Chrome (headless)
```

The WASM plugin handles: command construction, JSON response parsing, ref normalization, error wrapping.
The host handles: process execution, sandbox gating.
agent-browser handles: browser lifecycle, CDP, accessibility snapshots.

## Implementation

### 1. Add `host_exec` host function to wasm.rs

**File:** `crates/lx/src/stdlib/wasm.rs`

Add a third host function alongside `plugin_log` and `plugin_get_config`. It takes a command string and returns JSON with code, stdout, stderr — identical to `bi_bash` in `builtins/shell.rs`.

```rust
let exec_fn = extism::Function::new(
    "host_exec",
    [extism::PTR],           // input: command string
    [extism::PTR],           // output: JSON {code, stdout, stderr}
    extism::UserData::new(()),
    |plugin: &mut extism::CurrentPlugin, inputs: &[extism::Val], outputs: &mut [extism::Val], _ud: extism::UserData<()>| {
        let cmd: String = plugin.memory_get_val(&inputs[0])?;
        let result = match std::process::Command::new("bash").arg("-c").arg(&cmd).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let code = output.status.code().unwrap_or(-1);
                serde_json::json!({"code": code, "stdout": stdout, "stderr": stderr})
            }
            Err(e) => serde_json::json!({"code": -1, "stdout": "", "stderr": e.to_string()})
        };
        let json = result.to_string();
        let handle = plugin.memory_new(&json)?;
        outputs[0] = plugin.memory_to_val(handle);
        Ok(())
    },
);
```

Then change the builder line from:

```rust
let mut builder = extism::PluginBuilder::new(&extism_manifest)
    .with_wasi(wasi_enabled)
    .with_functions([log_fn, config_fn]);
```

to:

```rust
let mut builder = extism::PluginBuilder::new(&extism_manifest)
    .with_wasi(wasi_enabled)
    .with_functions([log_fn, config_fn, exec_fn]);
```

### 2. Add `shell` capability to sandbox policy

**Files:**
- `crates/lx/src/stdlib/sandbox/mod.rs` — add `pub shell: bool` to `Policy` struct
- `crates/lx/src/stdlib/sandbox/sandbox_policy.rs`:
  - All presets: `shell: false` except "full" which gets `shell: true`
  - `parse_policy`: handle `shell` Bool key
  - `intersect_policies`: AND the `shell` fields
  - `permits_check`: add `"shell"` match arm
  - `policy_to_describe`: include `shell` in output record

This is not blocking for the initial plugin (host_exec won't check sandbox in phase 1) but should be done before any untrusted code runs WASM plugins.

### 3. Gate `host_exec` on sandbox policy

**File:** `crates/lx/src/stdlib/wasm.rs`

The challenge: Extism host functions don't have access to lx's `RuntimeCtx` or the thread-local `POLICY_STACK`. Two approaches:

**Option A (simpler, do this first):** Add a `plugin.toml` field `[sandbox] shell = true` and check it at plugin load time. If a plugin declares `shell = true`, `host_exec` is registered. If not, `host_exec` is registered as a function that always returns an error. This is load-time gating, not runtime gating.

```toml
[sandbox]
wasi = true
shell = true   # <-- new: plugin needs host_exec
```

In `load_plugin()`, conditionally build `exec_fn`:

```rust
let shell_enabled = manifest.sandbox.as_ref().and_then(|s| s.shell).unwrap_or(false);

let exec_fn = if shell_enabled {
    // real implementation
} else {
    // error stub: returns {"code": -1, "stdout": "", "stderr": "shell not permitted"}
};
```

**Option B (later):** Use a global `AtomicBool` or thread-local that `sandbox.scope()` toggles before calling into WASM. The host function checks this flag. More granular but more complex.

### 4. Test `host_exec` with a minimal plugin

**New fixture:** `tests/fixtures/plugins/test_exec/`

Create a WASM plugin that calls `host_exec("echo hello_from_wasm")` and returns stdout.

**Plugin source** (`test_exec/src/lib.rs`):
```rust
use extism_pdk::*;

#[host_fn]
extern "ExtismHost" {
    fn host_exec(command: &str) -> String;
}

#[plugin_fn]
pub fn run_cmd(input: String) -> FnResult<String> {
    let result = unsafe { host_exec(&input)? };
    Ok(result)
}
```

**Manifest** (`test_exec/plugin.toml`):
```toml
[plugin]
name = "test_exec"
version = "0.1.0"
description = "Test host_exec host function"
wasm = "test_exec.wasm"

[exports]
run_cmd = { arity = 1 }

[sandbox]
wasi = true
shell = true
```

**Test** (`tests/wasm_exec.lx`):
```lx
use wasm/test_exec
result = test_exec.run_cmd "echo hello_from_wasm"
-- result is JSON string: {"code":0,"stdout":"hello_from_wasm\n","stderr":""}
```

Build the .wasm, commit it alongside plugin.toml in the fixture directory.

### 5. Scaffold lx-browser plugin

**Location:** `plugins/lx-browser/` (new directory in repo root)

Scaffold structure:
```
plugins/lx-browser/
  Cargo.toml          # cdylib, extism-pdk, serde, serde_json
  .cargo/config.toml  # target = "wasm32-unknown-unknown"
  plugin.toml         # name, wasm path, exports, sandbox config
  src/
    lib.rs            # all exports, host_exec import, helpers
```

**Cargo.toml:**
```toml
[package]
name = "lx-browser"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.4.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**.cargo/config.toml:**
```toml
[build]
target = "wasm32-unknown-unknown"
```

**plugin.toml:**
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
select = { arity = 1 }
wait = { arity = 1 }
eval = { arity = 1 }
back = { arity = 1 }
forward = { arity = 1 }
close = { arity = 1 }
batch = { arity = 1 }

[sandbox]
wasi = true
shell = true
```

### 6. Implement plugin internals (src/lib.rs)

**Host function import:**
```rust
#[host_fn]
extern "ExtismHost" {
    fn host_exec(command: &str) -> String;
}
```

**Core helper — run an agent-browser command:**
```rust
fn ab(args: &str) -> Result<serde_json::Value, String> {
    let cmd = format!("agent-browser {args} --json --session lx-default");
    let raw = unsafe { host_exec(&cmd) }
        .map_err(|e| format!("host_exec failed: {e}"))?;
    let exec_result: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("host_exec JSON parse: {e}"))?;

    let code = exec_result["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        let stderr = exec_result["stderr"].as_str().unwrap_or("unknown error");
        return Err(format!("agent-browser exited {code}: {stderr}"));
    }

    let stdout = exec_result["stdout"].as_str().unwrap_or("");
    let response: serde_json::Value = serde_json::from_str(stdout)
        .map_err(|e| format!("agent-browser JSON parse: {e}"))?;

    if response["success"].as_bool() == Some(true) {
        Ok(response["data"].clone())
    } else {
        let err = response["error"].as_str().unwrap_or("unknown error");
        Err(err.to_string())
    }
}
```

**Result wrapping helper:**
```rust
fn ok_json(data: serde_json::Value) -> String {
    serde_json::json!({"Ok": data}).to_string()
}

fn err_json(msg: &str) -> String {
    serde_json::json!({"Err": msg}).to_string()
}

fn wrap(result: Result<serde_json::Value, String>) -> FnResult<String> {
    match result {
        Ok(data) => Ok(ok_json(data)),
        Err(msg) => Ok(err_json(&msg)),
    }
}
```

The marshaling pipeline: lx runtime marshals LxVal -> JSON string -> WASM plugin parses JSON -> constructs command -> calls host_exec -> gets stdout JSON -> parses agent-browser response -> wraps as Ok/Err JSON -> returns string -> lx runtime unmarshals JSON -> LxVal. The `{"Ok": ...}` / `{"Err": ...}` envelope is recognized by `json_to_lxval` in `wasm_marshal.rs` and becomes `LxVal::Ok(...)` / `LxVal::Err(...)`.

**Ref normalization:**
```rust
fn ref_arg(r: &str) -> String {
    if r.starts_with('@') { r.to_string() } else { format!("@{r}") }
}
```

### 7. Implement exports

Each export is a `#[plugin_fn]` that takes a JSON string (from LxVal marshaling), calls `ab()`, and returns a JSON string.

**open:**
```rust
#[plugin_fn]
pub fn open(input: String) -> FnResult<String> {
    // input is a JSON string (either bare "url" or {"url": "..."})
    let url = parse_string_or_field(&input, "url");
    wrap(ab(&format!("open {url}")))
}
```

**snapshot:**
```rust
#[plugin_fn]
pub fn snapshot(input: String) -> FnResult<String> {
    let config: serde_json::Value = serde_json::from_str(&input).unwrap_or_default();
    let mut flags = String::new();
    if config.get("interactive").and_then(|v| v.as_bool()).unwrap_or(true) {
        flags.push_str(" -i");
    }
    if config.get("content").and_then(|v| v.as_bool()).unwrap_or(false) {
        flags.push_str(" -c");
    }
    if config.get("detailed").and_then(|v| v.as_bool()).unwrap_or(false) {
        flags.push_str(" -d");
    }
    wrap(ab(&format!("snapshot{flags}")))
}
```

**click:**
```rust
#[plugin_fn]
pub fn click(input: String) -> FnResult<String> {
    let r = parse_string_or_field(&input, "ref");
    wrap(ab(&format!("click {}", ref_arg(&r))))
}
```

**fill:**
```rust
#[plugin_fn]
pub fn fill(input: String) -> FnResult<String> {
    let config: serde_json::Value = serde_json::from_str(&input)?;
    let r = config["ref"].as_str().unwrap_or("");
    let value = config["value"].as_str().unwrap_or("");
    wrap(ab(&format!("fill {} \"{}\"", ref_arg(r), value.replace('"', "\\\""))))
}
```

**type_text:**
```rust
#[plugin_fn]
pub fn type_text(input: String) -> FnResult<String> {
    let config: serde_json::Value = serde_json::from_str(&input)?;
    let r = config["ref"].as_str().unwrap_or("");
    let value = config["value"].as_str().unwrap_or("");
    wrap(ab(&format!("type {} \"{}\"", ref_arg(r), value.replace('"', "\\\""))))
}
```

**press:**
```rust
#[plugin_fn]
pub fn press(input: String) -> FnResult<String> {
    let key = parse_bare_string(&input);
    wrap(ab(&format!("press {key}")))
}
```

**get_text:**
```rust
#[plugin_fn]
pub fn get_text(input: String) -> FnResult<String> {
    let r = parse_bare_string(&input);
    wrap(ab(&format!("get text {}", ref_arg(&r))))
}
```

**get_url:**
```rust
#[plugin_fn]
pub fn get_url(_input: String) -> FnResult<String> {
    wrap(ab("get url"))
}
```

**screenshot:**
```rust
#[plugin_fn]
pub fn screenshot(input: String) -> FnResult<String> {
    let config: serde_json::Value = serde_json::from_str(&input).unwrap_or_default();
    let mut flags = String::new();
    if config.get("full").and_then(|v| v.as_bool()).unwrap_or(false) {
        flags.push_str(" --full");
    }
    if config.get("annotate").and_then(|v| v.as_bool()).unwrap_or(false) {
        flags.push_str(" --annotate");
    }
    if let Some(path) = config.get("path").and_then(|v| v.as_str()) {
        flags.push_str(&format!(" --output {path}"));
    }
    wrap(ab(&format!("screenshot{flags}")))
}
```

**scroll, select, wait, eval, back, forward, close, batch** follow the same pattern. Each parses its input, constructs the agent-browser command, calls `ab()`, wraps the result.

**batch** is the most complex — it takes a list of command records:
```rust
#[plugin_fn]
pub fn batch(input: String) -> FnResult<String> {
    // input is a JSON array of {action: "click", ref: "e2"} etc.
    // pipe it to: echo '<json>' | agent-browser batch --json --session lx-default
    let escaped = input.replace('\'', "'\\''");
    wrap(ab(&format!("batch --bail <<< '{escaped}'")))
}
```

### 8. String parsing helpers

The plugin receives JSON strings from LxVal marshaling. When the lx caller passes a bare string like `lx_browser.click "e2"`, Extism delivers `"e2"` (JSON-encoded string). When the caller passes a Record like `lx_browser.fill {ref: "e3", value: "hello"}`, Extism delivers `{"ref":"e3","value":"hello"}`.

```rust
fn parse_bare_string(input: &str) -> String {
    serde_json::from_str::<String>(input)
        .unwrap_or_else(|_| input.trim_matches('"').to_string())
}

fn parse_string_or_field(input: &str, field: &str) -> String {
    if let Ok(s) = serde_json::from_str::<String>(input) {
        return s;
    }
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(input) {
        if let Some(s) = obj[field].as_str() {
            return s.to_string();
        }
    }
    input.trim_matches('"').to_string()
}
```

### 9. Build pipeline

**Add to justfile:**

```just
build-plugin name:
    cd plugins/{{name}} && cargo build --release
    @echo "built: plugins/{{name}}/target/wasm32-unknown-unknown/release/$(echo {{name}} | tr '-' '_').wasm"

install-plugin name:
    cargo run -p lx-cli -- plugin install plugins/{{name}}
```

**Build commands:**
```bash
# requires: rustup target add wasm32-unknown-unknown
just build-plugin lx-browser
just install-plugin lx-browser
```

### 10. Pre-built .wasm for distribution

After building, copy the .wasm into the plugin directory so users without the Rust WASM toolchain can install directly:

```bash
cp plugins/lx-browser/target/wasm32-unknown-unknown/release/lx_browser.wasm plugins/lx-browser/
```

Update plugin.toml to reference the local copy:
```toml
wasm = "lx_browser.wasm"
```

Commit the .wasm binary. Users install with: `lx plugin install plugins/lx-browser`

### 11. Binary availability check

The plugin's `open` function should check if `agent-browser` is available before attempting to use it:

```rust
fn check_agent_browser() -> Result<(), String> {
    let result = unsafe { host_exec("command -v agent-browser") }
        .map_err(|e| format!("host_exec: {e}"))?;
    let parsed: serde_json::Value = serde_json::from_str(&result)
        .map_err(|e| format!("parse: {e}"))?;
    if parsed["code"].as_i64() != Some(0) {
        return Err("agent-browser not found in PATH. Install: npm install -g agent-browser && agent-browser install".into());
    }
    Ok(())
}
```

Call this once on the first `open` command (use a static `OnceLock` or just check every time — the command is fast).

### 12. Tests

**Unit tests** in `plugins/lx-browser/src/lib.rs`:
- `#[test] fn test_ref_arg()` — verifies `@` prefix logic
- `#[test] fn test_parse_bare_string()` — JSON string unwrapping
- `#[test] fn test_parse_string_or_field()` — bare string vs Record
- `#[test] fn test_ok_err_json()` — envelope formatting

These are standard Rust tests (`cargo test` in the plugin crate, running on host target not wasm32).

**Integration test** (`tests/browser_plugin.lx`):

This requires agent-browser installed. Use a guard to skip if unavailable:

```lx
use wasm/lx_browser

result = lx_browser.open "https://example.com"
assert result is Ok

snap = lx_browser.snapshot {interactive: true}
assert snap is Ok
assert snap.unwrap.refs != None

url = lx_browser.get_url ()
assert url is Ok

lx_browser.close ()
```

**host_exec sandbox test** (`tests/wasm_exec_sandbox.lx`):

Test that a plugin without `shell = true` in plugin.toml gets an error from host_exec. This requires a second test fixture plugin that does NOT declare `shell = true`.

### 13. SandboxConfig struct update

**File:** `crates/lx/src/stdlib/wasm.rs`

Add `shell` field to `SandboxConfig`:

```rust
#[derive(serde::Deserialize)]
struct SandboxConfig {
    #[serde(default)]
    wasi: Option<bool>,
    #[serde(default)]
    fuel: Option<u64>,
    #[serde(default)]
    shell: Option<bool>,  // <-- new
}
```

Use it when deciding whether to register the real `host_exec` or the error stub.

## Execution Order

1. **host_exec** (task 1) — unblocks everything
2. **SandboxConfig.shell** (task 13 + task 3) — gate host_exec at load time
3. **test_exec fixture** (task 4) — proves host_exec works
4. **Plugin scaffold** (task 5) — directory structure, Cargo.toml, plugin.toml
5. **Plugin internals** (tasks 6, 7, 8) — the actual lx-browser code
6. **Build pipeline** (task 9) — justfile recipes
7. **Pre-built wasm** (task 10) — commit the binary
8. **Binary check** (task 11) — user-friendly error
9. **Tests** (task 12) — unit + integration
10. **Sandbox policy shell flag** (task 2) — complete sandbox integration

## Files Changed / Created

**Changed:**
- `crates/lx/src/stdlib/wasm.rs` — add host_exec, shell gating
- `crates/lx/src/stdlib/sandbox/mod.rs` — add `shell: bool` to Policy
- `crates/lx/src/stdlib/sandbox/sandbox_policy.rs` — presets, parsing, intersection, permits
- `justfile` — add build-plugin, install-plugin recipes

**Created:**
- `plugins/lx-browser/Cargo.toml`
- `plugins/lx-browser/.cargo/config.toml`
- `plugins/lx-browser/plugin.toml`
- `plugins/lx-browser/src/lib.rs`
- `plugins/lx-browser/lx_browser.wasm` (pre-built binary)
- `tests/fixtures/plugins/test_exec/plugin.toml`
- `tests/fixtures/plugins/test_exec/test_exec.wasm`
- `tests/fixtures/plugins/test_exec/src/lib.rs` (source, not shipped)
- `tests/wasm_exec.lx`
- `tests/browser_plugin.lx`
