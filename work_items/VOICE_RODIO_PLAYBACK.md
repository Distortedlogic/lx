# Goal

Move TTS audio playback from the browser (HTMLAudioElement via JS voice widget) to Rust-native playback via rodio. Eliminates GStreamer, base64 encoding, blob URLs, and browser audio pipeline entirely. Audio capture (microphone) stays in JS.

# Why

The current path: Rust TTS → WAV bytes → base64 encode → send to JS voice widget → base64 decode → blob URL → HTMLAudioElement → GStreamer → PipeWire → speakers. This produces GStreamer-CRITICAL warnings on WebKit2GTK (single-value caps range bug), startup clipping (PipeWire suspend-on-idle), and requires a 50ms silence prepend hack.

The new path: Rust TTS → WAV bytes → rodio Sink → PipeWire → speakers. Direct. No browser audio stack. No GStreamer. No base64. No blob URLs. No silence prepend.

# Architecture

Dioxus desktop uses a multi-threaded tokio runtime (confirmed in `reference/dioxus/packages/desktop/src/launch.rs:119`). The voice pipeline runs in `spawn(async move { ... })` on tokio worker threads. rodio creates its own audio output thread via cpal, which talks directly to ALSA/PipeWire. Three independent thread pools: tao/GTK event loop (main thread), tokio workers (pipeline), rodio audio (playback). No conflicts.

rodio's `OutputStream` must stay alive for the duration of playback. It's created once (lazily on first TTS) and held in a `LazyLock<(OutputStream, OutputStreamHandle)>`. The `Sink` is created per-playback from the handle.

