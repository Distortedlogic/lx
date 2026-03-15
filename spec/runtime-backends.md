# Runtime Backends

How lx decouples language semantics from host-specific implementations. Every I/O-touching operation goes through a `RuntimeCtx` that the embedder provides.

## Problem

Currently, stdlib modules hardcode their implementations directly in the builtin functions:
- `std/ai` calls `Command::new("claude")` inline
- `std/http` constructs `reqwest::blocking::Client` inline
- `yield` uses a separate callback field on the interpreter
- `emit` is unimplemented
- `$shell` uses `std::process::Command` inline

The implementations themselves are mostly correct — Claude Code CLI is the right AI backend, reqwest is the right HTTP backend. The problem is that they're not behind an abstraction boundary, so an embedder cannot swap them for testing, server deployment, or sandboxing.

## Design: `RuntimeCtx`

A single struct passed as a parameter to every builtin function. The embedder constructs it before creating the interpreter.

```rust
pub struct RuntimeCtx {
    pub ai: Arc<dyn AiBackend>,
    pub emit: Arc<dyn EmitBackend>,
    pub http: Arc<dyn HttpBackend>,
    pub shell: Arc<dyn ShellBackend>,
    pub yield_: Arc<dyn YieldBackend>,
    pub log: Arc<dyn LogBackend>,
}
```

### Backend Traits

Each trait is a focused interface for one capability:

```rust
pub trait AiBackend: Send + Sync {
    fn prompt(&self, text: &str, opts: &AiOpts, span: Span) -> Result<Value, LxError>;
}

pub trait EmitBackend: Send + Sync {
    fn emit(&self, value: Value, span: Span) -> Result<(), LxError>;
}

pub trait HttpBackend: Send + Sync {
    fn request(&self, method: &str, url: &str, opts: HttpOpts, span: Span) -> Result<Value, LxError>;
}

pub trait ShellBackend: Send + Sync {
    fn exec(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
    fn exec_capture(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
}

pub trait YieldBackend: Send + Sync {
    fn yield_value(&self, value: Value, span: Span) -> Result<Value, LxError>;
}

pub trait LogBackend: Send + Sync {
    fn log(&self, level: LogLevel, msg: &str);
}
```

### Builtin Signature Change

Current:
```rust
fn bi_prompt(args: &[Value], span: Span) -> Result<Value, LxError>
```

New:
```rust
fn bi_prompt(args: &[Value], span: Span, ctx: &RuntimeCtx) -> Result<Value, LxError>
```

The `mk` helper changes accordingly:
```rust
pub fn mk(name: &str, arity: usize, f: fn(&[Value], Span, &RuntimeCtx) -> Result<Value, LxError>) -> Value
```

Builtins that don't need backends ignore `ctx` with `_ctx`.

### Interpreter Threading

The interpreter holds `RuntimeCtx` and passes it through to `eval` → `apply_func` → builtin call:

```rust
pub struct Interpreter {
    // ...existing fields...
    pub(crate) ctx: Arc<RuntimeCtx>,
}
```

`yield` and `emit` AST nodes also go through the context instead of separate handler fields.

### Standard Default Backends

These are the production defaults — not placeholders. `RuntimeCtx::default()` returns all of them.

| Backend | Implementation | Why it's the right default |
|---------|---------------|---------------------------|
| `ClaudeCodeAiBackend` | Claude Code CLI (`claude -p --output-format json`) | Handles auth, model routing, tool permissions, session management, cost tracking. The CLI is the standard programmatic interface to Claude. |
| `StdoutEmitBackend` | `println!` for strings, `serde_json::to_string` for structured values | Direct, zero-dependency. Correct for CLI tools and scripts. |
| `ReqwestHttpBackend` | `reqwest::blocking::Client` | Battle-tested HTTP client. Handles TLS, redirects, timeouts. |
| `ProcessShellBackend` | `std::process::Command` via `/bin/sh -c` | Standard POSIX shell execution. Same as every scripting language. |
| `StdinStdoutYieldBackend` | JSON-line protocol on stdin/stdout | Universal IPC. Any orchestrator (Python, Node, another lx process) can drive it. |
| `StderrLogBackend` | `eprintln!` with level prefix | Keeps logs out of stdout data stream. Standard Unix convention. |

```rust
let ctx = RuntimeCtx::default();
let mut interp = Interpreter::new(source, source_dir, ctx);
```

