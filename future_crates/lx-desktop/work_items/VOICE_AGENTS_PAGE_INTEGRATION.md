# Voice Pipeline — Agents Page Integration

## Goal

Migrate the voice pipeline from the pane system into the Agents page. The JS voice widget becomes a headless audio engine (no DOM rendering). VoiceBanner becomes the full voice UI: mounts the headless widget, manages voice state via Dioxus signals, runs the STT-LLM-TTS pipeline, and renders status/transcript/controls in RSX. The PUSH TO TALK button controls the pipeline directly.

## Why

- Voice is conceptually an agent interaction surface, not a generic pane — it belongs on the Agents page next to agent cards
- The current VoiceBanner PUSH TO TALK button is decorative with no event handlers
- The JS widget renders its own HTML UI that duplicates what Dioxus should own — violates the idiomatic widget-bridge split where JS handles browser-native APIs and Dioxus handles all UI and state

## What changes

Two files rewritten, one file unchanged.

## Files affected

| File | Change |
|------|--------|
| `crates/lx-desktop/assets/voice-client.js` | Strip all DOM rendering from voice widget; mount becomes headless (audio engine only); update handler sends status_change messages instead of manipulating DOM; add "start_capture" / "stop_capture" command handling |
| `crates/lx-desktop/src/pages/agents/voice_banner.rs` | Rewrite from static 18-line stub to full voice integration component: mount headless widget, Dioxus signals for status/pcm_buffer/transcript, use_future message loop, inline pipeline logic, full RSX UI with working PUSH TO TALK |
| `crates/lx-desktop/src/voice_backend.rs` | No change — ClaudeCliBackend stays as-is, called from voice_banner |

## Task List

### Task 1: Strip UI from voice-client.js — headless audio engine

Rewrite `crates/lx-desktop/assets/voice-client.js`. Keep all three classes unchanged: VoiceActivityDetector, AudioCapture, AudioPlayback. Keep the registerWidget call and the states Map. Rewrite the widget registration as follows.

The `mount` function receives `(elementId, config, dx)`. It creates AudioCapture (sampleRate 16000) and AudioPlayback. It stores state in the states Map as `{ capture, playback, status: "idle", dx }` — no DOM elements in state. It wires capture.onChunk to send `{ type: "audio_chunk", data: b64pcm, seq: capture.currentSeq }` via dx.send. It wires capture.onSilence to check `state.status === "listening"`, then stop capture, set `state.status = "processing"`, send `{ type: "status_change", status: "processing" }` and `{ type: "silence_detected" }` via dx.send. It wires playback.onComplete to set `state.status = "idle"`, send `{ type: "status_change", status: "idle" }` and `{ type: "playback_complete" }` via dx.send. It does NOT create any DOM elements — the mount div stays empty.

The `update` function receives `(elementId, data)`. It gets state from the states Map. It handles these message types from Rust:
- `"start_capture"`: if status is "idle", call `capture.start()` (async — use `.then()`), set status to "listening", send `{ type: "status_change", status: "listening" }` and `{ type: "start_standby" }` via dx.send
- `"stop_capture"`: call `capture.stop()` and `playback.stop()`, set status to "idle", send `{ type: "status_change", status: "idle" }` and `{ type: "cancel" }` via dx.send
- `"audio_response"`: set status to "speaking", send `{ type: "status_change", status: "speaking" }`, enqueue `msg.data` on playback

Remove the `setStatus` helper function entirely. Remove the `addEntry` helper function entirely. The `resize` function stays as empty no-op. The `dispose` function stays the same (dispose capture, dispose playback, delete from states, clear innerHTML).

Keep the IIFE wrapper, the module exports (AudioCapture, AudioPlayback, VoiceActivityDetector), and the sourcemap comment at the end.

### Task 2: Rewrite VoiceBanner as voice integration component

Rewrite `crates/lx-desktop/src/pages/agents/voice_banner.rs`. Add these imports at the top:

```rust
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use dioxus::prelude::*;
use kokoro_client::SpeechRequest;
use voice_agent::AgentBackend as _;
use whisper_client::InferenceClient as _;
use whisper_client::TranscribeRequest;
use widget_bridge::use_ts_widget;
```

Define a VoiceStatus enum with variants Idle, Listening, Processing, Speaking — derive Clone, Copy, PartialEq. Implement Display for VoiceStatus returning the uppercase string for each variant ("IDLE", "LISTENING", "PROCESSING", "SPEAKING").

