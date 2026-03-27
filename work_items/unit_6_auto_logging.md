# Unit 6: Auto-logging Interception

Add automatic event stream logging at key interpreter interception points: program start/done, emit, yield, tool calls, agent operations.

## Prerequisites

- **Unit 3** (Tool Module) must be complete -- tool dispatch path exists
- **Unit 4** (EventStream) must be complete -- `EventStream` trait, `StreamEntry`, `IdGenerator` exist
- **Unit 5** (Stream Module) must be complete -- `RuntimeCtx.event_stream`, `RuntimeCtx.replay_cache`, `RuntimeCtx.call_id_counter` exist

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify

## Current State

- `Interpreter::exec` is in `crates/lx/src/interpreter/mod.rs` (lines 91-118)
- `Expr::Emit` handling is in `crates/lx/src/interpreter/mod.rs` (lines 195-199)
- `Expr::Yield` handling is in `crates/lx/src/interpreter/mod.rs` (lines 200-203)
- Tool dispatch closures are created in `crates/lx/src/interpreter/modules.rs` inside the `UseKind::Tool` handler (added by Unit 3)
- `RuntimeCtx.event_stream` is `parking_lot::Mutex<Option<Arc<dyn EventStream>>>` (added by Units 4+5)
- `RuntimeCtx.xadd(entry)` is the convenience method that checks if stream is active (added by Unit 4)
- `RuntimeCtx.call_id_counter` is `Arc<AtomicU64>` starting at 1 (added by Unit 5)
- `RuntimeCtx.replay_cache` is `Arc<parking_lot::Mutex<HashMap<u64, LxVal>>>` (added by Unit 5)
- `RuntimeCtx.id_gen` is `IdGenerator` (added by Unit 5)
- `BuiltinFunc.name` is `&'static str` (not `Sym`)
- `mk_dyn_async(name: &'static str, arity: usize, func: DynAsyncBuiltinFn) -> LxVal` returns an `LxVal` wrapping a `BuiltinFunc`
- `StreamEntry::new(kind, agent, id_gen)` creates an entry with auto-generated ID and timestamp
- `StreamEntry::with_field(key, serde_json::Value)` adds a field
- `SpanInfo { line, col }` for source location

## Files to Modify

- `crates/lx/src/interpreter/mod.rs` -- program.start/program.done in `exec`, emit/yield interception in `eval`
- `crates/lx/src/interpreter/tool_module.rs` -- tool.call/tool.result/tool.error in tool dispatch closure (created by Unit 3)
- `crates/lx/src/runtime/event_stream.rs` -- add `SpanInfo::from_source_span` helper

## Step 1: Add span conversion helper

File: `crates/lx/src/runtime/event_stream.rs`

Add an impl block on `SpanInfo`:

```rust
impl SpanInfo {
    pub fn from_source_span(span: miette::SourceSpan, source: &str) -> Self {
        let offset = span.offset();
        let mut line = 1u32;
        let mut col = 1u32;
        for (i, ch) in source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        Self { line, col }
    }
}
```

## Step 2: program.start and program.done in exec

File: `crates/lx/src/interpreter/mod.rs`

Modify the `exec` method (lines 91-118). Add `program.start` event before execution and `program.done` after.

Current `exec` method:
```rust
pub async fn exec(&mut self, program: &Program<Core>) -> Result<LxVal, LxError> {
    self.arena = Arc::new(program.arena.clone());
    let mut forward_names = Vec::new();
    for &sid in &program.stmts {
        if let Stmt::Binding(b) = self.arena.stmt(sid)
            && let BindTarget::Name(name) = b.target
            && matches!(self.arena.expr(b.value), Expr::Func(_))
        {
            forward_names.push(name);
        }
    }
    if !forward_names.is_empty() {
        let env = self.env.child();
        for name in &forward_names {
            env.bind_mut(*name, LxVal::Unit);
        }
        self.env = Arc::new(env);
    }
    let mut result = LxVal::Unit;
    let stmts = program.stmts.clone();
    for sid in &stmts {
        result = self.eval_stmt(*sid).await.map_err(|e| match e {
            EvalSignal::Error(e) => e,
            EvalSignal::Break(_) => LxError::runtime("break outside loop", self.arena.stmt_span(*sid)),
        })?;
    }
    Ok(result)
}
```

