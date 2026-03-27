# Unit 5: Stream Integration + Auto-Logging

## Goal

Wire the event stream into the interpreter:
1. `use stream {backend: "jsonl", path: "..."}` creates a stream backend and binds `stream` as a module
2. Auto-log at every interception point: tool calls, emit, log, yield, agent ops, program start/done
3. Expose stream methods (xadd, xrange, xread, xlen, xtrim) to lx programs

## Preconditions

- Unit 1 complete: `UseKind::Stream(ExprId)` exists in AST, parser accepts `use stream {config}`
- Unit 3 complete: Tool module dispatch works, tool call execution has a clear call path
- Unit 4 complete: `StreamBackend` trait, `JsonlBackend`, `StreamEntry`, `StreamId` exist in `crates/lx/src/event_stream/`
- `eval_use` at `crates/lx/src/interpreter/modules.rs:15-62`
- Emit interception at `crates/lx/src/interpreter/mod.rs:195-198` (Expr::Emit)
- Yield interception at `crates/lx/src/interpreter/mod.rs:200-203` (Expr::Yield)
- Agent builtins at `crates/lx/src/builtins/agent.rs`
- Log builtins at `crates/lx/src/builtins/register.rs:14-26` (`make_log_builtin` function), `ctx.log.log(level, s)` call at line 17, registered at lines 145-149

## Step 1: Add event stream to RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

Add a field to `RuntimeCtx`:

```rust
pub event_stream: parking_lot::Mutex<Option<Arc<dyn crate::event_stream::StreamBackend>>>,
```

Default: `parking_lot::Mutex::new(None)`. Uses `Mutex` because the field is set during `use stream` evaluation but read from auto-logging callsites. `parking_lot::Mutex` (already a dependency) avoids the `Mutex::lock().unwrap()` dance.

Also add a field to track the current agent name:

```rust
#[default(parking_lot::Mutex::new("main".to_string()))]
pub agent_name: parking_lot::Mutex<String>,
```

This identifies which agent produced each stream entry. The main program is `"main"`. Spawned agents set their own name.

## Step 2: Implement eval_use for UseKind::Stream

File: `crates/lx/src/interpreter/modules.rs`

Replace the `UseKind::Stream` placeholder with:

