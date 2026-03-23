# Voice Pipeline Integration Plan

**Status: COMPLETED**

## Prerequisites

Read `/home/entropybender/repos/lx/CLAUDE.md` first and follow all rules in it. Key rules that apply here:
- No code comments or doc strings
- No `#[allow(...)]` macros
- No `.unwrap()` — use `?` or `.expect("reason")`
- Do not swallow errors — propagate or surface them
- 300 line file limit
- Verify with `cd crates/lx-desktop && cargo clippy --features desktop -- -D warnings`

## Goal

Wire the voice chat pipeline into the lx-desktop Dioxus app. A human opens a Voice pane, speaks into their microphone, speech is transcribed (Whisper STT), sent to Claude CLI for an LLM response, and the response is spoken back (Kokoro TTS).

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ lx-desktop (Dioxus desktop app)                             │
│                                                             │
│  ┌──────────────┐    widget-bridge     ┌──────────────────┐ │
│  │ VoiceView    │◄──────────────────►  │ TS voice widget  │ │
│  │ (Rust comp)  │   dx.send/recv       │ (audio capture + │ │
│  │              │                      │  playback + UI)  │ │
│  └──────┬───────┘                      └──────────────────┘ │
│         │                                                   │
│         ├── audio_core::wrap_pcm_as_wav()                   │
│         ├── audio_core::chunk_wav()                         │
│         ├── whisper_client::WHISPER.infer()   → STT         │
│         ├── ClaudeCliBackend.query()          → LLM         │
│         └── kokoro_client::KOKORO.infer()     → TTS         │
└─────────────────────────────────────────────────────────────┘
```

The TS voice widget handles browser-side audio capture, VAD, and playback. It communicates with the Rust side via `widget.send_update()` / `widget.recv()` — no WebSocket server. The Rust `VoiceView` component orchestrates the STT → LLM → TTS pipeline.

## Exact API Signatures of Existing Code

You will call these — do NOT modify any of these crates.

### `audio_core` (crate at `crates/audio-core/`)

```rust
pub fn wrap_pcm_as_wav(raw: &[u8], sample_rate: u32, channels: u16, bits_per_sample: u16) -> Vec<u8>
pub fn chunk_wav(wav: &[u8], max_chunk_bytes: usize) -> Vec<Vec<u8>>
pub const SAMPLE_RATE: u32 = 16000;
pub const CHANNELS: u16 = 1;
pub const BITS_PER_SAMPLE: u16 = 16;
```

### `whisper_client` (crate at `crates/whisper-client/`)

```rust
pub use inference_client::InferenceClient;  // trait with async fn infer(&self, req) -> Result<Resp>

pub struct TranscribeRequest {
    pub audio_data: String,       // base64-encoded WAV
    pub language: Option<String>,
}
pub struct TranscribeResponse {
    pub text: String,
    pub language: String,
}

pub static WHISPER: LazyLock<WhisperClient>;  // call WHISPER.infer(&req).await
```

### `kokoro_client` (crate at `crates/kokoro-client/`)

```rust
pub use inference_client::InferenceClient;  // trait with async fn infer(&self, req) -> Result<Vec<u8>>

pub struct SpeechRequest {
    pub text: String,
    pub voice: String,
    pub lang_code: String,
    pub speed: f32,
}

pub static KOKORO: LazyLock<KokoroClient>;  // call KOKORO.infer(&req).await -> Vec<u8> (WAV bytes)
```

### `voice_agent` (crate at `crates/voice-agent/`)

```rust
#[async_trait::async_trait]
pub trait AgentBackend: Send + Sync {
    async fn query(&self, text: &str) -> anyhow::Result<String>;
}
```

### `widget_bridge` (crate at `crates/widget-bridge/`)

```rust
pub fn use_ts_widget(widget: &str, config: impl Serialize) -> (String, TsWidgetHandle)

