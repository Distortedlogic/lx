# Unit 5: Stream Module -- Interpreter Integration

Wire `use stream {config}` into the interpreter: evaluate config, create JSONL backend, bind stream module with xadd/xrange/xread/xlen/xtrim methods. Build resume/replay cache from existing stream.

## Prerequisites

- **Unit 2** (Parser) must be complete -- provides `UseKind::Stream(ExprId)` variant
- **Unit 4** (EventStream Trait) must be complete -- provides `EventStream` trait, `StreamEntry`, `JsonlBackend`

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify

## Current State

- `eval_use` is in `crates/lx/src/interpreter/modules.rs` (lines 15-61)
- Unit 2 added a placeholder: `if let UseKind::Stream(_) = ... { return Err(...) }`
- `RuntimeCtx` has `event_stream: Option<Arc<dyn EventStream>>` (added by Unit 4)
- `LxVal::BuiltinFunc` is used for builtin methods bound as record fields
- `BuiltinFunc.name` is `&'static str` (not `Sym`)
- `BuiltinKind::DynAsync(f)` for async variants that capture state
- `mk_dyn_async(name: &'static str, arity: usize, func: DynAsyncBuiltinFn) -> LxVal` returns an `LxVal` wrapping a `BuiltinFunc`
- `LxVal::from(serde_json::Value)` and `serde_json::Value::from(&LxVal)` conversions exist in `serde_impl.rs`
- `crate::sym::intern` interns string slices into `Sym`
- `IdGenerator` implements `Default` (via explicit `impl Default`)

## Files to Create

- `crates/lx/src/interpreter/stream_module.rs` -- stream binding construction and replay cache (extracted because modules.rs is 261 lines and adding ~100 lines for stream bindings + replay cache would exceed 300)

## Files to Modify

- `crates/lx/src/interpreter/modules.rs` -- implement `UseKind::Stream` in `eval_use`
- `crates/lx/src/interpreter/mod.rs` -- add `mod stream_module;`
- `crates/lx/src/runtime/mod.rs` -- add `replay_cache` field to RuntimeCtx, add `id_gen` field, change `event_stream` to use interior mutability

## Step 1: Add replay_cache, call_id_counter, and id_gen to RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

Add fields to `RuntimeCtx` for the resume/replay cache and shared ID generator.

Add after the `event_stream` field:
```rust
#[default(Arc::new(parking_lot::Mutex::new(HashMap::new())))]
pub replay_cache: Arc<parking_lot::Mutex<HashMap<u64, crate::value::LxVal>>>,
#[default(Arc::new(std::sync::atomic::AtomicU64::new(1)))]
pub call_id_counter: Arc<std::sync::atomic::AtomicU64>,
pub id_gen: IdGenerator,
```

`SmartDefault` uses `IdGenerator::default()` automatically since `IdGenerator` implements `Default`.

Change the `event_stream` field type to use interior mutability (it is set during program execution when `use stream` is evaluated, but `RuntimeCtx` is behind `Arc`):

From:
```rust
pub event_stream: Option<Arc<dyn EventStream>>,
```

To:
```rust
pub event_stream: parking_lot::Mutex<Option<Arc<dyn EventStream>>>,
```

Update the `xadd` helper accordingly:

```rust
impl RuntimeCtx {
    pub fn xadd(&self, entry: StreamEntry) -> Option<String> {
        self.event_stream.lock().as_ref().and_then(|s| s.xadd(entry).ok())
    }
}
```

## Step 2: Implement UseKind::Stream in eval_use

File: `crates/lx/src/interpreter/modules.rs`

Replace the placeholder `UseKind::Stream` handling with the real implementation.

The early-return guard from Unit 2 looks like:
```rust
if let UseKind::Stream(_) = use_stmt.kind {
    return Err(LxError::runtime("use stream not yet implemented", span));
}
```

Replace with:

```rust
if let UseKind::Stream(config_expr) = use_stmt.kind {
    let config = self.eval(config_expr).await.map_err(|e| match e {
        crate::error::EvalSignal::Error(e) => e,
        crate::error::EvalSignal::Break(_) => LxError::runtime("break in use stream config", span),
    })?;

    let backend_str = config.str_field("backend")
        .ok_or_else(|| LxError::runtime("use stream: config must have 'backend' field of type Str", span))?;

    let stream: Arc<dyn crate::runtime::EventStream> = match backend_str {
        "jsonl" => {
            let path = config.str_field("path")
                .ok_or_else(|| LxError::runtime("use stream: jsonl backend requires 'path' field", span))?;
            let backend = crate::runtime::JsonlBackend::new(path)
                .map_err(|e| LxError::runtime(format!("use stream: {e}"), span))?;
            Arc::new(backend)
        },
        other => {
            return Err(LxError::runtime(
                format!("use stream: unknown backend '{other}' (available: jsonl)"),
                span,
            ));
        },
    };

    self.build_replay_cache(&stream, span)?;

    *self.ctx.event_stream.lock() = Some(Arc::clone(&stream));

    let bindings = self.build_stream_bindings(Arc::clone(&stream));
    let module_record = LxVal::record(bindings);
    let env = self.env.child();
    env.bind(crate::sym::intern("stream"), module_record);
    self.env = Arc::new(env);
    return Ok(());
}
```

## Step 3: Create stream_module.rs

File: `crates/lx/src/interpreter/stream_module.rs`

After Units 2 and 3 modify modules.rs, it is ~290 lines. Adding stream bindings and replay cache would exceed 300. Create `stream_module.rs` as a separate file.

Add to `crates/lx/src/interpreter/mod.rs` (after `mod modules;`):
```rust
mod stream_module;
```

### build_replay_cache method

```rust
use std::sync::Arc;

use indexmap::IndexMap;
use miette::SourceSpan;

use crate::error::LxError;
use crate::value::LxVal;

impl super::Interpreter {
    pub(super) fn build_replay_cache(
        &self,
        stream: &Arc<dyn crate::runtime::EventStream>,
        span: SourceSpan,
    ) -> Result<(), LxError> {
        let entries = stream.xrange("-", "+", None)
            .map_err(|e| LxError::runtime(format!("replay cache: {e}"), span))?;

        let mut cache = self.ctx.replay_cache.lock();
        cache.clear();

        let mut call_entries: std::collections::HashMap<u64, serde_json::Value> =
            std::collections::HashMap::new();

        for entry in &entries {
            if entry.kind == "tool.call" {
                if let Some(call_id_val) = entry.fields.get("call_id") {
                    if let Some(call_id) = call_id_val.as_u64() {
                        call_entries.insert(call_id, serde_json::Value::Null);
                    }
                }
            }
            if entry.kind == "tool.result" {
                if let Some(call_id_val) = entry.fields.get("call_id") {
                    if let Some(call_id) = call_id_val.as_u64() {
                        if call_entries.contains_key(&call_id) {
                            if let Some(result_val) = entry.fields.get("result") {
                                cache.insert(call_id, LxVal::from(result_val.clone()));
                            }
                        }
                    }
                }
            }
        }

        self.ctx.call_id_counter.store(1, std::sync::atomic::Ordering::SeqCst);

        Ok(())
    }
```

### build_stream_bindings method

