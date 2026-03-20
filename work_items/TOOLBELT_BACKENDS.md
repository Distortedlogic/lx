# Goal

Create mcp-toolbelt-specific backend implementations that the desktop app wires into RuntimeCtx when running lx programs. These are NOT defaults — they are the implementations the desktop app's `build_runtime_ctx` selects.

Four backends:
- `TeiEmbedBackend` — calls local TEI embedding server (Qwen3-Embedding-0.6B at port 8096)
- `DesktopPaneBackend` — HTTP POST to desktop app's pane management API
- `LocalLlmAiBackend` — already created in AI_BACKEND_LOCAL_LLM work item, wired here
- DX event-emitting wrappers for all new backend traits

# Why

- The desktop app (`backends/dx/`) already constructs a custom `RuntimeCtx` via `build_runtime_ctx`. It wraps each backend to emit `RuntimeEvent` to the `EventBus`. New backend traits (Embed, Pane, Transcribe, Speech, ImageGen) need the same treatment.
- `TeiEmbedBackend` calls the local embedding server directly (not Voyage AI) because the desktop app runs on machines with local GPU inference infrastructure.
- `DesktopPaneBackend` calls the desktop app's HTTP API to create actual UI panes (terminal, browser, editor, canvas) instead of yielding JSON-line messages.

# What Changes

**`backends/dx/src/backends/embed.rs` — TeiEmbedBackend + DX wrapper:**

`TeiEmbedBackend` calls `POST http://localhost:8096/infer` with `{"inputs": texts, "normalize": true, "truncate": true}`. Response is a raw `Vec<Vec<f32>>` JSON array (no wrapper). This is the mcp-toolbelt embedding server protocol, NOT the Voyage/OpenAI format.

`DxEmbedBackend` wraps any `EmbedBackend`, emits `EmbedCall`/`EmbedResult` events to EventBus, delegates to inner.

**`backends/dx/src/backends/pane.rs` — DesktopPaneBackend + DX wrapper:**

`DesktopPaneBackend` POSTs to the desktop app's pane API:
- `open` → `POST {api_base}/api/terminal-requests` with `{kind, config}`, gets back `{pane_id}`
- `update` → `POST {api_base}/api/pane/update` with `{pane_id, content}`
- `close` → `POST {api_base}/api/pane/close` with `{pane_id}`
- `list` → `GET {api_base}/api/pane/list`

`DxPaneBackend` wraps any `PaneBackend`, emits `PaneOpened`/`PaneClosed` events to EventBus.

**`backends/dx/src/backends/media.rs` — DX wrappers for media backends:**

`DxTranscribeBackend`, `DxSpeechBackend`, `DxImageGenBackend` — each wraps inner backend and emits events to EventBus. The actual media backends (WhisperBackend, KokoroBackend, FluxBackend) are already the default implementations from the lx crate — the DX crate just wraps them for event emission.

**`backends/dx/src/backends/mod.rs` — wire into build_runtime_ctx:**

Update `build_runtime_ctx` to construct RuntimeCtx with:
- `embed: DxEmbedBackend(TeiEmbedBackend::new(env_or_default("EMBEDDING_URL", "http://localhost:8096")))`
- `pane: DxPaneBackend(DesktopPaneBackend::new(env_or_default("LX_PANE_API", "http://localhost:5173")))`
- `transcribe: DxTranscribeBackend(WhisperBackend::new(env_or_default("WHISPER_URL", "http://localhost:8095")))`
- `speech: DxSpeechBackend(KokoroBackend::new(env_or_default("KOKORO_URL", "http://localhost:8094")))`
- `image_gen: DxImageGenBackend(FluxBackend::new(env_or_default("FLUX_URL", "http://localhost:8091")))`

# Files Affected

- `backends/dx/src/backends/embed.rs` — New file: TeiEmbedBackend + DxEmbedBackend
- `backends/dx/src/backends/pane.rs` — New file: DesktopPaneBackend + DxPaneBackend
- `backends/dx/src/backends/media.rs` — New file: DxTranscribeBackend + DxSpeechBackend + DxImageGenBackend
- `backends/dx/src/backends/mod.rs` — Wire into build_runtime_ctx
- `backends/dx/src/event.rs` — Add new RuntimeEvent variants
- `backends/dx/src/adapters/ansi.rs` — Add formatters for new events

# Task List

### Task 1: Add RuntimeEvent variants for new backends

**Subject:** Add Pane, Embed, and Media event variants to RuntimeEvent

**Description:** Edit `backends/dx/src/event.rs`.

Add variants:

```rust
PaneOpened { agent_id: String, pane_id: String, kind: String, ts: Instant },
PaneClosed { agent_id: String, pane_id: String, ts: Instant },
EmbedCall { agent_id: String, text_count: usize, ts: Instant },
EmbedResult { agent_id: String, vector_count: usize, dimensions: usize, duration_ms: u64, ts: Instant },
TranscribeCall { agent_id: String, ts: Instant },
TranscribeResult { agent_id: String, text_len: usize, duration_ms: u64, ts: Instant },
SpeechCall { agent_id: String, text_len: usize, ts: Instant },
SpeechResult { agent_id: String, format: String, duration_ms: u64, ts: Instant },
ImageGenCall { agent_id: String, prompt_len: usize, ts: Instant },
ImageGenResult { agent_id: String, format: String, duration_ms: u64, ts: Instant },
```

Update `agent_id()` match arms to include new variants.

**ActiveForm:** Adding RuntimeEvent variants

---