// TsWidgetHandle is Copy (wraps Signal internally)
impl TsWidgetHandle {
    pub fn send_update(&self, data: impl Serialize)
    pub fn send_resize(&self)
    pub async fn recv<T: DeserializeOwned>(&self) -> Result<T, EvalError>
}
```

`TsWidgetHandle` is `Copy` because it wraps a `Signal<Option<document::Eval>>`. Both `widget` and Dioxus `Signal`s can be used directly inside `use_future` closures without explicit cloning.

## Messages: TS Widget → Rust (via `widget.recv::<serde_json::Value>()`)

```json
{ "type": "audio_chunk", "data": "<base64 16kHz 16-bit mono PCM>", "seq": 0 }
{ "type": "start_standby" }
{ "type": "silence_detected" }
{ "type": "cancel" }
{ "type": "playback_complete" }
{ "type": "status_change", "status": "idle" }
```

Access fields via `msg["type"].as_str()` and `msg["data"].as_str()`. These return `Option<&str>`.

## Messages: Rust → TS Widget (via `widget.send_update(serde_json::json!({...}))`)

```json
{ "type": "transcript", "text": "what the user said" }
{ "type": "agent_response", "text": "claude's reply" }
{ "type": "audio_response", "data": "<base64 WAV chunk>" }
{ "type": "error", "message": "something went wrong" }
```

## Changes — Step by Step

### Step 1: Cargo.toml dependencies

The crate uses `dioxus = { version = "0.7", features = ["fullstack", "router"] }` with separate `desktop` and `server` feature flags. Tokio requires the `process` feature for `ClaudeCliBackend`.

Required dependencies for voice pipeline:
```toml
audio-core = { path = "../audio-core" }
whisper-client = { path = "../whisper-client" }
kokoro-client = { path = "../kokoro-client" }
voice-agent = { path = "../voice-agent" }
anyhow = "1"
async-trait = "0.1"
```

Tokio must include the `process` feature:
```toml
tokio = { workspace = true, features = ["process"] }
```

### Step 2: Create `crates/lx-desktop/src/voice_backend.rs`

Implements `AgentBackend` by shelling out to `claude` CLI:

```rust
use voice_agent::AgentBackend;

pub struct ClaudeCliBackend;

#[async_trait::async_trait]
impl AgentBackend for ClaudeCliBackend {
  async fn query(&self, text: &str) -> anyhow::Result<String> {
    let output = tokio::process::Command::new("claude").args(["-p", text, "--output-format", "text"]).output().await?;
    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      anyhow::bail!("claude cli failed: {stderr}");
    }
    let response = String::from_utf8(output.stdout)?;
    Ok(response.trim().to_owned())
  }
}
```

### Step 3: Module declarations

The crate uses a `lib.rs` + `main.rs` split for fullstack support. `voice_backend` is declared in `lib.rs` as `pub mod voice_backend;`. The `main.rs` uses `lx_desktop::` paths and has dual `#[cfg(feature = "server")]` / `#[cfg(not(feature = "server"))]` entry points.

### Step 4: VoiceView in `crates/lx-desktop/src/terminal/view.rs`

#### Step 4a: Add imports

Add after `use widget_bridge::use_ts_widget;`:

```rust
use kokoro_client::SpeechRequest;
use voice_agent::AgentBackend as _;
use whisper_client::InferenceClient as _;
use whisper_client::TranscribeRequest;
```

The `as _` imports bring traits into scope for `.infer()` and `.query()` calls without unused-import warnings.

#### Step 4b: VoiceView component and pipeline

Clippy requires `let...else` for the recv pattern and collapsible `if let` chains:

```rust
#[component]
pub fn VoiceView(voice_id: String) -> Element {
    let (element_id, widget) = use_ts_widget("voice", serde_json::json!({}));
    let mut pcm_buffer: Signal<Vec<u8>> = use_signal(Vec::new);

    let eid_rsx = element_id.clone();
    use_future(move || async move {
        loop {
            let Ok(msg) = widget.recv::<serde_json::Value>().await else { break };

            match msg["type"].as_str() {
                Some("audio_chunk") => {
                    if let Some(data) = msg["data"].as_str()
                        && let Ok(bytes) = B64.decode(data) {
                            pcm_buffer.write().extend_from_slice(&bytes);
                        }
                }
                Some("silence_detected") => {
                    let buffer = std::mem::take(&mut *pcm_buffer.write());
                    if buffer.is_empty() {
                        continue;
                    }
                    if let Err(e) = process_voice_pipeline(&buffer, widget).await {
                        widget.send_update(serde_json::json!({
                            "type": "error",
                            "message": e.to_string(),
                        }));
                    }
                }
                Some("start_standby") | Some("cancel") => {
                    pcm_buffer.write().clear();
                }
                Some("playback_complete") => {}
                _ => {}
            }
        }
    });

    rsx! {
        div { id: "{eid_rsx}", class: "w-full h-full bg-[var(--surface-container-lowest)]" }
    }
}

async fn process_voice_pipeline(pcm: &[u8], widget: widget_bridge::TsWidgetHandle) -> anyhow::Result<()> {
    let wav = audio_core::wrap_pcm_as_wav(pcm, audio_core::SAMPLE_RATE, audio_core::CHANNELS, audio_core::BITS_PER_SAMPLE);
    let audio_data = B64.encode(&wav);

    let transcription = whisper_client::WHISPER.infer(&TranscribeRequest { audio_data, language: None }).await?;

    let text = transcription.text.trim().to_owned();
    widget.send_update(serde_json::json!({ "type": "transcript", "text": text }));

    if text.is_empty() {
        return Ok(());
    }

    let response = crate::voice_backend::ClaudeCliBackend.query(&text).await?;
    widget.send_update(serde_json::json!({ "type": "agent_response", "text": response }));

    let speech_req = SpeechRequest { text: response, voice: "af_heart".into(), lang_code: "a".into(), speed: 1.0 };
    let wav_bytes = kokoro_client::KOKORO.infer(&speech_req).await?;
    let chunks = audio_core::chunk_wav(&wav_bytes, 32768);
    for chunk in chunks {
        widget.send_update(serde_json::json!({ "type": "audio_response", "data": B64.encode(&chunk) }));
    }
    Ok(())
}
```

### Step 5: Voice widget TS integration

The voice widget must be part of the `widget-bridge` bundle, not a separate `voice-client.js` bundle. A separate IIFE bundle creates its own `widgets` Map via its own copy of `registerWidget`, so the voice widget registers into a Map that `widget-bridge.js` never sees.

Create `ts/widget-bridge/widgets/voice.ts` containing the voice widget implementation. Import from `../src/registry` (same package, same Map). Add the side-effect import `import "../widgets/voice"` to `ts/widget-bridge/src/index.ts`.

Add `@lx/audio-capture` and `@lx/audio-playback` as dependencies to `ts/widget-bridge/package.json`. Add TS path mappings for these packages to `ts/widget-bridge/tsconfig.json`. Widen `rootDir` to `".."` to accommodate cross-package source resolution.

Remove `voice-client.js` from `build.rs` copy list and from `app.rs` script loading.

### Step 6: JS global registration

Dioxus desktop wraps every `document::eval()` call in `new AsyncFunction("dioxus", script)(dioxus)` (see `reference/dioxus/packages/desktop/src/query.rs` line 80). This creates a function scope where `var` declarations are local, not global. An IIFE like `var WidgetBridge = (function(exports) { ... })({})` does NOT create a persistent global when eval'd. Appending `; window.WidgetBridge = WidgetBridge;` externally in Rust also fails — the IIFE may error inside the AsyncFunction wrapper before the assignment executes.

The TS source must explicitly assign to the window object inside the module itself. This applies to both `widget-bridge` and `dx-charts`.

Add to `ts/widget-bridge/src/index.ts`:

```typescript
import * as self from "./index";

declare global {
  interface Window {
    WidgetBridge: typeof self;
  }
}

window.WidgetBridge = self;
```

Add to `ts/dx-charts/src/index.ts`:

```typescript
import * as self from "./index";

declare global {
  interface Window {
    DxCharts: typeof self;
  }
}

window.DxCharts = self;
```

These compile into the IIFE bundles, so `window.WidgetBridge` and `window.DxCharts` are set during module initialization and persist for subsequent eval calls.

### Step 7: JS loading in desktop mode

JS assets must be loaded via `include_str!` + `document::eval()` in a `use_hook` in the Shell component, not via `document::Script` tags. `document::Script` creates script tags asynchronously — the `WidgetBridge` global may not exist when components try to use it.