```rust
    pub(super) fn build_stream_bindings(
        &self,
        stream: Arc<dyn crate::runtime::EventStream>,
    ) -> IndexMap<crate::sym::Sym, LxVal> {
        use crate::sym::intern;

        let mut bindings = IndexMap::new();

        let s = Arc::clone(&stream);
        bindings.insert(intern("xadd"), crate::value::mk_dyn_async(
            "xadd",
            1,
            Arc::new(move |args: Vec<LxVal>, span: SourceSpan, ctx: Arc<crate::runtime::RuntimeCtx>| {
                let s = Arc::clone(&s);
                Box::pin(async move {
                    let record = args[0].require_record("stream.xadd", span)?;
                    let kind = record.get(&intern("kind"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("custom")
                        .to_string();
                    let mut entry = crate::runtime::StreamEntry::new(&kind, "main", &ctx.id_gen);
                    for (k, v) in record.iter() {
                        if k.as_str() != "kind" {
                            entry.fields.insert(
                                k.as_str().to_string(),
                                serde_json::Value::from(v),
                            );
                        }
                    }
                    match s.xadd(entry) {
                        Ok(id) => Ok(LxVal::str(id)),
                        Err(e) => Ok(LxVal::err_str(e)),
                    }
                })
            }),
        ));

        let s = Arc::clone(&stream);
        bindings.insert(intern("xrange"), crate::value::mk_dyn_async(
            "xrange",
            2,
            Arc::new(move |args: Vec<LxVal>, span: SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
                let s = Arc::clone(&s);
                Box::pin(async move {
                    let start = args[0].require_str("stream.xrange start", span)?;
                    let end = args[1].require_str("stream.xrange end", span)?;
                    match s.xrange(start, end, None) {
                        Ok(entries) => {
                            let vals: Vec<LxVal> = entries.iter().map(|e| e.to_lx_val()).collect();
                            Ok(LxVal::list(vals))
                        },
                        Err(e) => Ok(LxVal::err_str(e)),
                    }
                })
            }),
        ));

        let s = Arc::clone(&stream);
        bindings.insert(intern("xread"), crate::value::mk_dyn_async(
            "xread",
            1,
            Arc::new(move |args: Vec<LxVal>, span: SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
                let s = Arc::clone(&s);
                Box::pin(async move {
                    let last_id = args[0].require_str("stream.xread last_id", span)?;
                    match s.xread(last_id, None) {
                        Ok(Some(entry)) => Ok(entry.to_lx_val()),
                        Ok(None) => Ok(LxVal::None),
                        Err(e) => Ok(LxVal::err_str(e)),
                    }
                })
            }),
        ));

        let s = Arc::clone(&stream);
        bindings.insert(intern("xlen"), crate::value::mk_dyn_async(
            "xlen",
            1,
            Arc::new(move |_args: Vec<LxVal>, _span: SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
                let s = Arc::clone(&s);
                Box::pin(async move {
                    match s.xlen() {
                        Ok(n) => Ok(LxVal::int(n as i64)),
                        Err(e) => Ok(LxVal::err_str(e)),
                    }
                })
            }),
        ));

        let s = Arc::clone(&stream);
        bindings.insert(intern("xtrim"), crate::value::mk_dyn_async(
            "xtrim",
            1,
            Arc::new(move |args: Vec<LxVal>, span: SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
                let s = Arc::clone(&s);
                Box::pin(async move {
                    let maxlen = args[0].get_field("maxlen")
                        .and_then(|v| v.as_int())
                        .and_then(|n| num_traits::ToPrimitive::to_usize(n))
                        .ok_or_else(|| crate::error::LxError::runtime(
                            "stream.xtrim requires {maxlen: Int}",
                            span,
                        ))?;
                    match s.xtrim(maxlen) {
                        Ok(n) => Ok(LxVal::int(n as i64)),
                        Err(e) => Ok(LxVal::err_str(e)),
                    }
                })
            }),
        ));

        bindings
    }
}
```

Note on `num_traits::ToPrimitive`: The `to_usize()` method in `xtrim` comes from the `ToPrimitive` trait. Add `use num_traits::ToPrimitive;` at the top of the file if not already present.

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. All existing tests pass unchanged
3. Write a test `.lx` file:
```lx
use stream {backend: "jsonl", path: "/tmp/lx_test_events.jsonl"}
stream.xadd {kind: "test", msg: "hello"}
entries = stream.xrange "-" "+"
assert (entries | length) == 1
```
This verifies end-to-end: parsing, config evaluation, backend creation, module binding, xadd, xrange.