Change to:

```rust
pub async fn exec(&mut self, program: &Program<Core>) -> Result<LxVal, LxError> {
    self.arena = Arc::new(program.arena.clone());

    let start_ts = std::time::Instant::now();
    {
        let source_path = self.source_dir.as_ref()
            .map(|d| d.display().to_string())
            .unwrap_or_default();
        let mut entry = crate::runtime::StreamEntry::new("program.start", "main", &self.ctx.id_gen);
        entry = entry.with_field("source_path", serde_json::Value::String(source_path));
        self.ctx.xadd(entry);
    }

    let mut forward_names = Vec::new();
    for &sid in &program.stmts {
        if let Stmt::Binding(b) = self.arena.stmt(sid)
            && let BindTarget::Name(name) = b.target
            && matches!(self.arena.expr(b.value), Expr::Func(_))
        {
            forward_names.push(name);
        }
    }
    if !forward_names.is_empty() {
        let env = self.env.child();
        for name in &forward_names {
            env.bind_mut(*name, LxVal::Unit);
        }
        self.env = Arc::new(env);
    }
    let mut result = LxVal::Unit;
    let stmts = program.stmts.clone();
    for sid in &stmts {
        result = self.eval_stmt(*sid).await.map_err(|e| match e {
            EvalSignal::Error(e) => e,
            EvalSignal::Break(_) => LxError::runtime("break outside loop", self.arena.stmt_span(*sid)),
        })?;
    }

    {
        let duration_ms = start_ts.elapsed().as_millis() as u64;
        let result_json = serde_json::Value::from(&result);
        let mut entry = crate::runtime::StreamEntry::new("program.done", "main", &self.ctx.id_gen);
        entry = entry.with_field("result", result_json);
        entry = entry.with_field("duration_ms", serde_json::Value::Number(duration_ms.into()));
        self.ctx.xadd(entry);
    }

    Ok(result)
}
```

## Step 3: emit interception

File: `crates/lx/src/interpreter/mod.rs`

Current emit handling (lines 195-199):
```rust
Expr::Emit(ExprEmit { value }) => {
    let v = self.eval(value).await?;
    self.ctx.emit.emit(&v, span)?;
    Ok(LxVal::Unit)
},
```

Change to:
```rust
Expr::Emit(ExprEmit { value }) => {
    let v = self.eval(value).await?;
    self.ctx.emit.emit(&v, span)?;
    {
        let mut entry = crate::runtime::StreamEntry::new("emit", "main", &self.ctx.id_gen);
        entry = entry.with_field("value", serde_json::Value::from(&v));
        entry.span = Some(crate::runtime::SpanInfo::from_source_span(span, &self.source));
        self.ctx.xadd(entry);
    }
    Ok(LxVal::Unit)
},
```

## Step 4: yield interception

File: `crates/lx/src/interpreter/mod.rs`

Current yield handling (lines 200-203):
```rust
Expr::Yield(ExprYield { value }) => {
    let v = self.eval(value).await?;
    Ok(self.ctx.yield_.yield_value(v, span)?)
},
```

Change to:
```rust
Expr::Yield(ExprYield { value }) => {
    let v = self.eval(value).await?;
    let prompt_id = self.ctx.call_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    {
        let mut entry = crate::runtime::StreamEntry::new("yield.out", "main", &self.ctx.id_gen);
        entry = entry.with_field("prompt_id", serde_json::Value::Number(prompt_id.into()));
        entry = entry.with_field("value", serde_json::Value::from(&v));
        entry.span = Some(crate::runtime::SpanInfo::from_source_span(span, &self.source));
        self.ctx.xadd(entry);
    }
    let response = self.ctx.yield_.yield_value(v, span)?;
    {
        let mut entry = crate::runtime::StreamEntry::new("yield.in", "main", &self.ctx.id_gen);
        entry = entry.with_field("prompt_id", serde_json::Value::Number(prompt_id.into()));
        entry = entry.with_field("response", serde_json::Value::from(&response));
        self.ctx.xadd(entry);
    }
    Ok(response)
},
```