```rust
UseKind::Stream(config_expr) => {
  let config = self.eval(*config_expr).await.map_err(|e| match e {
    EvalSignal::Error(e) => e,
    EvalSignal::Break(_) => LxError::runtime("break in use stream config", span),
  })?;

  let backend_name = config.str_field("backend")
    .ok_or_else(|| LxError::runtime("use stream: missing 'backend' field", span))?;

  let stream: Arc<dyn crate::event_stream::StreamBackend> = match backend_name {
    "jsonl" => {
      let path = config.str_field("path")
        .ok_or_else(|| LxError::runtime("use stream jsonl: missing 'path' field", span))?;
      let path = std::path::PathBuf::from(path);
      // Resolve relative to source dir
      let abs_path = if path.is_relative() {
        let source_dir = self.source_dir.as_ref()
          .ok_or_else(|| LxError::runtime("cannot resolve relative stream path", span))?;
        source_dir.join(&path)
      } else {
        path
      };
      Arc::new(crate::event_stream::JsonlBackend::new(abs_path)
        .map_err(|e| LxError::runtime(format!("use stream: {e}"), span))?)
    },
    other => {
      return Err(LxError::runtime(
        format!("use stream: unknown backend '{other}' (available: jsonl)"), span
      ));
    },
  };

  // Store in RuntimeCtx's event_stream field (set once, read many)
  // RuntimeCtx is behind Arc and event_stream is Option<Arc<dyn StreamBackend>>.
  // Since RuntimeCtx is constructed before the interpreter runs and passed as Arc,
  // we need interior mutability. Use a Mutex<Option<...>> for the event_stream field.
  // (See Step 1 — the field should be Mutex<Option<Arc<dyn StreamBackend>>>)
  *self.ctx.event_stream.lock() = Some(Arc::clone(&stream));

  // Build stream module bindings using mk_dyn_async from value/func.rs:48-50
  // BuiltinFunc.name must be &'static str — these are fixed method names, no leak needed
  use crate::value::mk_dyn_async;
  let mut bindings = IndexMap::new();

  // xadd: (record) -> Str
  let s1 = Arc::clone(&stream);
  bindings.insert(intern("xadd"), mk_dyn_async("stream.xadd", 1, Arc::new(move |args: Vec<LxVal>, span: miette::SourceSpan, ctx: Arc<crate::runtime::RuntimeCtx>| {
    let s = Arc::clone(&s1);
    Box::pin(async move {
      let record = args[0].require_record("stream.xadd", span)?;
      let kind = record.get(&crate::sym::intern("kind"))
        .and_then(|v| v.as_str())
        .unwrap_or("custom")
        .to_string();
      let mut fields = serde_json::Map::new();
      for (k, v) in record.iter() {
        if k.as_str() != "kind" {
          fields.insert(k.to_string(), serde_json::Value::from(v));
        }
      }
      let entry = crate::event_stream::StreamEntry {
        id: String::new(),
        kind,
        agent: ctx.agent_name.lock().clone(),
        ts: 0,
        span: None,
        fields,
      };
      match s.xadd(entry) {
        Ok(id) => Ok(LxVal::ok(LxVal::str(&id))),
        Err(e) => Ok(LxVal::err_str(&e)),
      }
    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<LxVal, crate::error::LxError>>>>
  })));

  // xrange: (start, end) -> List   [opts as optional 3rd arg handled via arity]
  let s2 = Arc::clone(&stream);
  bindings.insert(intern("xrange"), mk_dyn_async("stream.xrange", 2, Arc::new(move |args: Vec<LxVal>, span: miette::SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
    let s = Arc::clone(&s2);
    Box::pin(async move {
      let start = args[0].require_str("stream.xrange start", span)?;
      let end = args[1].require_str("stream.xrange end", span)?;
      let count = args.get(2).and_then(|v| v.get_field("count")).and_then(|v| v.as_int()).and_then(|n| n.to_usize());
      match s.xrange(start, end, count) {
        Ok(entries) => {
          let list: Vec<LxVal> = entries.into_iter().map(|e| {
            let mut rec = indexmap::IndexMap::new();
            rec.insert(crate::sym::intern("id"), LxVal::str(&e.id));
            rec.insert(crate::sym::intern("kind"), LxVal::str(&e.kind));
            rec.insert(crate::sym::intern("agent"), LxVal::str(&e.agent));
            rec.insert(crate::sym::intern("ts"), LxVal::int(e.ts as i64));
            for (k, v) in e.fields {
              rec.insert(crate::sym::intern(&k), LxVal::from(v));
            }
            LxVal::record(rec)
          }).collect();
          Ok(LxVal::list(list))
        },
        Err(e) => Ok(LxVal::err_str(&e)),
      }
    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<LxVal, crate::error::LxError>>>>
  })));

  // xread: (last_id) -> entry or None
  let s3 = Arc::clone(&stream);
  bindings.insert(intern("xread"), mk_dyn_async("stream.xread", 1, Arc::new(move |args: Vec<LxVal>, span: miette::SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
    let s = Arc::clone(&s3);
    Box::pin(async move {
      let last_id = args[0].require_str("stream.xread", span)?;
      let timeout = args.get(1).and_then(|v| v.get_field("timeout_ms")).and_then(|v| v.as_int()).and_then(|n| n.to_u64());
      match s.xread(last_id, timeout) {
        Ok(Some(entry)) => {
          let mut rec = indexmap::IndexMap::new();
          rec.insert(crate::sym::intern("id"), LxVal::str(&entry.id));
          rec.insert(crate::sym::intern("kind"), LxVal::str(&entry.kind));
          for (k, v) in entry.fields {
            rec.insert(crate::sym::intern(&k), LxVal::from(v));
          }
          Ok(LxVal::some(LxVal::record(rec)))
        },
        Ok(None) => Ok(LxVal::None),
        Err(e) => Ok(LxVal::err_str(&e)),
      }
    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<LxVal, crate::error::LxError>>>>
  })));

  // xlen: () -> Int
  let s4 = Arc::clone(&stream);
  bindings.insert(intern("xlen"), mk_dyn_async("stream.xlen", 0, Arc::new(move |_args: Vec<LxVal>, _span: miette::SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
    let s = Arc::clone(&s4);
    Box::pin(async move {
      match s.xlen() {
        Ok(n) => Ok(LxVal::int(n as i64)),
        Err(e) => Ok(LxVal::err_str(&e)),
      }
    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<LxVal, crate::error::LxError>>>>
  })));

  // xtrim: (opts) -> Int
  let s5 = Arc::clone(&stream);
  bindings.insert(intern("xtrim"), mk_dyn_async("stream.xtrim", 1, Arc::new(move |args: Vec<LxVal>, span: miette::SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
    let s = Arc::clone(&s5);
    Box::pin(async move {
      let maxlen = args[0].get_field("maxlen")
        .ok_or_else(|| crate::error::LxError::runtime("stream.xtrim: missing 'maxlen' field", span))?
        .require_usize("stream.xtrim maxlen", span)?;
      match s.xtrim(maxlen) {
        Ok(n) => Ok(LxVal::int(n as i64)),
        Err(e) => Ok(LxVal::err_str(&e)),
      }
    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<LxVal, crate::error::LxError>>>>
  })));

  let record = LxVal::record(bindings);
  let env = self.env.child();
  env.bind(intern("stream"), record);
  self.env = Arc::new(env);
  return Ok(());
}
```