### Claude Code AI Backend Details

The `ClaudeCodeAiBackend` invokes `claude -p --output-format json` as a subprocess. This is the standard interface — not a stopgap for a "real" API client. Claude Code CLI handles:

- API authentication (reads `~/.claude` config)
- Model selection (respects `--model` or defaults)
- Tool permissions and MCP server management
- Session continuity (`--resume` with session IDs)
- Cost tracking (returns `cost_usd` in response)
- Multi-turn conversation management (`--max-turns`)
- System prompt injection (`--system-prompt`, `--append-system-prompt`)

The backend maps `AiOpts` fields to CLI flags:

| `AiOpts` field | CLI flag |
|----------------|----------|
| `system` | `--system-prompt` |
| `model` | `--model` |
| `max_turns` | `--max-turns` |
| `resume` | `--resume` |
| `tools` | `--allowedTools` |
| `append_system` | `--append-system-prompt` |

Response JSON is parsed into lx `Value::Record` with fields: `text`, `session_id`, `cost`, `turns`, `duration_ms`, `model`.

### Alternate Backends (Examples)

For when the defaults don't fit:

```rust
// Server deployment — different emit/yield transport, restricted shell
let ctx = RuntimeCtx {
    ai: Arc::new(ClaudeCodeAiBackend),         // still use Claude Code CLI
    emit: Arc::new(WebSocketEmitBackend::new(conn)),
    http: Arc::new(ReqwestHttpBackend),         // still use reqwest
    shell: Arc::new(SandboxShellBackend),       // restricted
    yield_: Arc::new(HttpYieldBackend::new(callback_url)),
    log: Arc::new(StructuredLogBackend::new(sink)),
};

// Testing — mock AI and collect emits
let ctx = RuntimeCtx {
    ai: Arc::new(MockAiBackend::new(responses)),
    emit: Arc::new(CollectEmitBackend::new(buffer)),
    ..RuntimeCtx::default()
};

// Direct API (bypass CLI, for embedded use cases)
let ctx = RuntimeCtx {
    ai: Arc::new(AnthropicApiBackend::new(api_key)),
    ..RuntimeCtx::default()
};
```

## Scope

The following operations route through `RuntimeCtx`:

| Operation | Backend trait | Standard default |
|-----------|-------------|-----------------|
| `ai.prompt`, `ai.prompt_with` | `AiBackend` | `ClaudeCodeAiBackend` (Claude Code CLI) |
| `emit expr` | `EmitBackend` | `StdoutEmitBackend` (println / JSON) |
| `http.get/post/put/delete` | `HttpBackend` | `ReqwestHttpBackend` |
| `$cmd`, `$^cmd` | `ShellBackend` | `ProcessShellBackend` |
| `yield expr` | `YieldBackend` | `StdinStdoutYieldBackend` |
| `log.info/warn/err/debug` | `LogBackend` | `StderrLogBackend` |

Operations that stay hardcoded (no meaningful alternative backend):
- `std/fs` — always local filesystem
- `std/env` — always process environment
- `std/json` — pure data transformation
- `std/math`, `std/re`, `std/time` — pure computation
- `std/ctx` — file-backed, always local
- Agent send/ask (`~>`, `~>?`) — subprocess protocol, always local

## Migration

The refactor is mechanical:

1. Define `RuntimeCtx` and backend traits in `crates/lx/src/backends.rs`
2. Add default implementations in `crates/lx/src/backends/defaults.rs`
3. Change `BuiltinFn` signature to include `&RuntimeCtx`
4. Update `mk()` in `builtins/mod.rs`
5. Thread `ctx` through `Interpreter::eval` → `apply_func` → builtin dispatch
6. Update each stdlib module to use `ctx.ai`, `ctx.http`, etc. instead of direct implementations
7. Remove `yield_handler` field (subsumed by `ctx.yield_`)
8. Implement `emit` AST node evaluation using `ctx.emit`

Most builtins gain `_ctx: &RuntimeCtx` and are otherwise unchanged. Only `ai.rs`, `http.rs`, `shell.rs`, and the yield/emit eval paths change substantively.

## Cross-References

- Runtime semantics: [runtime.md](runtime.md)
- Emit/yield semantics: [agents-advanced.md](agents-advanced.md)
- Shell integration: [shell.md](shell.md)
- Stdlib modules: [stdlib.md](stdlib.md)
