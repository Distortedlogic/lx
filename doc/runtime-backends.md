# Runtime Backends — Reference

## `RuntimeCtx`

Single struct passed to every builtin function. Embedder constructs it before creating the interpreter.

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

## Backend Traits

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

## Builtin Signature

```rust
fn bi_prompt(args: &[Value], span: Span, ctx: &RuntimeCtx) -> Result<Value, LxError>
```

Builtins that don't need backends ignore `ctx` with `_ctx`.

## Default Backends

`RuntimeCtx::default()` returns all of these:

| Backend | Implementation | What it does |
|---------|---------------|-------------|
| `ClaudeCodeAiBackend` | Claude Code CLI (`claude -p --output-format json`) | Auth, model routing, tools, sessions, cost tracking |
| `StdoutEmitBackend` | `println!` / `serde_json::to_string` | Direct stdout for CLI tools |
| `ReqwestHttpBackend` | `reqwest::blocking::Client` | TLS, redirects, timeouts |
| `ProcessShellBackend` | `std::process::Command` via `/bin/sh -c` | Standard POSIX shell |
| `StdinStdoutYieldBackend` | JSON-line protocol on stdin/stdout | Universal IPC |
| `StderrLogBackend` | `eprintln!` with level prefix | Logs to stderr |

## AiOpts to CLI Flag Mapping

| `AiOpts` field | CLI flag |
|----------------|----------|
| `system` | `--system-prompt` |
| `model` | `--model` |
| `max_turns` | `--max-turns` |
| `resume` | `--resume` |
| `tools` | `--allowedTools` |
| `append_system` | `--append-system-prompt` |

Response fields: `text`, `session_id`, `cost`, `turns`, `duration_ms`, `model`.

## Scope: What Routes Through `RuntimeCtx`

| Operation | Backend trait |
|-----------|-------------|
| `ai.prompt`, `ai.prompt_with` | `AiBackend` |
| `emit expr` | `EmitBackend` |
| `http.get/post/put/delete` | `HttpBackend` |
| `$cmd`, `$^cmd` | `ShellBackend` |
| `yield expr` | `YieldBackend` |
| `log.info/warn/err/debug` | `LogBackend` |

Hardcoded (no backend): `std/fs`, `std/env`, `std/json`, `std/math`, `std/re`, `std/time`, `std/ctx`, agent send/ask (`~>`, `~>?`).