The voice widget (JS) stops handling `audio_response` messages. It keeps handling `start_capture`, `stop_capture`, and audio capture callbacks. The `playback_complete` status transition moves from JS to Rust (after rodio's `Sink::sleep_until_end()`).

# Verified facts

- rodio 0.22.2 is the latest version. It decodes WAV via symphonia by default.
- Kokoro outputs 24kHz 16-bit mono WAV with proper RIFF headers. rodio/symphonia decodes this.
- cpal defaults to ALSA backend on Linux. ALSA calls go through PipeWire's ALSA compatibility layer on Fedora. No extra deps needed.
- `Sink::sleep_until_end()` is async-blocking (it blocks the current thread). In a tokio context, use `tokio::task::spawn_blocking` to avoid blocking the executor.
- The 50ms silence prepend becomes unnecessary — rodio doesn't have HTMLAudioElement's startup latency, and PipeWire suspend-on-idle affects the GStreamer path, not the ALSA/cpal path.

# Files Affected

| File | Change |
|------|--------|
| `Cargo.toml` | Add rodio dependency |
| `src/pages/agents/voice_banner.rs` | Replace JS audio_response with rodio playback |
| `dioxus-common/ts/widget-bridge/widgets/voice.ts` | Remove AudioPlayback import and usage |

# Task List

### Task 1: Add rodio dependency

**Subject:** Add rodio to lx-desktop Cargo.toml

**Description:** Edit `crates/lx-desktop/Cargo.toml`. Add this line in the `[dependencies]` section, after the `base64` line:

```toml
rodio = { version = "0.22", default-features = false, features = ["symphonia-wav"] }
```

`default-features = false` avoids pulling in decoders for MP3, FLAC, Vorbis, AAC — we only need WAV. The `symphonia-wav` feature enables WAV decoding via symphonia.

**ActiveForm:** Adding rodio dependency

---

### Task 2: Replace JS audio playback with rodio in voice_banner.rs

**Subject:** Play TTS audio directly from Rust instead of sending to the browser

**Description:** Edit `crates/lx-desktop/src/pages/agents/voice_banner.rs`.

**Remove the `prepend_silence` function** (lines 176-206). It's no longer needed — rodio doesn't have HTMLAudioElement's startup latency.

**Rewrite the `tts` function** (lines 170-174). Remove the `prepend_silence` call:

```rust
async fn tts(text: &str) -> anyhow::Result<Vec<u8>> {
  let req = SpeechRequest { text: text.to_owned(), voice: "am_michael".into(), lang_code: "a".into(), speed: 1.2 };
  common_kokoro::KOKORO.infer(&req).await
}
```

**Rewrite the TTS playback section in `run_pipeline`** (lines 239-247). Replace:

```rust
  ctx.pipeline_stage.set(PipelineStage::SynthesizingSpeech);
  let wav_bytes = tts(&response).await?;
  ctx.pending.write().insert("r0".to_string(), response);
  voice_widget.send_update(serde_json::json!({
      "type": "audio_response",
      "data": B64.encode(&wav_bytes),
      "id": "r0",
  }));
  ctx.pipeline_stage.set(PipelineStage::Idle);
```

With:

```rust
  ctx.pipeline_stage.set(PipelineStage::SynthesizingSpeech);
  let wav_bytes = tts(&response).await?;

  ctx.status.set(VoiceStatus::Speaking);

  let transcript_entry = response.clone();
  tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
    let cursor = std::io::Cursor::new(wav_bytes);
    let (_stream, handle) = rodio::OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&handle)?;
    let source = rodio::Decoder::new(cursor)?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
  }).await??;

  let mut t = ctx.transcript.write();
  match t.last_mut() {
    Some(entry) if !entry.is_user => entry.text.push_str(&format!(" {transcript_entry}")),
    _ => t.push(TranscriptEntry { is_user: false, text: transcript_entry }),
  }

  voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
  ctx.pipeline_stage.set(PipelineStage::Idle);
```

Key details:
- `tokio::task::spawn_blocking` moves the rodio playback to a blocking thread pool, not the tokio executor. `Sink::sleep_until_end()` blocks that thread until playback finishes. The `await` on `spawn_blocking` suspends the pipeline future without blocking tokio.
- `OutputStream` is created inside `spawn_blocking` and dropped when the closure returns (after playback ends). Creating it per-playback is simpler than managing a global static. The overhead of opening/closing the audio device is negligible compared to TTS synthesis time.
- `ctx.status.set(VoiceStatus::Speaking)` sets the Rust-side status directly before playback. This works because `run_pipeline` runs inside `dioxus::spawn()` which executes on the main thread (Dioxus scheduler), not a tokio worker thread. Signal::set() is safe here.
- After playback, `voice_widget.send_update(stop_capture)` tells the JS voice widget to transition to "idle." JS calls `transition(state, "idle")` which updates `state.status` AND sends `status_change(idle)` back to Rust. The Rust message loop receives this and sets `ctx.status` to Idle. This keeps JS and Rust status in sync — JS needs `state.status == "idle"` to accept the next `start_capture`.
- The `audio_playing` / `audio_response` / `playback_complete` message types are no longer sent. The `pending` map insertion is removed — the transcript entry is written directly after playback. Task 4 removes the dead handlers. Task 5 removes the `pending` field from VoiceContext.

**Remove the `base64` import if no longer used.** After this change, `B64.encode` is still used in the transcription step (line 216: `B64.encode(&wav)` for whisper). So the import stays.

**ActiveForm:** Replacing JS audio playback with rodio

---

### Task 3: Remove AudioPlayback from voice widget

**Subject:** Strip playback handling from voice.ts since audio plays from Rust

**Description:** Edit `/home/entropybender/repos/dioxus-common/ts/widget-bridge/widgets/voice.ts`.

Remove the `AudioPlayback` import (line 2):
```typescript
import { AudioPlayback } from "@dioxus-common/audio-playback";
```

Remove `playback` from `VoiceState` interface (line 11):
```typescript
  playback: AudioPlayback;
```

Remove `AudioPlayback` construction in `mount` (line 26):
```typescript
const playback = new AudioPlayback();
```

Remove `playback` from the state object (line 28). The state becomes:
```typescript
const state: VoiceState = { capture, status: "idle", dx };
```

Remove `playback.onComplete` callback (lines 47-50):
```typescript
    playback.onComplete = () => {
      transition(state, "idle");
      dx.send({ type: "playback_complete" });
    };
```

Remove `playback.onItemStart` callback (lines 52-54):
```typescript
    playback.onItemStart = (id: string) => {
      dx.send({ type: "audio_playing", id });
    };
```

Remove the `audio_response` case from the update switch (lines 77-80):
```typescript
      case "audio_response":
        transition(state, "speaking");
        if (msg.data) state.playback.enqueue(msg.data, msg.id ?? "");
        break;
```

Remove `state.playback.stop()` from the `stop_capture` case (line 73). It becomes:
```typescript
      case "stop_capture":
        state.capture.stop();
        transition(state, "idle");
        state.dx.send({ type: "cancel" });
        break;
```

Remove `state.playback.dispose()` from `dispose` (line 90). It becomes:
```typescript
      state.capture.dispose();
```

The voice widget now only handles audio capture (microphone). All playback is Rust-side.

**ActiveForm:** Removing AudioPlayback from voice widget

---

### Task 4: Clean up dead voice message handlers in VoiceBanner

**Subject:** Remove handlers for messages that are no longer sent

**Description:** Edit `crates/lx-desktop/src/pages/agents/voice_banner.rs`. In the voice widget message loop (the first `use_future`, lines 19-72), remove these match arms that handle messages no longer sent by the JS voice widget:

Remove the `audio_playing` arm (lines 44-53):
```rust
        Some("audio_playing") => {
          if let Some(id) = msg["id"].as_str()
            && let Some(text) = ctx.pending.write().remove(id)
          {
            let mut t = ctx.transcript.write();
            match t.last_mut() {
              Some(entry) if !entry.is_user => entry.text.push_str(&format!(" {text}")),
              _ => t.push(TranscriptEntry { is_user: false, text }),
            }
          }
        },
```

Remove the `playback_complete` arm (line 68):
```rust
        Some("playback_complete") => {},
```

These message types (`audio_playing`, `playback_complete`) were sent by `AudioPlayback.onItemStart` and `AudioPlayback.onComplete` in voice.ts. Since AudioPlayback is removed in Task 3, these messages are never sent.

**ActiveForm:** Cleaning up dead voice message handlers

---

### Task 5: Remove pending field from VoiceContext

**Subject:** Remove the unused pending HashMap signal

**Description:** Edit `crates/lx-desktop/src/pages/agents/voice_context.rs`.

Remove the `pending` field from the `VoiceContext` struct (line 52):
```rust
pub pending: Signal<HashMap<String, String>>,
```

Remove its initialization in `provide()` (line 64):
```rust
pending: Signal::new(HashMap::new()),
```

Remove the `HashMap` import (line 2):
```rust
use std::collections::HashMap;
```

The `pending` map was used to associate audio chunk IDs with transcript text for the `audio_playing` JS callback. Since audio playback moved to Rust and the `audio_playing` handler was removed in Task 4, the map has no readers or writers.

**ActiveForm:** Removing pending field from VoiceContext

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/VOICE_RODIO_PLAYBACK.md" })
```