### Stream module methods as builtins

For each stream method, create a `BuiltinFunc` (or `LxVal::BuiltinFunc`) that captures the `Arc<dyn StreamBackend>` and calls the appropriate trait method. The pattern is the same as how stdlib builtins work — check `crates/lx/src/stdlib/` for examples of how builtins are registered.

Key methods:

**xadd**: Takes a record argument, converts to `StreamEntry`, calls `stream.xadd()`, returns the ID string.
```rust
// args[0] is the record to append
// Extract "kind" field (required), plus any other fields
// Set agent = ctx.agent_name, ts = now, span = caller span
```

**xrange**: Takes start (Str), end (Str), optional opts record with `count` field.

**xread**: Takes last_id (Str), optional opts record with `timeout_ms` field.

**xlen**: No arguments, returns Int.

**xtrim**: Takes opts record with `maxlen` field, returns Int (number trimmed).

## Step 3: Auto-logging helper

File: `crates/lx/src/event_stream/mod.rs` (or a new file `crates/lx/src/event_stream/auto_log.rs`)

Create a helper function that builds and appends a `StreamEntry`:

```rust
use crate::runtime::RuntimeCtx;
use std::sync::Arc;

pub fn auto_log(ctx: &RuntimeCtx, kind: &str, fields: serde_json::Map<String, serde_json::Value>, span: Option<miette::SourceSpan>) {
  let stream_guard = ctx.event_stream.lock();
  let Some(ref stream) = *stream_guard else { return };
  let span_info = span.map(|s| SpanInfo {
    line: s.offset() as u32,
    col: 0,
  });
  let entry = StreamEntry {
    id: String::new(),
    kind: kind.to_string(),
    agent: ctx.agent_name.lock().clone(),
    ts: 0,
    span: span_info,
    fields,
  };
  // Intentionally ignore errors — auto-logging must never crash the program
  let _ = stream.xadd(entry);
}
```

## Step 4: Auto-log at interpreter interception points

### 4a: program.start and program.done

File: `crates/lx/src/interpreter/mod.rs`

In `exec()` (line 91), add at the beginning:
```rust
crate::event_stream::auto_log(&self.ctx, "program.start", serde_json::Map::new(), None);
```

And after the main loop completes (before returning `result`):
```rust
let fields = serde_json::Map::from_iter([
  ("duration_ms".to_string(), serde_json::json!(/* track duration */)),
]);
crate::event_stream::auto_log(&self.ctx, "program.done", fields, None);
```

### 4b: emit

File: `crates/lx/src/interpreter/mod.rs`, lines 195-198

Current:
```rust
Expr::Emit(ExprEmit { value }) => {
  let v = self.eval(value).await?;
  self.ctx.emit.emit(&v, span)?;
  Ok(LxVal::Unit)
}
```

Add after the `emit` call:
```rust
let fields = serde_json::Map::from_iter([
  ("value".to_string(), serde_json::Value::from(&v)),
]);
crate::event_stream::auto_log(&self.ctx, "emit", fields, Some(span));
```

### 4c: yield

File: `crates/lx/src/interpreter/mod.rs`, lines 200-203

Add before the yield call (yield.out) and after (yield.in):
```rust
Expr::Yield(ExprYield { value }) => {
  let v = self.eval(value).await?;
  // auto-log yield.out
  let out_fields = serde_json::Map::from_iter([
    ("value".to_string(), serde_json::Value::from(&v)),
  ]);
  crate::event_stream::auto_log(&self.ctx, "yield.out", out_fields, Some(span));

  let response = self.ctx.yield_.yield_value(v, span)?;

  // auto-log yield.in
  let in_fields = serde_json::Map::from_iter([
    ("response".to_string(), serde_json::Value::from(&response)),
  ]);
  crate::event_stream::auto_log(&self.ctx, "yield.in", in_fields, Some(span));

  Ok(response)
}
```

### 4d: tool calls

Add a `tool_call_counter` field to `RuntimeCtx`:
```rust
pub tool_call_counter: std::sync::atomic::AtomicU64,
```
Default: `AtomicU64::new(0)` (add `#[default(std::sync::atomic::AtomicU64::new(0))]`).

