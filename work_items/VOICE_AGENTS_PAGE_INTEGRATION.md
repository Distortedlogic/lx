# Voice Pipeline — Agents Page Integration

## Goal

Migrate the voice pipeline from the pane system into the Agents page. The voice TypeScript widget becomes a headless audio engine (no DOM rendering). VoiceBanner becomes the full voice UI: mounts the headless widget, manages voice state via Dioxus signals, runs the STT-LLM-TTS pipeline, and renders status/transcript/controls in RSX. The PUSH TO TALK button controls the pipeline directly.

## Why

- Voice is conceptually an agent interaction surface, not a generic pane — it belongs on the Agents page next to agent cards
- The current VoiceBanner PUSH TO TALK button is decorative with no event handlers
- The JS widget renders its own HTML UI that duplicates what Dioxus should own — violates the idiomatic widget-bridge split where JS handles browser-native APIs and Dioxus handles all UI and state

## Architecture: status ownership

JS is the single source of truth for audio state. The JS widget tracks its internal `state.status` and sends `status_change` messages to Rust whenever the status transitions. Rust ONLY updates the Dioxus `status` signal from these `status_change` messages — the pipeline function never sets status directly. This eliminates dual-update races between Rust signal writes and incoming JS messages.

When the pipeline completes without producing audio (empty transcription or error), Rust sends `stop_capture` to JS, which resets JS state to idle and sends `status_change("idle")` back. When audio IS produced, JS transitions through `speaking` → `idle` automatically via `playback.onComplete`.

## Build system context

The voice widget source is `ts/widget-bridge/widgets/voice.ts`. It is compiled by Vite into the `widget-bridge.js` bundle. The Rust `build.rs` at `crates/lx-desktop/build.rs` automatically runs `pnpm build` in `ts/widget-bridge/` and copies the output to `crates/lx-desktop/assets/widget-bridge.js` during `cargo check`/`cargo build`. No manual JS build step is needed — editing the `.ts` source is sufficient.

The file `crates/lx-desktop/assets/voice-client.js` is a dead artifact from before the voice widget was merged into the widget-bridge bundle. It has its own `widgets` Map that `widget-bridge.js` never reads. It is not loaded by `app.rs`. It must be deleted.

## Dioxus Signal API note

`Signal<T>` in Dioxus is `Copy` (confirmed at `reference/dioxus/packages/signals/src/signal.rs:550`). It uses interior mutability — call `.set()`, `.write()`, `.read()` on the value directly. Never pass `&mut Signal<T>`. Pass `Signal<T>` by value (it's a lightweight generational box ID).

## Files affected

| File | Change |
|------|--------|
| `ts/widget-bridge/widgets/voice.ts` | Rewrite: strip all DOM rendering, make headless audio engine, handle start_capture/stop_capture/audio_response commands from Rust |
| `crates/lx-desktop/src/pages/agents/voice_banner.rs` | Rewrite: mount headless widget, Dioxus signals for status/pcm_buffer/transcript, use_future message loop, inline pipeline, full RSX UI |
| `crates/lx-desktop/assets/voice-client.js` | Delete (dead file, not loaded by app.rs) |

## Task List

### Task 1: Rewrite voice widget as headless audio engine

Replace the contents of `ts/widget-bridge/widgets/voice.ts` with:

```typescript
import { AudioCapture } from "@lx/audio-capture";
import { AudioPlayback } from "@lx/audio-playback";
import { registerWidget } from "../src/registry";
import type { Widget } from "../src/registry";
import type { Dioxus } from "../src/types";

type VoiceStatus = "idle" | "listening" | "processing" | "speaking";

interface VoiceState {
  capture: AudioCapture;
  playback: AudioPlayback;
  status: VoiceStatus;
  dx: Dioxus;
}

const states = new Map<string, VoiceState>();

function transition(state: VoiceState, status: VoiceStatus): void {
  state.status = status;
  state.dx.send({ type: "status_change", status });
}

const voiceWidget: Widget = {
  mount(elementId: string, _config: unknown, dx: Dioxus) {
    const capture = new AudioCapture({ sampleRate: 16000 });
    const playback = new AudioPlayback();

    const state: VoiceState = { capture, playback, status: "idle", dx };
    states.set(elementId, state);

    capture.onChunk = (b64pcm: string) => {
      dx.send({ type: "audio_chunk", data: b64pcm, seq: capture.currentSeq });
    };

    capture.onSilence = () => {
      if (state.status === "listening") {
        capture.stop();
        transition(state, "processing");
        dx.send({ type: "silence_detected" });
      }
    };

    playback.onComplete = () => {
      transition(state, "idle");
      dx.send({ type: "playback_complete" });
    };
  },

  update(elementId: string, data: unknown) {
    const state = states.get(elementId);
    if (!state) return;

    const msg = data as { type: string; data?: string };

    switch (msg.type) {
      case "start_capture":
        if (state.status !== "idle") return;
        state.capture.start().then(() => {
          transition(state, "listening");
          state.dx.send({ type: "start_standby" });
        });
        break;
      case "stop_capture":
        state.capture.stop();
        state.playback.stop();
        transition(state, "idle");
        state.dx.send({ type: "cancel" });
        break;
      case "audio_response":
        transition(state, "speaking");
        if (msg.data) state.playback.enqueue(msg.data);
        break;
    }
  },

  resize(_elementId: string) {},

  dispose(elementId: string) {
    const state = states.get(elementId);
    if (state) {
      state.capture.dispose();
      state.playback.dispose();
      states.delete(elementId);
    }
  },
};

registerWidget("voice", voiceWidget);
```

Key changes from the current file:
- `VoiceState` no longer has `statusEl` or `transcriptEl` DOM references
- `setStatus` replaced by `transition` which only updates internal state and sends `status_change` to Rust — no DOM manipulation
- `addEntry` deleted entirely (transcript rendering moves to Dioxus)
- `mount` creates no DOM elements — no container, no buttons, no status/transcript divs
- Button click handlers removed — Rust sends `start_capture`/`stop_capture` commands via the `update` handler
- `update` handles three message types from Rust: `start_capture`, `stop_capture`, `audio_response` (replaces old `transcript`, `agent_response`, `audio_response`, `error`)
- `dispose` no longer calls `el.innerHTML = ""` since mount creates nothing

### Task 2: Rewrite VoiceBanner as voice integration component

Replace the contents of `crates/lx-desktop/src/pages/agents/voice_banner.rs` with:

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use dioxus::prelude::*;
use kokoro_client::SpeechRequest;
use voice_agent::AgentBackend as _;
use whisper_client::InferenceClient as _;
use whisper_client::TranscribeRequest;
use widget_bridge::use_ts_widget;

#[derive(Clone, Copy, PartialEq)]
enum VoiceStatus {
    Idle,
    Listening,
    Processing,
    Speaking,
}

impl std::fmt::Display for VoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "IDLE"),
            Self::Listening => write!(f, "LISTENING"),
            Self::Processing => write!(f, "PROCESSING"),
            Self::Speaking => write!(f, "SPEAKING"),
        }
    }
}

