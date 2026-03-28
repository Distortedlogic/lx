# Work Item 10: JSONL Persistence

Auto-write every event stream entry to `.lx/stream.jsonl` as a persistent durability layer. This runs always -- no configuration needed.

## Prerequisites

- **unit_4_event_stream** must be complete -- provides `EventStream` trait, `StreamEntry`, `IdGenerator`, `JsonlBackend` in `crates/lx/src/runtime/event_stream.rs` and `crates/lx/src/runtime/jsonl_backend.rs`
- **unit_5_stream_module** must be complete -- provides `RuntimeCtx.event_stream` (`parking_lot::Mutex<Option<Arc<dyn EventStream>>>`), `RuntimeCtx.id_gen` (`IdGenerator`), `RuntimeCtx.xadd(entry)` convenience method
- **unit_6_auto_logging** must be complete -- interpreter interception points call `ctx.xadd(entry)` for program start/done, emit, yield, tool calls

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify

## Current State

- `RuntimeCtx` is in `crates/lx/src/runtime/mod.rs` (lines 20-39) with `SmartDefault` derive
- `RuntimeCtx.source_dir` is `parking_lot::Mutex<Option<PathBuf>>` -- the directory containing the program's `.lx` file
- `RuntimeCtx.event_stream` is `parking_lot::Mutex<Option<Arc<dyn EventStream>>>` (added by unit_4_event_stream + unit_5_stream_module)
- `RuntimeCtx.xadd(entry)` is the convenience method: `self.event_stream.lock().as_ref().and_then(|s| s.xadd(entry).ok())` (added by unit_5_stream_module)
- `JsonlBackend` in `crates/lx/src/runtime/jsonl_backend.rs` implements `EventStream` and writes to a JSONL file
- `JsonlBackend::new(path: &str) -> Result<Self, String>` opens (or creates) the file and loads existing entries
- `Interpreter::new` in `crates/lx/src/interpreter/mod.rs` (lines 52-65) sets `*ctx.source_dir.lock() = source_dir.clone()`
- `Interpreter::exec` in `crates/lx/src/interpreter/mod.rs` (lines 91-118) is the program entry point. After unit_6_auto_logging, it calls `ctx.xadd` for `program.start` at the beginning.
- `run::run` in `crates/lx-cli/src/run.rs` creates the interpreter with `source_dir = Path::new(filename).parent()`, then calls `interp.exec(&program)`
- `StreamEntry` has `Serialize` + `Deserialize` derives (serde). Each entry serializes to a single JSON line via `serde_json::to_string`.
- `LxVal`'s `Serialize` impl is in `crates/lx/src/value/serde_impl.rs` and handles all `LxVal` variants.

## Architecture

The JSONL persistence layer is always-on. When the interpreter starts, before executing any program code, it:
1. Determines the JSONL file path: `{source_dir}/.lx/stream.jsonl`
2. Creates the `.lx/` directory if it does not exist
3. Opens the file for append (creates if missing)
4. Creates a `JsonlBackend` and sets it as `RuntimeCtx.event_stream`

Every `ctx.xadd(entry)` call throughout the interpreter's lifetime appends a JSON line to this file. The `JsonlBackend` already handles append + flush on each write.

If a `use stream` statement later provides a custom event stream configuration, it replaces the default JSONL backend. The file written up to that point persists.

## Files to Modify

- `crates/lx/src/interpreter/mod.rs` -- initialize JSONL persistence in `Interpreter::new`

## Step 1: Initialize JSONL persistence on interpreter creation

File: `crates/lx/src/interpreter/mod.rs`

In `Interpreter::new` (lines 52-65), after `*ctx.source_dir.lock() = source_dir.clone();`, add JSONL backend initialization:

Current `Interpreter::new`:
```rust
pub fn new(source: &str, source_dir: Option<PathBuf>, ctx: Arc<RuntimeCtx>) -> Self {
    let env = Env::default();
    crate::builtins::register(&env);
    *ctx.source_dir.lock() = source_dir.clone();
    Self {
        env: Arc::new(env),
        source: source.to_string(),
        source_dir,
        module_cache: Arc::new(Mutex::new(HashMap::new())),
        loading: Arc::new(Mutex::new(HashSet::new())),
        ctx,
        arena: Arc::new(AstArena::new()),
    }
}
```

Change to:

```rust
pub fn new(source: &str, source_dir: Option<PathBuf>, ctx: Arc<RuntimeCtx>) -> Self {
    let env = Env::default();
    crate::builtins::register(&env);
    *ctx.source_dir.lock() = source_dir.clone();

    if ctx.event_stream.lock().is_none() {
        if let Some(ref dir) = source_dir {
            let lx_dir = dir.join(".lx");
            let _ = std::fs::create_dir_all(&lx_dir);
            let jsonl_path = lx_dir.join("stream.jsonl");
            if let Ok(backend) = crate::runtime::JsonlBackend::new(
                &jsonl_path.to_string_lossy(),
            ) {
                *ctx.event_stream.lock() = Some(Arc::new(backend));
            }
        }
    }

    Self {
        env: Arc::new(env),
        source: source.to_string(),
        source_dir,
        module_cache: Arc::new(Mutex::new(HashMap::new())),
        loading: Arc::new(Mutex::new(HashSet::new())),
        ctx,
        arena: Arc::new(AstArena::new()),
    }
}
```

The `is_none()` guard ensures JSONL initialization only happens once -- the top-level interpreter creates the backend, and child interpreters (spawned for module loading via `load_module`) skip it because the event stream is already set on the shared `RuntimeCtx`.

`std::fs::create_dir_all` is a no-op if the directory already exists. The `let _ =` discards the error -- if directory creation fails (e.g. read-only filesystem), the program runs without persistence.

The `JsonlBackend::new` call opens the file for append. If the file already exists (from a previous run), new entries are appended after existing content. The in-memory entries vec in `JsonlBackend` is populated from the existing file content (this is how `JsonlBackend::new` works -- it reads back existing lines on construction).

## Step 2: Verify file path convention

The JSONL file path is `{source_dir}/.lx/stream.jsonl` where `source_dir` is the directory containing the `.lx` program file. For a program at `/home/user/project/main.lx`, the file is `/home/user/project/.lx/stream.jsonl`.

This matches the existing `.lx/` directory convention used for plugins (`source_dir.join(".lx").join("plugins")` in `crates/lx/src/interpreter/modules.rs` line 86) and deps (`root.join(".lx").join("deps")` in `crates/lx-cli/src/manifest.rs` line 136).

## Step 3: Verify file length

`crates/lx/src/interpreter/mod.rs` gains ~10 lines in the constructor. Current file is ~219 lines (after unit_6_auto_logging modifications, ~260 lines). The addition keeps it under 300.

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. All existing tests pass unchanged
3. Write a test:
   - Create a temp directory with a file `test.lx` containing `emit "hello"`
   - Run `lx run test.lx`
   - Check that `{temp_dir}/.lx/stream.jsonl` exists
   - Check that it contains at least one JSON line with `"kind":"program.start"` and one with `"kind":"emit"`
   - Run the program again -- new entries are appended after existing ones