In `layout/shell.rs`, all three JS bundles are loaded:
```rust
#[cfg(feature = "desktop")]
const ECHARTS_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/echarts-5.5.1.min.js"));
#[cfg(feature = "desktop")]
const DX_CHARTS_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/dx-charts.js"));
#[cfg(feature = "desktop")]
const WIDGET_BRIDGE_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/widget-bridge.js"));

// In Shell component:
#[cfg(feature = "desktop")]
use_hook(|| {
    document::eval(ECHARTS_JS);
    document::eval(DX_CHARTS_JS);
    document::eval(WIDGET_BRIDGE_JS);
});
```

For web builds, `app.rs` declares static assets with `with_static_head(true)` which injects blocking `<script>` tags into the initial HTML. These statics are underscore-prefixed since they are not referenced in rsx — manganis picks them up via link sections in the binary regardless:
```rust
static _ECHARTS_JS: Asset = asset!("/assets/echarts-5.5.1.min.js", AssetOptions::js().with_static_head(true));
static _DX_CHARTS_JS: Asset = asset!("/assets/dx-charts.js", AssetOptions::js().with_static_head(true));
static _WIDGET_BRIDGE_JS: Asset = asset!("/assets/widget-bridge.js", AssetOptions::js().with_static_head(true));
```

CSS is loaded at runtime via `document::Stylesheet` since it works for both desktop and web:
```rust
static TAILWIND_CSS: Asset = asset!("/assets/tailwind.css", AssetOptions::css().with_static_head(true));

// In App rsx:
document::Stylesheet { href: TAILWIND_CSS }
```

`with_static_head(true)` only takes effect for web builds (the CLI injects into `index.html`). It has no effect on desktop builds, where the hardcoded webview HTML template is used instead.

## Verification

```bash
cd crates/lx-desktop && cargo clippy --features desktop -- -D warnings
```

Must pass with zero errors and zero warnings.

## Runtime Prerequisites

To actually use the voice pane (not required for compilation):

1. `claude` CLI on PATH — the `ClaudeCliBackend` shells out to `claude -p "<text>" --output-format text`
2. Whisper inference server at `WHISPER_URL` env var (default `http://localhost:8095`)
3. Kokoro inference server at `KOKORO_URL` env var (default `http://localhost:8094`)

Without these, the voice pane will show errors when you try to speak. All other pane types work independently.

## Manual Test

1. `just desktop` (runs `dx serve -p lx-desktop`)
2. Click the `+` button in the tab bar → select "Voice"
3. A voice pane appears with Start/Stop buttons and a transcript area
4. Click Start → speak → wait for silence detection → observe transcript + agent response + audio playback

### Step 8: View container backgrounds

All view components in `view.rs` were updated with design system CSS variable backgrounds as part of the UX design system update:
- TerminalView: `bg-[var(--surface-container-lowest)] p-[1.1rem]`
- EditorView, VoiceView: `bg-[var(--surface-container-lowest)]`
- BrowserView, AgentView, CanvasView: `bg-[var(--surface-container)]`
- ChartView: `bg-[var(--surface-container)]`

## Implementation Notes

Issues encountered during implementation:

- **Clippy `manual_let_else`**: The `match widget.recv() { Ok(msg) => msg, Err(_) => break }` pattern must use `let Ok(msg) = ... else { break }` form.
- **Clippy `collapsible_if`**: Nested `if let Some(data) = ... { if let Ok(bytes) = ... { ... } }` must be collapsed into a single `if let ... && let ...` chain.
- **Tokio `process` feature**: The workspace tokio dependency does not include the `process` feature. The lx-desktop Cargo.toml must add `features = ["process"]` to its tokio dependency.
- **Separate IIFE bundles don't share state**: A voice-client.js built as a separate IIFE creates its own `widgets` Map. The voice widget must be part of the widget-bridge bundle.
- **AsyncFunction scope isolation**: `document::eval()` wraps JS in `new AsyncFunction(...)`, making `var` declarations local. Window global assignments must be inside the TS source, not appended externally in Rust.
- **`document::Script` is async in desktop**: Script tags created via `document::Script` load asynchronously in the webview. JS globals may not exist when components mount. Use `include_str!` + `document::eval()` in `use_hook` for synchronous loading.
- **`with_static_head(true)` is web-only**: The Dioxus CLI injects static head assets into `index.html` at build time. Desktop uses a hardcoded HTML template that the CLI does not modify.