#[derive(Clone)]
struct TranscriptEntry {
    is_user: bool,
    text: String,
}

#[component]
pub fn VoiceBanner() -> Element {
    let (element_id, widget) = use_ts_widget("voice", serde_json::json!({}));
    let mut status: Signal<VoiceStatus> = use_signal(|| VoiceStatus::Idle);
    let mut pcm_buffer: Signal<Vec<u8>> = use_signal(Vec::new);
    let mut transcript: Signal<Vec<TranscriptEntry>> = use_signal(Vec::new);

    use_future(move || async move {
        loop {
            let Ok(msg) = widget.recv::<serde_json::Value>().await else {
                break;
            };
            match msg["type"].as_str() {
                Some("audio_chunk") => {
                    if let Some(data) = msg["data"].as_str()
                        && let Ok(bytes) = B64.decode(data)
                    {
                        pcm_buffer.write().extend_from_slice(&bytes);
                    }
                }
                Some("silence_detected") => {
                    let buffer = std::mem::take(&mut *pcm_buffer.write());
                    if buffer.is_empty() {
                        widget.send_update(serde_json::json!({ "type": "stop_capture" }));
                        continue;
                    }
                    match process_voice_pipeline(&buffer, widget, transcript).await {
                        Ok(true) => {}
                        Ok(false) => {
                            widget.send_update(serde_json::json!({ "type": "stop_capture" }));
                        }
                        Err(e) => {
                            transcript.write().push(TranscriptEntry {
                                is_user: false,
                                text: format!("Error: {e}"),
                            });
                            widget.send_update(serde_json::json!({ "type": "stop_capture" }));
                        }
                    }
                }
                Some("status_change") => match msg["status"].as_str() {
                    Some("idle") => status.set(VoiceStatus::Idle),
                    Some("listening") => status.set(VoiceStatus::Listening),
                    Some("processing") => status.set(VoiceStatus::Processing),
                    Some("speaking") => status.set(VoiceStatus::Speaking),
                    _ => {}
                },
                Some("start_standby") | Some("cancel") => {
                    pcm_buffer.write().clear();
                }
                Some("playback_complete") => {}
                _ => {}
            }
        }
    });

    let current_status = status();
    let is_active = current_status != VoiceStatus::Idle;
    let status_text = current_status.to_string();
    let entries = transcript.read().clone();
    let bar_glow = if is_active {
        "shadow-[0_0_12px_var(--primary)]"
    } else {
        ""
    };
    let icon = if is_active { "\u{1F534}" } else { "\u{1F512}" };
    let button_label = if status() == VoiceStatus::Idle {
        "PUSH TO TALK"
    } else {
        "STOP"
    };

    rsx! {
        div { class: "flex flex-col gap-2",
            div {
                class: "bg-[var(--surface-container)] rounded-lg px-4 py-2 flex items-center gap-3 {bar_glow}",
                span { class: "text-[var(--primary)] text-sm", "{icon}" }
                span {
                    class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]",
                    "{status_text}"
                }
                if is_active {
                    span {
                        class: "text-[var(--primary)] text-sm ml-1 animate-pulse",
                        "\u{2581}\u{2582}\u{2583}\u{2584}"
                    }
                }
                div { class: "flex-1" }
                button {
                    class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150 font-semibold",
                    onclick: move |_| {
                        if status() == VoiceStatus::Idle {
                            widget.send_update(serde_json::json!({ "type": "start_capture" }));
                        } else {
                            widget.send_update(serde_json::json!({ "type": "stop_capture" }));
                        }
                    },
                    "{button_label}"
                }
            }
            if !entries.is_empty() {
                div {
                    class: "bg-[var(--surface-container-lowest)] rounded-lg px-4 py-3 max-h-48 overflow-y-auto text-sm space-y-1",
                    for entry in entries.iter() {
                        div {
                            class: if entry.is_user {
                                "text-[#64b5f6]"
                            } else {
                                "text-[#81c784]"
                            },
                            if entry.is_user {
                                "You: {entry.text}"
                            } else {
                                "Agent: {entry.text}"
                            }
                        }
                    }
                }
            }
            div { id: "{element_id}", class: "hidden" }
        }
    }
}