Define a TranscriptEntry struct with fields: `is_user: bool` and `text: String` — derive Clone.

Rewrite the VoiceBanner component. It calls `use_ts_widget("voice", serde_json::json!({}))` and destructures into `(element_id, widget)`. It creates three signals:
- `let mut status: Signal<VoiceStatus> = use_signal(|| VoiceStatus::Idle);`
- `let mut pcm_buffer: Signal<Vec<u8>> = use_signal(Vec::new);`
- `let mut transcript: Signal<Vec<TranscriptEntry>> = use_signal(Vec::new);`

Clone widget into `widget_loop` for the message loop closure. Set up a `use_future` that loops on `widget_loop.recv::<serde_json::Value>().await`. Match on `msg["type"].as_str()`:
- `"audio_chunk"`: decode base64 data from `msg["data"]` via B64.decode, extend pcm_buffer
- `"silence_detected"`: take the buffer via `std::mem::take(&mut *pcm_buffer.write())`, skip if empty, set `status.set(VoiceStatus::Processing)`, call `process_voice_pipeline(&buffer, widget_loop, &mut status, &mut transcript).await`, on error send `{ type: "error", message: e.to_string() }` to widget and set status back to Idle
- `"status_change"`: match `msg["status"].as_str()` — "idle" sets Idle, "listening" sets Listening, "processing" sets Processing, "speaking" sets Speaking
- `"start_standby"` / `"cancel"`: clear pcm_buffer
- `"playback_complete"`: set status to Idle
- wildcard: ignore

Clone widget into `widget_rsx` for use in the onclick handlers. Render the RSX:

```rust
let current_status = status();
let is_idle = current_status == VoiceStatus::Idle;
let is_active = !is_idle;
let status_text = current_status.to_string();
let entries = transcript.read().clone();
let bar_glow = if is_active { "shadow-[0_0_12px_var(--primary)]" } else { "" };

rsx! {
    div { class: "flex flex-col gap-2",
        div { class: "bg-[var(--surface-container)] rounded-lg px-4 py-2 flex items-center gap-3 {bar_glow}",
            span { class: "text-[var(--primary)] text-sm",
                if is_active { "\u{1F534}" } else { "\u{1F512}" }
            }
            span { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]",
                "{status_text}"
            }
            if is_active {
                span { class: "text-[var(--primary)] text-sm ml-1 animate-pulse", "\u{2581}\u{2582}\u{2583}\u{2584}" }
            }
            div { class: "flex-1" }
            button {
                class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150 font-semibold",
                onclick: move |_| {
                    if is_idle {
                        widget_rsx.send_update(serde_json::json!({ "type": "start_capture" }));
                    } else {
                        widget_rsx.send_update(serde_json::json!({ "type": "stop_capture" }));
                    }
                },
                if is_idle { "PUSH TO TALK" } else { "STOP" }
            }
        }
        if !entries.is_empty() {
            div { class: "bg-[var(--surface-container-lowest)] rounded-lg px-4 py-3 max-h-48 overflow-y-auto text-sm space-y-1",
                for entry in entries.iter() {
                    div {
                        class: if entry.is_user { "text-[#64b5f6]" } else { "text-[#81c784]" },
                        if entry.is_user { "You: {entry.text}" } else { "Agent: {entry.text}" }
                    }
                }
            }
        }
        div { id: "{element_id}", class: "hidden" }
    }
}
```

After the component, define the pipeline function:

```rust
async fn process_voice_pipeline(
    pcm: &[u8],
    widget: widget_bridge::TsWidgetHandle,
    status: &mut Signal<VoiceStatus>,
    transcript: &mut Signal<Vec<TranscriptEntry>>,
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
    if text.is_empty() {
        status.set(VoiceStatus::Idle);
        return Ok(());
    }
    transcript.write().push(TranscriptEntry { is_user: true, text: text.clone() });
    let response = crate::voice_backend::ClaudeCliBackend.query(&text).await?;
    transcript.write().push(TranscriptEntry { is_user: false, text: response.clone() });
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

### Task 3: Update agents page imports

Edit `crates/lx-desktop/src/pages/agents/mod.rs`. No changes needed — VoiceBanner is already imported and rendered at the top of the Agents component. Verify the existing `use self::voice_banner::VoiceBanner;` and `VoiceBanner {}` in the RSX are present and correct.

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