Then modify the tool call dispatch code in `crates/lx/src/interpreter/modules.rs` (the `BuiltinFunc` created in Unit 3's Step 2 for each tool method). Wrap the `call_tool` invocation with auto-logging:

```rust
// Inside the DynAsync closure for each tool method:
let call_id = ctx.tool_call_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
let args_json = serde_json::Value::from(&args[0]);

// Log tool.call
let mut call_fields = serde_json::Map::new();
call_fields.insert("call_id".into(), serde_json::json!(call_id));
call_fields.insert("tool".into(), serde_json::json!(tool_module_name));
call_fields.insert("method".into(), serde_json::json!(method_name));
call_fields.insert("args".into(), args_json.clone());
crate::event_stream::auto_log(&ctx, "tool.call", call_fields, Some(span));

// Execute the actual MCP call (use .await, NOT block_on — we're inside async context)
let c = client_ref.lock().await;
let result = c.call_tool(&method_name, args_json).await;
drop(c);

// Log tool.result or tool.error
match &result {
  Ok(json) => {
    let mut fields = serde_json::Map::new();
    fields.insert("call_id".into(), serde_json::json!(call_id));
    fields.insert("tool".into(), serde_json::json!(tool_module_name));
    fields.insert("method".into(), serde_json::json!(method_name));
    fields.insert("result".into(), json.clone());
    crate::event_stream::auto_log(&ctx, "tool.result", fields, Some(span));
  },
  Err(e) => {
    let mut fields = serde_json::Map::new();
    fields.insert("call_id".into(), serde_json::json!(call_id));
    fields.insert("tool".into(), serde_json::json!(tool_module_name));
    fields.insert("method".into(), serde_json::json!(method_name));
    fields.insert("error".into(), serde_json::json!(e));
    crate::event_stream::auto_log(&ctx, "tool.error", fields, Some(span));
  },
}
```

This means the tool call closure in Unit 3's Step 2 needs to capture `tool_module_name` (the alias, e.g. "Browser"). Add this capture to Unit 3's builtin creation code. The `_ctx` parameter in the closure signature should be renamed to `ctx` since it's now used.

### 4e: log builtins

File: `crates/lx/src/builtins/register.rs:14-26`

The `make_log_builtin` function creates each log variant. The `ctx.log.log(level, s)` call is at line 17. After that call (line 17), add auto-logging:

```rust
fn log_fn(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>, level: LogLevel, name: &str) -> Result<LxVal, LxError> {
  let s = args[0].require_str(&format!("log.{name}"), span)?;
  ctx.log.log(level, s);
  // Auto-log to event stream
  let mut fields = serde_json::Map::new();
  fields.insert("level".into(), serde_json::json!(name));
  fields.insert("msg".into(), serde_json::json!(s));
  crate::event_stream::auto_log(ctx, "log", fields, Some(span));
  Ok(LxVal::Unit)
}
```

### 4f: agent builtins

File: `crates/lx/src/builtins/agent.rs`

After each agent operation, add auto-logging:

- `bi_agent_spawn` (line 24): after successful spawn, log `agent.spawn` with `agent_id` and `script`
- `bi_agent_kill` (line 78): log `agent.kill` with `agent_id`
- `bi_agent_ask` (line 86): log `agent.ask` before send, log `agent.response` after receive
- `bi_agent_tell` (line 107): log `agent.tell`

Each auto-log call follows the same pattern as Step 3's `auto_log` function.

## Step 5: Set agent_name for spawned agents

File: `crates/lx/src/builtins/agent.rs`

At line 24, the `_ctx` parameter is named with underscore (unused). Rename it to `ctx` so it can be used.

At lines 55-56, the spawned agent creates a fresh `RuntimeCtx` with `..RuntimeCtx::default()`. Modify to propagate the event stream and set the agent name:

```rust
let parent_event_stream = ctx.event_stream.lock().clone();
// ... (inside the spawn_blocking closure):
let ctx = Arc::new(RuntimeCtx {
  source_dir: parking_lot::Mutex::new(source_dir),
  yield_: yield_backend,
  tokio_runtime: Arc::new(rt),
  agent_name: parking_lot::Mutex::new(format!("agent-{id}")),
  event_stream: parking_lot::Mutex::new(parent_event_stream),
  ..RuntimeCtx::default()
});
```

The `parent_event_stream` must be cloned BEFORE the `spawn_blocking` closure (since `Arc<dyn StreamBackend>` is `Send + Sync`, this works across thread boundaries). Clone it at line 47 (after `id` is assigned, before `spawn_blocking`).

## Verification

1. Run `just diagnose` — no errors or warnings
2. Write a test `.lx` program:
```lx
use stream {backend: "jsonl", path: ".lx/test_events.jsonl"}
emit "hello"
entries = stream.xrange "-" "+"
-- should have program.start + emit entries
```
3. Check that `.lx/test_events.jsonl` contains valid JSONL entries with correct kind/agent/ts fields
