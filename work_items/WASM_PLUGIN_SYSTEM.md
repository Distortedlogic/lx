# Goal

Add WASM plugin support to lx via Extism. Developers write Rust (or any language), compile to `wasm32-unknown-unknown`, place in `~/.lx/plugins/` or `.lx/plugins/`, and lx loads them as modules via `use wasm/plugin_name`. This is the generic extension mechanism for compiled code — the equivalent of Python C extensions.

# Why

lx has no way for external developers to extend the runtime with compiled code. The only option is editing the lx source and recompiling. WASM plugins let developers write fast, sandboxed extensions that lx loads at runtime.

This also enables migrating pure-compute stdlib modules (`std/json`, `std/re`, `std/schema`, `std/md`, `std/math`) out of the binary into plugins.

# Critical: BuiltinFunc Cannot Capture State

The current `BuiltinFunc` system uses bare `fn` pointers (`SyncBuiltinFn` / `AsyncBuiltinFn` in `crates/lx/src/value/func.rs`):

```rust
pub type SyncBuiltinFn = fn(&[LxVal], SourceSpan, &Arc<RuntimeCtx>) -> Result<LxVal, LxError>;
pub type AsyncBuiltinFn = fn(Vec<LxVal>, SourceSpan, Arc<RuntimeCtx>) -> Pin<Box<dyn Future<...>>>;

#[derive(Clone, Copy)]
pub enum BuiltinKind {
  Sync(SyncBuiltinFn),
  Async(AsyncBuiltinFn),
}
```

`fn` pointers cannot capture state. A WASM plugin wrapper needs to capture the plugin name and function name per-export. **This means `BuiltinKind` must be extended to support closures.**

Add a new variant:

```rust
pub enum BuiltinKind {
  Sync(SyncBuiltinFn),
  Async(AsyncBuiltinFn),
  DynAsync(Arc<dyn Fn(Vec<LxVal>, SourceSpan, Arc<RuntimeCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> + Send + Sync>),
}
```

`BuiltinKind` currently derives `Clone, Copy` — adding `Arc` means it can no longer be `Copy`. Change to `Clone` only. This affects `BuiltinFunc` which is `#[derive(Clone)]` — no issue there. Check all pattern matches on `BuiltinKind` in `crates/lx/src/interpreter/apply.rs` — the `DynAsync` arm dispatches identically to `Async`.

# Dependencies

- `extism = "1.20.0"` — host SDK (wraps wasmtime)
- `extism-pdk = "1.4.1"` — plugin SDK (for plugin authors, not added to lx itself)
- `toml = "0.8"` — for parsing plugin.toml (check if already in Cargo.toml)

# What Changes

**`crates/lx/src/value/func.rs`:**
- Add `DynAsync` variant to `BuiltinKind`
- Remove `Copy` derive from `BuiltinKind`
- Add `mk_dyn_async` factory function that creates `BuiltinFunc` with `DynAsync` kind

**`crates/lx/src/interpreter/apply.rs`:**
- Add `BuiltinKind::DynAsync(f)` arm to the match in `apply_func()`, dispatching same as `Async`

**`crates/lx/src/stdlib/wasm_marshal.rs`** (new file):
- `lxval_to_json(val: &LxVal) -> Result<String, LxError>` — converts LxVal to JSON string
- `json_to_lxval(json: &str) -> Result<LxVal, LxError>` — converts JSON string back to LxVal
- Mapping: Int→number (i64 range) or string (BigInt), Float→number, Bool→boolean, Str→string, List→array, Record→object, None→null, Ok(v)→`{"Ok": v}`, Err(e)→`{"Err": e}`, Some(v)→v, Unit→null, Tagged→`{"_tag": name, "values": [...]}`

**`crates/lx/src/stdlib/wasm.rs`** (new file):
- `static PLUGINS: LazyLock<RwLock<HashMap<String, Mutex<Plugin>>>>` — global plugin registry. `Mutex<Plugin>` because `Plugin::call` takes `&mut self`.
- `load_plugin(name: &str, plugin_dir: &Path, span: SourceSpan) -> Result<ModuleExports, LxError>`:
  1. Read `plugin.toml` from `plugin_dir`
  2. Parse manifest (name, version, wasm path, exports)
  3. Resolve wasm file path relative to plugin_dir
  4. Create `Manifest::new([Wasm::file(wasm_path)]).with_wasi(false)`
  5. Create `Plugin::new(&manifest, [], false)`
  6. Store in PLUGINS map
  7. For each export in manifest, create a `BuiltinFunc` with `DynAsync` kind that:
     - Serializes args via `lxval_to_json`
     - Acquires PLUGINS lock, gets `&mut Plugin`
     - Calls `plugin.call::<&str, &str>(fn_name, &json_input)`
     - Deserializes result via `json_to_lxval`
  8. Return `ModuleExports { bindings, variant_ctors: vec![] }`