### Task 2: Create TeiEmbedBackend and DxEmbedBackend

**Subject:** Create embed.rs with TEI embedding backend and DX event wrapper

**Description:** Create `backends/dx/src/backends/embed.rs`.

`TeiEmbedBackend` — takes a URL string. Implements `EmbedBackend`:
- POST to `{url}/infer` with `{"inputs": texts, "normalize": true, "truncate": true}`
- Response is raw `Vec<Vec<f32>>` (the TEI server returns a plain JSON array, not wrapped in `data` field)
- Convert to `Value::List` of `Value::List` of `Value::Float`

`DxEmbedBackend` — wraps `Arc<dyn EmbedBackend>` + `Arc<EventBus>` + `agent_id: String`. Implements `EmbedBackend`:
- Emit `EmbedCall` before delegating
- Delegate to inner
- Emit `EmbedResult` after (with duration)

Add `pub mod embed;` to `backends/dx/src/backends/mod.rs`.

**ActiveForm:** Creating TEI and DX embed backends

---

### Task 3: Create DesktopPaneBackend and DxPaneBackend

**Subject:** Create pane.rs with desktop HTTP pane backend and DX event wrapper

**Description:** Create `backends/dx/src/backends/pane.rs`.

`DesktopPaneBackend` — takes a URL string (desktop app API base). Implements `PaneBackend`:
- `open(kind, config)` → POST `{url}/api/terminal-requests` with `{"kind": kind, "config": config_json}`. Parse response for `pane_id`. Return handle Record.
- `update(pane_id, content)` → POST `{url}/api/pane/update`
- `close(pane_id)` → POST `{url}/api/pane/close`
- `list()` → GET `{url}/api/pane/list`

`DxPaneBackend` — wraps `Arc<dyn PaneBackend>` + `Arc<EventBus>` + `agent_id: String`. Emits `PaneOpened`/`PaneClosed` events.

**ActiveForm:** Creating desktop and DX pane backends

---

### Task 4: Create DX media wrappers and wire build_runtime_ctx

**Subject:** Create media.rs with DX wrappers and update build_runtime_ctx

**Description:** Create `backends/dx/src/backends/media.rs`.

Three DX wrappers — `DxTranscribeBackend`, `DxSpeechBackend`, `DxImageGenBackend`. Each wraps inner + EventBus + agent_id, emits call/result events around delegation. Same pattern as DxEmbedBackend.

Edit `backends/dx/src/backends/mod.rs`:

Update `build_runtime_ctx` (or wherever the DX RuntimeCtx is constructed) to wire all new backends:

```rust
fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

// In build_runtime_ctx:
embed: Arc::new(DxEmbedBackend {
    inner: Arc::new(TeiEmbedBackend::new(env_or_default("EMBEDDING_URL", "http://localhost:8096"))),
    bus: bus.clone(),
    agent_id: agent_id.clone(),
}),
pane: Arc::new(DxPaneBackend {
    inner: Arc::new(DesktopPaneBackend::new(env_or_default("LX_PANE_API", "http://localhost:5173"))),
    bus: bus.clone(),
    agent_id: agent_id.clone(),
}),
transcribe: Arc::new(DxTranscribeBackend {
    inner: Arc::new(WhisperBackend::new(env_or_default("WHISPER_URL", "http://localhost:8095"))),
    bus: bus.clone(),
    agent_id: agent_id.clone(),
}),
speech: Arc::new(DxSpeechBackend {
    inner: Arc::new(KokoroBackend::new(env_or_default("KOKORO_URL", "http://localhost:8094"))),
    bus: bus.clone(),
    agent_id: agent_id.clone(),
}),
image_gen: Arc::new(DxImageGenBackend {
    inner: Arc::new(FluxBackend::new(env_or_default("FLUX_URL", "http://localhost:8091"))),
    bus: bus.clone(),
    agent_id: agent_id.clone(),
}),
```

Add `pub mod media;` to mod.rs.

**ActiveForm:** Creating DX media wrappers and wiring build_runtime_ctx

---

### Task 5: Update ANSI formatter

**Subject:** Add format functions for new events to ansi.rs

**Description:** Edit `backends/dx/src/adapters/ansi.rs`.

Add format functions for all new event types:
- `format_pane_opened(pane_id, kind)` → green `[PANE] opened {kind} ({pane_id})`
- `format_pane_closed(pane_id)` → dim `[PANE] closed ({pane_id})`
- `format_embed_call(text_count)` → blue `[EMBED] {text_count} texts`
- `format_embed_result(vector_count, dimensions, duration_ms)` → dim `[EMBED] {vector_count}x{dimensions}d in {duration_ms}ms`
- `format_transcribe_call()` → blue `[ASR] transcribing...`
- `format_transcribe_result(text_len, duration_ms)` → dim `[ASR] {text_len} chars in {duration_ms}ms`
- `format_speech_call(text_len)` → blue `[TTS] speaking {text_len} chars`
- `format_speech_result(format, duration_ms)` → dim `[TTS] {format} in {duration_ms}ms`
- `format_image_gen_call(prompt_len)` → blue `[IMG] generating from {prompt_len} char prompt`
- `format_image_gen_result(format, duration_ms)` → dim `[IMG] {format} in {duration_ms}ms`

Update `format_event` dispatch to handle new variants.

**ActiveForm:** Updating ANSI formatter for new events

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
mcp__workflow__load_work_item({ path: "work_items/TOOLBELT_BACKENDS.md" })
```

Then call `next_task` to begin.
