# Goal

Add `LocalLlmAiBackend` — an `AiBackend` implementation that calls a local LLM server over HTTP instead of shelling out to the `claude` CLI. Uses the mcp-toolbelt inference server protocol: `POST /infer` with `{messages, max_tokens, temperature, top_p}`, gets back `{text, usage}`. The mcp-toolbelt desktop app sets this as its `AiBackend` when running lx programs.

This is NOT a replacement for `ClaudeCodeAiBackend`. It is a separate backend implementation. The embedder (CLI, desktop app) picks which one to use. The default remains `ClaudeCodeAiBackend`.

# Why

- lx's only `AiBackend` implementation shells out to `claude` CLI. Every `ai.prompt` call requires an Anthropic API key and internet access.
- mcp-toolbelt runs a local Qwen3.5-27B server at port 8097/8098 with the shared inference protocol (`/health` + `/infer`). The request/response maps cleanly to `AiBackend::prompt`.
- The desktop app needs a backend it can wire in when launching lx programs against local models. This is that backend.

# What Changes

**New file `crates/lx/src/backends/ai_local.rs` — LocalLlmAiBackend:**

Takes a base URL (e.g., `http://localhost:8098`). Converts `AiBackend::prompt(text, opts)` to the inference server's chat format: `{messages: [{role: "system", content: system_prompt}, {role: "user", content: text}], max_tokens: 4096, temperature: 0.7}`. Parses `{text, usage}` response into the same `Value::Ok({text, model, duration_ms})` shape that `ClaudeCodeAiBackend` returns.

# Files Affected

- `crates/lx/src/backends/ai_local.rs` — New file: LocalLlmAiBackend
- `crates/lx/src/backends/mod.rs` — Add `mod ai_local; pub use ai_local::*;`
- `tests/109_ai_local.lx` — New test file

# Task List

### Task 1: Create LocalLlmAiBackend

**Subject:** Create ai_local.rs implementing AiBackend for local inference servers

**Description:** Create `crates/lx/src/backends/ai_local.rs`.

Imports: `std::sync::Arc`, `std::time::Instant`, `reqwest::Client`, `serde_json::json`, `indexmap::IndexMap`, `num_bigint::BigInt`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`, `super::{AiBackend, AiOpts}`.

```rust
pub struct LocalLlmAiBackend {
    pub url: String,
}

impl LocalLlmAiBackend {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
```

Implement `AiBackend for LocalLlmAiBackend`:

`fn prompt(&self, text: &str, opts: &AiOpts, span: Span) -> Result<Value, LxError>`:

Build messages array:
- If `opts.system` is Some, add `{"role": "system", "content": system}`
- If `opts.append_system` is Some, add as another system message
- Add `{"role": "user", "content": text}`

Build request body:
```json
{
  "messages": messages,
  "max_tokens": 4096,
  "temperature": 0.7,
  "top_p": 0.9
}
```

Use `tokio::task::block_in_place` + `Handle::current().block_on`:
- Record start time via `Instant::now()`
- POST to `{self.url}/infer` with JSON body
- On HTTP error, return `Ok(Value::Err(Box::new(Value::Str(Arc::from(format!("local LLM: {e}"))))))`
- Parse response JSON: `{text: String, usage: {prompt_tokens, completion_tokens}}`
- Compute `duration_ms` from start time
- Return same shape as `ClaudeCodeAiBackend`:
```rust
Ok(Value::Ok(Box::new(record! {
    "text" => Value::Str(Arc::from(response_text)),
    "model" => Value::Str(Arc::from("local")),
    "duration_ms" => Value::Int(BigInt::from(duration_ms)),
})))
```

Add `mod ai_local;` and `pub use ai_local::*;` to `crates/lx/src/backends/mod.rs`.

**ActiveForm:** Creating LocalLlmAiBackend for local inference servers

---

### Task 2: Write tests

**Subject:** Write test verifying LocalLlmAiBackend compiles and API shape is correct

**Description:** Create `tests/109_ai_local.lx`:

```
use std/ai

-- ai.prompt uses whatever AiBackend is configured
-- When running under mcp-toolbelt with LocalLlmAiBackend, this calls local Qwen
-- When running standalone, this calls claude CLI via ClaudeCodeAiBackend
-- Either way, the response shape is the same

result = ai.prompt "Say hello in exactly one word"
result ? {
  Ok r -> {
    assert (r.text | len > 0) "got response text"
    log.info "109_ai_local: got response: {r.text}"
  }
  Err e -> {
    log.info "109_ai_local: skipped ({e})"
  }
}

log.info "109_ai_local: all passed"
```

Run `just diagnose` to verify compilation.

**ActiveForm:** Writing tests for LocalLlmAiBackend

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AI_BACKEND_LOCAL_LLM.md" })
```

Then call `next_task` to begin.
