# Voice Pipeline Integration Plan

## Prerequisites

Read `/home/entropybender/repos/lx/CLAUDE.md` first and follow all rules in it. Key rules that apply here:
- No code comments or doc strings
- No `#[allow(...)]` macros
- No `.unwrap()` — use `?` or `.expect("reason")`
- Do not swallow errors — propagate or surface them
- 300 line file limit
- Run `cargo clippy -p lx-desktop -- -D warnings` to verify — must pass with zero warnings

## Goal

Wire the voice chat pipeline into the lx-desktop Dioxus app. A human opens a Voice pane, speaks into their microphone, speech is transcribed (Whisper STT), sent to Claude CLI for an LLM response, and the response is spoken back (Kokoro TTS). All infrastructure crates and TS packages already exist and compile. This plan covers only the remaining integration code — 4 file changes total.

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

### Step 1: Add dependencies to `crates/lx-desktop/Cargo.toml`

The current `[dependencies]` section is:

```toml
[dependencies]
pane-tree = { path = "../pane-tree" }
pty-mux = { path = "../pty-mux" }
widget-bridge = { path = "../widget-bridge" }
base64 = "0.22"
dioxus = { version = "0.7", features = ["desktop", "router"] }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
uuid = { version = "1", features = ["v4"] }
```

Add these 6 lines (4 local crates + 2 new external deps):

```toml
audio-core = { path = "../audio-core" }
whisper-client = { path = "../whisper-client" }
kokoro-client = { path = "../kokoro-client" }
voice-agent = { path = "../voice-agent" }
anyhow = "1"
async-trait = "0.1"
```

### Step 2: Create `crates/lx-desktop/src/voice_backend.rs`

Create this new file with this exact content:

```rust
use voice_agent::AgentBackend;

pub struct ClaudeCliBackend;

#[async_trait::async_trait]
impl AgentBackend for ClaudeCliBackend {
    async fn query(&self, text: &str) -> anyhow::Result<String> {
        let output = tokio::process::Command::new("claude")
            .args(["-p", text, "--output-format", "text"])
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("claude cli failed: {stderr}");
        }
        let response = String::from_utf8(output.stdout)?;
        Ok(response.trim().to_owned())
    }
}
```

### Step 3: Add module declaration to `crates/lx-desktop/src/main.rs`

The current file is:

```rust
mod app;
mod layout;
mod pages;
mod panes;
mod routes;
mod terminal;

fn main() {
    dioxus::launch(app::App);
}
```

Add `mod voice_backend;` after `mod terminal;`:

```rust
mod app;
mod layout;
mod pages;
mod panes;
mod routes;
mod terminal;
mod voice_backend;

fn main() {
    dioxus::launch(app::App);
}
```

### Step 4: Rewrite VoiceView in `crates/lx-desktop/src/terminal/view.rs`

This file contains 7 view components in this order: `TerminalView`, `BrowserView`, `EditorView`, `AgentView`, `CanvasView`, `ChartView`, `VoiceView`. **Do NOT modify any component except VoiceView.** VoiceView is the last component in the file (lines 161-171).

#### Step 4a: Add imports

The current import block at the top of `view.rs` is:

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use dioxus::prelude::*;
use pane_tree::TabsState;
use pane_tree::{NotificationLevel, PaneNotification};
use widget_bridge::use_ts_widget;