## Step 5: tool.call / tool.result / tool.error interception with replay cache

File: `crates/lx/src/interpreter/tool_module.rs`

In the tool dispatch closure inside `build_tool_module` (created by Unit 3 in `tool_module.rs`), wrap the MCP call with auto-logging and replay cache checking. The closure's `_ctx` parameter must be renamed to `ctx` so it can be used.

The closure from Unit 3 looks approximately like:
```rust
Arc::new(move |args: Vec<LxVal>, span: SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
```

Change `_ctx` to `ctx` and replace the closure body:

```rust
Arc::new(move |args: Vec<LxVal>, span: SourceSpan, ctx: Arc<crate::runtime::RuntimeCtx>| {
    let client_ref = Arc::clone(&client_ref);
    let method_name = method_name.clone();
    let tool_module_name = tool_module_name.clone();
    Box::pin(async move {
        if !client_ref.is_alive().await {
            return Ok(LxVal::err_str(format!(
                "tool '{}' process exited unexpectedly",
                tool_module_name
            )));
        }

        let call_id = ctx.call_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        {
            let cache = ctx.replay_cache.lock();
            if let Some(cached) = cache.get(&call_id) {
                return Ok(cached.clone());
            }
        }

        let arg_json = serde_json::Value::from(&args[0]);

        {
            let mut entry = crate::runtime::StreamEntry::new("tool.call", "main", &ctx.id_gen);
            entry = entry.with_field("call_id", serde_json::Value::Number(call_id.into()));
            entry = entry.with_field("tool", serde_json::Value::String(tool_module_name.clone()));
            entry = entry.with_field("method", serde_json::Value::String(method_name.clone()));
            entry = entry.with_field("args", arg_json.clone());
            ctx.xadd(entry);
        }

        match client_ref.call_tool(&method_name, arg_json).await {
            Ok(result) => {
                let val = LxVal::from(result.clone());
                {
                    let mut entry = crate::runtime::StreamEntry::new("tool.result", "main", &ctx.id_gen);
                    entry = entry.with_field("call_id", serde_json::Value::Number(call_id.into()));
                    entry = entry.with_field("tool", serde_json::Value::String(tool_module_name.clone()));
                    entry = entry.with_field("method", serde_json::Value::String(method_name.clone()));
                    entry = entry.with_field("result", result);
                    ctx.xadd(entry);
                }
                Ok(val)
            },
            Err(e) => {
                {
                    let mut entry = crate::runtime::StreamEntry::new("tool.error", "main", &ctx.id_gen);
                    entry = entry.with_field("call_id", serde_json::Value::Number(call_id.into()));
                    entry = entry.with_field("tool", serde_json::Value::String(tool_module_name.clone()));
                    entry = entry.with_field("method", serde_json::Value::String(method_name.clone()));
                    entry = entry.with_field("error", serde_json::Value::String(e.clone()));
                    ctx.xadd(entry);
                }
                Ok(LxVal::err_str(format!(
                    "tool '{}' method '{}': {e}",
                    tool_module_name, method_name
                )))
            },
        }
    })
})
```

MCP notification logging is excluded from this unit's scope. The `NotificationEvent` messages collect in the channel but are not processed.

## Step 6: Check file lengths

After modifications:
- `mod.rs` gains ~40 lines for program.start/done, emit, yield interception. Current is 219 lines, new is ~260. Under 300.
- `tool_module.rs` gains ~30 lines for tool.call/tool.result/tool.error wrapping. Unit 3 created this file at ~65 lines, so with auto-logging it reaches ~95 lines. Under 300.
- `modules.rs` is unchanged by this unit (tool dispatch lives in `tool_module.rs` since Unit 3). Under 300.

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. All existing tests pass unchanged
3. Write a test `.lx` file:
```lx
use stream {backend: "jsonl", path: "/tmp/lx_test_autolog.jsonl"}
emit "hello"
entries = stream.xrange "-" "+"
assert (entries | length) >= 2
kinds = entries | map (.kind)
assert (kinds | contains "program.start")
assert (kinds | contains "emit")
```