- Host functions registered for all plugins:
  - `plugin_log(level: u32, msg: String)` — routes to lx log backend
  - `plugin_get_config(key: String) -> String` — reads env var

**`crates/lx/src/interpreter/modules.rs`** — in `eval_use()`:
- After std module check, before workspace check:
  ```rust
  if str_path.starts_with("wasm/") {
      let plugin_name = &str_path[5..];
      return self.load_wasm_plugin(plugin_name, span);
  }
  ```
- `load_wasm_plugin(name, span)`:
  1. Check module_cache first
  2. Search `.lx/plugins/{name}/` relative to source_dir
  3. Search `~/.lx/plugins/{name}/`
  4. If found, call `wasm::load_plugin(name, dir, span)`
  5. Cache and return

**Plugin manifest format** — `plugin.toml`:
```toml
[plugin]
name = "json"
version = "0.1.0"
description = "JSON parsing and encoding"
wasm = "json.wasm"

[exports]
parse = { arity = 1 }
encode = { arity = 1 }

[sandbox]
wasi = false
fuel = 1000000
```

**Plugin directory layout:**
```
~/.lx/plugins/json/
├── plugin.toml
└── json.wasm
```

# Gotchas

- `Plugin::call` takes `&mut self` — the global PLUGINS map needs `Mutex<Plugin>` per entry, not just `RwLock` on the map. Multiple lx functions from the same plugin could be called concurrently.
- `BuiltinKind` losing `Copy` is a small ripple. Grep for `BuiltinKind` copies — there's one in `apply.rs` that does `match bf.kind { ... }`. Since `BuiltinKind::Sync` and `Async` contain `fn` pointers (which are Copy), they'll still work. Only `DynAsync` with `Arc` needs `.clone()`. The match can destructure by reference for `DynAsync`.
- First call to a WASM plugin pays compilation cost (~5-50ms depending on module size). Subsequent calls are fast (<1ms). Plugin compilation could be done eagerly at load time rather than lazily.
- Extism `Plugin::new` can fail if the WASM module is invalid. The error should include the plugin name and path for debuggability.
- `~/.lx/` directory may not exist. `load_wasm_plugin` should handle this gracefully (just means no global plugins found, continue searching).

# Task List

### Task 1: Extend BuiltinKind with DynAsync
Edit `crates/lx/src/value/func.rs`. Add `DynAsync(Arc<dyn Fn(...) -> Pin<Box<...>> + Send + Sync>)` variant. Remove `Copy` from `BuiltinKind`. Add `mk_dyn_async` factory. Edit `crates/lx/src/interpreter/apply.rs` — add `DynAsync` match arm. Run `just diagnose` to find and fix any compilation errors from removing `Copy`.

### Task 2: Implement JSON marshaling
Create `crates/lx/src/stdlib/wasm_marshal.rs`. Implement `lxval_to_json` and `json_to_lxval`. Write unit tests for round-trip marshaling of all LxVal types. Add `pub mod wasm_marshal;` to `stdlib/mod.rs`.

### Task 3: Add Extism dependency
Add `extism = "1.20.0"` and `toml = "0.8"` (if not present) to `crates/lx/Cargo.toml`. Verify compilation.

### Task 4: Implement plugin manager
Create `crates/lx/src/stdlib/wasm.rs`. Implement `load_plugin` with PLUGINS global, manifest parsing, `DynAsync` builtin creation, host function registration. Add `pub mod wasm;` to `stdlib/mod.rs`.

### Task 5: Add `use wasm/` resolution path
Edit `crates/lx/src/interpreter/modules.rs`. Add `wasm/` prefix handling in `eval_use()`. Implement `load_wasm_plugin` with directory search and caching.

### Task 6: Create test plugin and integration test
Create `tests/fixtures/plugins/test_upper/` with a minimal Rust→WASM plugin that has one function: `upper(text: String) -> String`. Include pre-compiled `.wasm` file in the fixture (so tests don't require wasm toolchain). Create `plugin.toml`. Write `tests/wasm_plugin.lx` that does `use wasm/test_upper` then `assert (test_upper.upper "hello") == "HELLO"`. The test's `.lx/plugins/test_upper/` symlinks to the fixture.

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/WASM_PLUGIN_SYSTEM.md" })
```