use super::use_tabs_state;
use crate::panes::DesktopPane;
```

Add these 4 lines after `use widget_bridge::use_ts_widget;` and before `use super::use_tabs_state;`:

```rust
use kokoro_client::SpeechRequest;
use voice_agent::AgentBackend as _;
use whisper_client::InferenceClient as _;
use whisper_client::TranscribeRequest;
```

The `as _` imports bring the traits into scope so `.infer()` and `.query()` can be called on the concrete types, without creating unused-import warnings (since the trait names themselves are never referenced directly).

#### Step 4b: Replace VoiceView

Delete the current VoiceView stub (lines 161-171):

```rust
#[component]
pub fn VoiceView(voice_id: String) -> Element {
    let (element_id, _widget) = use_ts_widget("voice", serde_json::json!({}));

    rsx! {
        div {
            id: "{element_id}",
            class: "w-full h-full",
        }
    }
}
```

Replace with:

```rust
#[component]
pub fn VoiceView(voice_id: String) -> Element {
    let (element_id, widget) = use_ts_widget("voice", serde_json::json!({}));
    let mut pcm_buffer: Signal<Vec<u8>> = use_signal(Vec::new);

    let eid_rsx = element_id.clone();
    use_future(move || async move {
        loop {
            let msg = match widget.recv::<serde_json::Value>().await {
                Ok(msg) => msg,
                Err(_) => break,
            };

            match msg["type"].as_str() {
                Some("audio_chunk") => {
                    if let Some(data) = msg["data"].as_str() {
                        if let Ok(bytes) = B64.decode(data) {
                            pcm_buffer.write().extend_from_slice(&bytes);
                        }
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
        div {
            id: "{eid_rsx}",
            class: "w-full h-full",
        }
    }
}

async fn process_voice_pipeline(
    pcm: &[u8],
    widget: widget_bridge::TsWidgetHandle,
) -> anyhow::Result<()> {
    let wav = audio_core::wrap_pcm_as_wav(
        pcm,
        audio_core::SAMPLE_RATE,
        audio_core::CHANNELS,
        audio_core::BITS_PER_SAMPLE,
    );
    let audio_data = B64.encode(&wav);

    let transcription = whisper_client::WHISPER
        .infer(&TranscribeRequest { audio_data, language: None })
        .await?;

    let text = transcription.text.trim().to_owned();
    widget.send_update(serde_json::json!({
        "type": "transcript",
        "text": text,
    }));

    if text.is_empty() {
        return Ok(());
    }

    let response = crate::voice_backend::ClaudeCliBackend.query(&text).await?;
    widget.send_update(serde_json::json!({
        "type": "agent_response",
        "text": response,
    }));

    let speech_req = SpeechRequest {
        text: response,
        voice: "af_heart".into(),
        lang_code: "a".into(),
        speed: 1.0,
    };
    let wav_bytes = kokoro_client::KOKORO.infer(&speech_req).await?;
    let chunks = audio_core::chunk_wav(&wav_bytes, 32768);
    for chunk in chunks {
        widget.send_update(serde_json::json!({
            "type": "audio_response",
            "data": B64.encode(&chunk),
        }));
    }
    Ok(())
}
```

Key implementation details:
- `pcm_buffer` is a `Signal<Vec<u8>>` — Dioxus signals are `Copy`, so it can be used inside `use_future` without cloning
- `widget` is `TsWidgetHandle` which is also `Copy` — same applies
- `process_voice_pipeline()` is a standalone async function that takes the PCM buffer by reference and the widget handle. It propagates all errors via `?`. The caller in VoiceView catches the error and sends it to the widget for display
- `std::mem::take()` on `pcm_buffer.write()` swaps the buffer contents with an empty Vec, giving ownership of the accumulated PCM to the processing pipeline while clearing the buffer for the next recording
- `B64` is already imported at line 2 of the file as `use base64::engine::general_purpose::STANDARD as B64`

## Verification

After making all 4 changes:

```bash
cargo clippy -p lx-desktop -- -D warnings
```

Must pass with zero errors and zero warnings. Common pitfalls:
- `clippy::unwrap_used` — do not use `.unwrap()` anywhere
- `clippy::needless_pass_by_value` — if clippy suggests `&str`, change the parameter
- `dead_code` — every type/function must be used

## Runtime Prerequisites

To actually use the voice pane (not required for compilation):

1. `claude` CLI on PATH — the `ClaudeCliBackend` shells out to `claude -p "<text>" --output-format text`
2. Whisper inference server at `WHISPER_URL` env var (default `http://localhost:8095`)
3. Kokoro inference server at `KOKORO_URL` env var (default `http://localhost:8094`)

Without these, the voice pane will show errors when you try to speak. All other pane types work independently.

## Manual Test

1. `cargo run -p lx-desktop`
2. Right-click the `+` button in the tab bar → select "Voice"
3. A voice pane appears with Start/Stop buttons and a transcript area
4. Click Start → speak → wait for silence detection → observe transcript + agent response + audio playback
