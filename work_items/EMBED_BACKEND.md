# Goal

Add `EmbedBackend` trait to RuntimeCtx and extend `std/ai` with embedding functions. Default backend: `VoyageEmbedBackend` — calls Voyage AI's embedding API. Voyage is Anthropic's recommended embedding provider and the most portable choice for anyone using Claude-based tooling.

# Why

- Embeddings are the backbone of semantic search, RAG, and codebase indexing. lx has `std/ai` for LLM chat but no way to compute text embeddings.
- Embedding providers are distinct from chat providers — Claude doesn't offer embeddings. The backend must be separate from `AiBackend`.
- Voyage AI is the default because it's what Anthropic recommends, has the best integration story with Claude-based workflows, and requires only a single env var (`VOYAGE_API_KEY`) to work.
- The mcp-toolbelt desktop app swaps in `TeiEmbedBackend` (local Qwen3-Embedding at port 8096). That's a different backend, not a fallback.

# What Changes

**`crates/lx/src/backends/mod.rs` — new EmbedBackend trait:**

```rust
#[derive(Debug, Clone, Default)]
pub struct EmbedOpts {
    pub model: Option<String>,
    pub dimensions: Option<usize>,
}

pub trait EmbedBackend: Send + Sync {
    fn embed(&self, texts: &[String], opts: &EmbedOpts, span: Span) -> Result<Value, LxError>;
}
```

Add `pub embed: Arc<dyn EmbedBackend>` field to `RuntimeCtx`. Default: `VoyageEmbedBackend`.

**`crates/lx/src/backends/embed.rs` — VoyageEmbedBackend:**

Calls `POST https://api.voyageai.com/v1/embeddings` with `Authorization: Bearer {VOYAGE_API_KEY}`, body `{"input": texts, "model": "voyage-3-lite"}`. Returns `Vec<Vec<f64>>` as nested lx Lists. If `VOYAGE_API_KEY` is not set, returns `Err "VOYAGE_API_KEY not set"`.

**`crates/lx/src/stdlib/ai.rs` — extend with embed functions:**

Add `ai.embed` and `ai.embed_with` to the existing `build()` function.

# Files Affected

- `crates/lx/src/backends/mod.rs` — Add EmbedBackend trait, EmbedOpts, add field to RuntimeCtx
- `crates/lx/src/backends/embed.rs` — New file: VoyageEmbedBackend
- `crates/lx/src/stdlib/ai.rs` — Add embed/embed_with entries to build()
- `tests/101_embed.lx` — New test file

# Task List

### Task 1: Add EmbedBackend trait to backends

**Subject:** Add EmbedBackend trait, EmbedOpts struct, and embed field to RuntimeCtx

**Description:** Edit `crates/lx/src/backends/mod.rs`:

Add after `HttpOpts`:

```rust
#[derive(Debug, Clone, Default)]
pub struct EmbedOpts {
    pub model: Option<String>,
    pub dimensions: Option<usize>,
}
```

Add after the last trait definition:

```rust
pub trait EmbedBackend: Send + Sync {
    fn embed(&self, texts: &[String], opts: &EmbedOpts, span: Span) -> Result<Value, LxError>;
}
```

Add `mod embed;` and `pub use embed::*;` at the top.

Add `pub embed: Arc<dyn EmbedBackend>,` to `RuntimeCtx`.

Add `embed: Arc::new(VoyageEmbedBackend),` to `Default` impl.

**ActiveForm:** Adding EmbedBackend trait to RuntimeCtx

---

### Task 2: Implement VoyageEmbedBackend

**Subject:** Create embed.rs with VoyageEmbedBackend calling Voyage AI API

**Description:** Create `crates/lx/src/backends/embed.rs`.

Imports: `std::sync::Arc`, `reqwest::Client`, `serde_json::json`, `indexmap::IndexMap`, `crate::error::LxError`, `crate::span::Span`, `crate::value::Value`, `super::{EmbedBackend, EmbedOpts}`.

`pub struct VoyageEmbedBackend;`

Implement `EmbedBackend for VoyageEmbedBackend`:

`fn embed(&self, texts: &[String], opts: &EmbedOpts, span: Span) -> Result<Value, LxError>`:

Read `VOYAGE_API_KEY` from env. If not set, return `Ok(Value::Err(Box::new(Value::Str(Arc::from("VOYAGE_API_KEY not set — get one at https://dash.voyageai.com/")))))`.

Use `tokio::task::block_in_place` + `Handle::current().block_on` (same pattern as `ReqwestHttpBackend`).

Inside the async block:
- Model: `opts.model.as_deref().unwrap_or("voyage-3-lite")`
- POST to `https://api.voyageai.com/v1/embeddings`
- Headers: `Authorization: Bearer {api_key}`, `Content-Type: application/json`
- Body: `{"input": texts, "model": model}`
- If `opts.dimensions` is Some, add `"output_dimension": dim` to body
- Parse response JSON: `data` is an array of objects with `embedding` field (array of floats)
- Convert each embedding to `Value::List(Arc::new(floats.iter().map(|f| Value::Float(*f)).collect()))`
- Collect into `Value::List` of vectors
- Return `Ok(Value::Ok(Box::new(Value::List(Arc::new(vectors)))))`
- On HTTP error, return `Ok(Value::Err(...))`

**ActiveForm:** Implementing VoyageEmbedBackend

---

### Task 3: Extend std/ai with embed functions and write tests

**Subject:** Add ai.embed and ai.embed_with to std/ai and write tests

**Description:** Edit `crates/lx/src/stdlib/ai.rs`:

In `build()`, add:
- `"embed"` → `bi_embed` arity 1
- `"embed_with"` → `bi_embed_with` arity 1

`fn bi_embed(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is a List of Str. Extract strings. Call `ctx.embed.embed(&texts, &EmbedOpts::default(), span)`.

`fn bi_embed_with(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is a Record with `texts` (List of Str), optional `model` (Str), optional `dimensions` (Int). Build `EmbedOpts`. Call `ctx.embed.embed(&texts, &opts, span)`.

Import `EmbedOpts` from `crate::backends`.

Create `tests/101_embed.lx`:

```
use std/ai

result = ai.embed ["hello world" "test"]
result ? {
  Ok vectors -> {
    assert (vectors | len == 2) "two vectors returned"
    assert (vectors.[0] | len > 0) "vector has dimensions"
    log.info "101_embed: embed test passed"
  }
  Err e -> {
    assert (e | to_str | len > 0) "error message is descriptive"
    log.info "101_embed: skipped ({e})"
  }
}

log.info "101_embed: all passed"
```

Run `just diagnose` to verify compilation.

**ActiveForm:** Adding embed functions to std/ai and writing tests

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
mcp__workflow__load_work_item({ path: "work_items/EMBED_BACKEND.md" })
```

Then call `next_task` to begin.