async fn process_voice_pipeline(
    pcm: &[u8],
    widget: widget_bridge::TsWidgetHandle,
    transcript: Signal<Vec<TranscriptEntry>>,
) -> anyhow::Result<bool> {
    let wav = audio_core::wrap_pcm_as_wav(
        pcm,
        audio_core::SAMPLE_RATE,
        audio_core::CHANNELS,
        audio_core::BITS_PER_SAMPLE,
    );
    let audio_data = B64.encode(&wav);
    let transcription = whisper_client::WHISPER
        .infer(&TranscribeRequest {
            audio_data,
            language: None,
        })
        .await?;
    let text = transcription.text.trim().to_owned();
    if text.is_empty() {
        return Ok(false);
    }
    transcript
        .write()
        .push(TranscriptEntry { is_user: true, text: text.clone() });
    let response = crate::voice_backend::ClaudeCliBackend.query(&text).await?;
    transcript
        .write()
        .push(TranscriptEntry { is_user: false, text: response.clone() });
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
    Ok(true)
}
```

Key design decisions in the Rust component:

**Status reads in onclick use `status()` at click time** (not the pre-rendered `is_active` bool) to avoid stale state if the signal changed between the last render and the click.

**`process_voice_pipeline` returns `Result<bool>`** — `true` means audio chunks were sent (JS handles speaking→idle transition via playback.onComplete), `false` means empty transcription (caller sends stop_capture to reset JS). On `Err`, caller pushes the error to transcript and sends stop_capture.

**Pipeline takes `Signal<Vec<TranscriptEntry>>` by value** (not `&mut`) because Dioxus Signal is Copy with interior mutability.

**The hidden div** (`div { id: "{element_id}", class: "hidden" }`) is the widget mount point. The widget-bridge `runWidgetBridge` polls for this element's existence before calling mount (see `ts/widget-bridge/src/registry.ts:28`). Dioxus renders the full RSX tree before effects run, so the element exists by the time JS looks for it. The headless widget creates no DOM children inside it.

**`element_id` is a `String`** (not Copy). It is NOT captured by the `use_future(move || ...)` closure because the closure body does not reference it — only `widget` (Copy), `pcm_buffer` (Copy Signal), `status` (Copy Signal), and `transcript` (Copy Signal) are captured. So `element_id` remains available for the RSX block.

### Task 3: Delete dead voice-client.js

Delete the file `crates/lx-desktop/assets/voice-client.js`. This file is not loaded by `app.rs` (only `widget-bridge.js` is loaded via `static _WIDGET_BRIDGE_JS`). It contains a duplicate voice widget registered in its own isolated `widgets` Map that `widget-bridge.js` never reads. The active voice widget lives inside the `widget-bridge.js` bundle, compiled from `ts/widget-bridge/widgets/voice.ts`.

### Task 4: Verify agents page wiring

Verify `crates/lx-desktop/src/pages/agents/mod.rs` — no changes needed. Confirm these lines are present and unchanged:

- `mod voice_banner;` (line 3)
- `use self::voice_banner::VoiceBanner;` (line 9)
- `VoiceBanner {}` in the Agents RSX (line 15)

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "future_crates/lx-desktop/work_items/VOICE_AGENTS_PAGE_INTEGRATION.md" })
```

Then call `next_task` to begin.
