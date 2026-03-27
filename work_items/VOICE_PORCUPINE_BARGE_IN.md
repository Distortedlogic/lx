# Voice Pipeline — Porcupine Keyword Detection + Barge-In

## Goal

Replace the Whisper-based trigger word loop with Porcupine keyword detection (via the Porcupine C library loaded through Rust FFI) and add barge-in — the ability to interrupt TTS playback mid-speech by saying the keyword. When Porcupine is unavailable (missing library/key), fall back to the existing Whisper-based trigger detection without barge-in.

## Why

The current Whisper-based standby loop has two fundamental problems:

1. **No barge-in.** The mic is gated by status — no audio flows during Speaking. The user must wait for TTS to finish before re-activating. Real voice assistants let you interrupt mid-speech.

2. **High latency for keyword detection.** Every utterance goes through silence detection (2s) + Whisper transcription (~200ms) + string matching. Porcupine detects keywords in <100ms from a continuous audio stream, without waiting for silence.

Porcupine also handles echo robustness — the TTS audio coming through speakers into the mic doesn't cause false triggers because the keyword model is trained to distinguish the specific wake word pattern from ambient noise/speech.

## Architecture

### Why Porcupine C library via Rust FFI (not the Web SDK)

The Dioxus desktop app loads JS via `document::eval(WIDGET_BRIDGE_JS)` — inline evaluation with no origin URL. Porcupine's Web SDK loads its WASM binary via `fetch()` or `WebAssembly.instantiateStreaming()`, which fails without a URL origin. The Picovoice Rust crate (`pv_porcupine`) is deprecated since July 2025. The C library (`libpv_porcupine.so`) is the stable foundation all SDKs use. Loading it directly via `libloading` gives us the latest library without depending on a deprecated crate.

### Audio flow

```
JS AudioWorklet (16kHz PCM)
    │
    ▼ onChunk (always, no status gate)
voice.ts sends audio_chunk to Rust
    │
    ▼ voice_banner.rs audio_chunk handler
    ├── always: decode base64 → i16 samples → feed to Porcupine frame buffer
    │     └── for each 512-sample frame: porcupine.process()
    │           └── keyword detected → handle_keyword_detected()
    │
    └── if status == Listening: also accumulate in pcm_buffer for Whisper
```

### State machine

```
[Idle] ──start_standby_listen──▶ [Standby] (mic on, Porcupine processing every frame)
                                     │
                                     │ Porcupine keyword detected
                                     ▼
                                 ack tone + start_recording
                                     │
                                     ▼
                                [Listening] ──silence_detected──▶ [Processing]
                                                                     │
                                                                Whisper → LLM → TTS
                                                                     │
                                                                     ▼
                                                                [Speaking] (Porcupine still processing)
                                                                     │
                                     ┌───────────────────────────────┤
                                     │                               │
                              keyword detected                 playback ends
                              (barge-in)                       naturally
                                     │                               │
                              stop playback                          │
                              ack + start_recording                  │
                                     │                               │
                                     ▼                               ▼
                                [Listening]                     [Standby]
                                (new query)

Push-to-talk (always_listen off, unchanged):
[Idle] ──start_capture──▶ [Listening] ──silence──▶ [Processing] ──▶ ... ──▶ [Idle]
```

Two modes, controlled by `always_listen`:
- **Porcupine available + always_listen ON:** Porcupine keyword detection in standby + barge-in during speaking
- **Porcupine unavailable + always_listen ON:** Existing Whisper-based standby loop, no barge-in (fallback)
- **always_listen OFF:** Push-to-talk, no keyword detection (unchanged)

### Porcupine C API (5 functions via libloading)

```c
pv_status_t pv_porcupine_init(
    const char *access_key, const char *model_path,
    int32_t num_keywords, const char *const *keyword_paths,
    const float *sensitivities, pv_porcupine_t **object);

void pv_porcupine_delete(pv_porcupine_t *object);

pv_status_t pv_porcupine_process(
    pv_porcupine_t *object, const int16_t *pcm, int32_t *keyword_index);

int32_t pv_porcupine_frame_length(void);  // returns 512
int32_t pv_sample_rate(void);              // returns 16000
```

`pv_status_t` is `i32`: 0 = success, nonzero = error.

### Barge-in via rodio Player::stop()

`rodio::Player::stop()` sets an internal `AtomicBool` (`controls.stopped`) which causes the audio source iterator to yield `None`, which signals the `sleep_until_end()` receiver to complete. `Player` is `Send + Sync` (all fields are `Arc`/`Mutex`/`Atomic`-based). This means `stop()` can be called from the async executor thread while `sleep_until_end()` blocks in `spawn_blocking`.

Implementation:
```rust
static ACTIVE_PLAYER: LazyLock<std::sync::Mutex<Option<Arc<rodio::Player>>>> =
    LazyLock::new(|| std::sync::Mutex::new(None));
```

`play_wav_interruptible`: creates `Arc<Player>`, stores a clone in `ACTIVE_PLAYER`, calls `sleep_until_end()` on the original, clears `ACTIVE_PLAYER` after return.

`stop_active_playback`: takes `Arc<Player>` from `ACTIVE_PLAYER`, calls `stop()`. The other `Arc` (in `spawn_blocking`) is still alive — `sleep_until_end()` returns when the source stops.

### Barge-in coordination

When `stop_active_playback()` is called, `run_pipeline`'s `sleep_until_end()` returns early. The pipeline continues to update transcript and set `pipeline_stage = Idle`. Meanwhile, `handle_keyword_detected` already sent `start_recording` to JS. Without coordination, the pipeline's post-completion logic (send `resume_standby`) would override the `start_recording`.

Fix: `barge_in: Signal<bool>` in VoiceContext. `handle_keyword_detected` sets it `true` before stopping playback. The pipeline spawn's post-completion check: if `barge_in` is true, clear it and don't send `resume_standby`. The `handle_keyword_detected` handler already managed the state transition.

Ordering guarantee: `handle_keyword_detected` runs in the `use_future` message loop. The pipeline `spawn` runs as a separate Dioxus task. The message loop processes the keyword event first (setting `barge_in = true` and calling `stop_active_playback`), then yields at the next `recv().await`. The `spawn_blocking` thread (now unblocked by `stop()`) completes, and the pipeline spawn task runs next, sees `barge_in = true`, and skips `resume_standby`.

### Echo during Speaking

The TTS plays through rodio (native ALSA/PipeWire). The mic captures via WebAudio (WebKit2GTK). The mic picks up TTS through room acoustics. Porcupine is designed to detect wake words in noisy environments — it fires only when its trained keyword pattern matches with sufficient confidence (tunable via `sensitivity`, default 0.5). The TTS voice ("am_michael") is spectrally distinct from the user saying "Computer" or a custom keyword. False triggers from TTS echo are unlikely at default sensitivity. If they occur, lower sensitivity to 0.3.

### Configuration

Three environment variables, all optional:

| Variable | Purpose | If missing |
|----------|---------|-----------|
| `PICOVOICE_ACCESS_KEY` | Picovoice account key (free at console.picovoice.ai) | Porcupine disabled, Whisper fallback |
| `PICOVOICE_MODEL_PATH` | Path to `porcupine_params.pv` | Porcupine disabled |
| `PICOVOICE_KEYWORD_PATH` | Path to `.ppn` keyword file (e.g., `Computer_en_linux_v3_0_0.ppn`) | Porcupine disabled |

The library path for `libpv_porcupine.so` is resolved via `PICOVOICE_LIBRARY_PATH` env var, or falls back to standard library search (`LD_LIBRARY_PATH`, `/usr/local/lib`, etc.).

When Porcupine is disabled, the ALWAYS LISTEN button still works using the existing Whisper-based standby loop (no barge-in). The audio_chunk handler skips the Porcupine feed path.

## Files affected

| File | Repo | Change |
|------|------|--------|
| `ts/widget-bridge/widgets/voice.ts` | dioxus-common | Always send chunks, onSilence only in listening, add `start_recording` command, remove `awaiting_query` command |
| `crates/lx-desktop/Cargo.toml` | lx | Add `libloading = "0.8"` |
| `crates/lx-desktop/src/pages/agents/voice_porcupine.rs` | lx | NEW: FFI wrapper for libpv_porcupine.so, lazy init, frame buffer + process |
| `crates/lx-desktop/src/pages/agents/voice_context.rs` | lx | Remove AwaitingQuery, add porcupine_buffer + barge_in signals, add porcupine_available |
| `crates/lx-desktop/src/pages/agents/voice_pipeline.rs` | lx | Remove match_trigger, add ACTIVE_PLAYER + play_wav_interruptible + stop_active_playback |
| `crates/lx-desktop/src/pages/agents/voice_banner.rs` | lx | Porcupine audio routing, keyword_detected handler, barge-in coordination, simplified handle_utterance |
| `crates/lx-desktop/src/pages/agents/mod.rs` | lx | Add `mod voice_porcupine;` |

## Task List

### Task 1: Modify voice widget for always-on chunk delivery

Rewrite `/home/entropybender/repos/dioxus-common/ts/widget-bridge/widgets/voice.ts`:

```typescript
import { AudioCapture } from "@dioxus-common/audio-capture";
import { registerWidget } from "../src/registry";
import type { Widget } from "../src/registry";
import type { Dioxus } from "../src/types";

type VoiceStatus = "idle" | "standby" | "listening" | "processing" | "speaking";

interface VoiceState {
  capture: AudioCapture;
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
    const state: VoiceState = { capture, status: "idle", dx };
    states.set(elementId, state);

    capture.onChunk = (b64pcm: string) => {
      if (state.status !== "idle") {
        dx.send({ type: "audio_chunk", data: b64pcm, seq: capture.currentSeq });
      }
    };

    capture.onSilence = () => {
      if (state.status === "listening") {
        capture.stop();
        transition(state, "processing");
        dx.send({ type: "silence_detected" });
      }
    };

    capture.onRms = (rms: number) => {
      dx.send({ type: "rms", level: rms });
    };
  },

  update(elementId: string, data: unknown) {
    const state = states.get(elementId);
    if (!state) return;
    const msg = data as { type: string };

    switch (msg.type) {
      case "start_capture":
        if (state.status !== "idle") return;
        state.capture.start().then(() => {
          transition(state, "listening");
          state.dx.send({ type: "start_standby" });
        });
        break;
      case "start_standby_listen":
        if (state.status !== "idle") return;
        state.capture.start().then(() => {
          transition(state, "standby");
        });
        break;
      case "start_recording":
        if (state.status !== "standby" && state.status !== "speaking") return;
        state.capture.resetVad();
        transition(state, "listening");
        break;
      case "resume_standby":
        if (!state.capture.isRunning) {
          state.capture.start().then(() => {
            state.capture.resetVad();
            transition(state, "standby");
          });
        } else {
          state.capture.resetVad();
          transition(state, "standby");
        }
        break;
      case "stop_capture":
        state.capture.stop();
        transition(state, "idle");
        state.dx.send({ type: "cancel" });
        break;
    }
  },

  resize(_elementId: string) {},

  dispose(elementId: string) {
    const state = states.get(elementId);
    if (state) {
      state.capture.dispose();
      states.delete(elementId);
    }
  },
};

registerWidget("voice", voiceWidget);
```

Key changes from current:
- **`hasSpeech` flag removed.** No longer needed — Porcupine handles keyword detection, not silence-then-check.
- **`onChunk` sends in ALL non-idle states** (`standby`, `listening`, `processing`, `speaking`). Rust decides what to do with the audio (Porcupine feed vs pcm_buffer).
- **`onSilence` only fires in `"listening"` state.** In standby, Porcupine handles detection. In processing/speaking, silence is irrelevant.
- **`start_recording` command added.** Resets VAD and transitions to `"listening"`. Used after Porcupine keyword detection to begin recording the query. Accepts both `"standby"` and `"speaking"` as source states (the latter for barge-in).
- **`awaiting_query` command removed.** No longer needed — Porcupine fires instantly, no "trigger word only" edge case.
- **`resume_standby` kept** for post-pipeline return to standby. Still checks `capture.isRunning` and restarts if needed.

### Task 2: Create Porcupine FFI module

Create `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/agents/voice_porcupine.rs`:

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{LazyLock, Mutex};

use dioxus::logger::tracing::{error, info, warn};

type PvPorcupineT = std::ffi::c_void;

struct PorcupineLib {
    _lib: libloading::Library,
    init: unsafe extern "C" fn(*const c_char, *const c_char, i32, *const *const c_char, *const f32, *mut *mut PvPorcupineT) -> i32,
    delete: unsafe extern "C" fn(*mut PvPorcupineT),
    process: unsafe extern "C" fn(*mut PvPorcupineT, *const i16, *mut i32) -> i32,
    frame_length: unsafe extern "C" fn() -> i32,
}

unsafe impl Send for PorcupineLib {}
unsafe impl Sync for PorcupineLib {}

struct PorcupineEngine {
    lib: PorcupineLib,
    handle: *mut PvPorcupineT,
    frame_len: usize,
}

unsafe impl Send for PorcupineEngine {}
unsafe impl Sync for PorcupineEngine {}

impl Drop for PorcupineEngine {
    fn drop(&mut self) {
        unsafe { (self.lib.delete)(self.handle) };
    }
}

impl PorcupineEngine {
    fn process(&self, frame: &[i16]) -> Option<i32> {
        let mut keyword_index: i32 = -1;
        let status = unsafe { (self.lib.process)(self.handle, frame.as_ptr(), &mut keyword_index) };
        if status != 0 {
            return None;
        }
        if keyword_index >= 0 { Some(keyword_index) } else { None }
    }
}

static ENGINE: LazyLock<Option<PorcupineEngine>> = LazyLock::new(|| {
    let access_key = std::env::var("PICOVOICE_ACCESS_KEY").ok()?;
    let model_path = std::env::var("PICOVOICE_MODEL_PATH").ok()?;
    let keyword_path = std::env::var("PICOVOICE_KEYWORD_PATH").ok()?;
    let lib_path = std::env::var("PICOVOICE_LIBRARY_PATH").unwrap_or_else(|_| "libpv_porcupine.so".into());

    let lib = match unsafe { libloading::Library::new(&lib_path) } {
        Ok(l) => l,
        Err(e) => { warn!("porcupine: failed to load {lib_path}: {e}"); return None; },
    };

    let (init_fn, delete_fn, process_fn, frame_length_fn) = unsafe {
        let init = *lib.get::<unsafe extern "C" fn(*const c_char, *const c_char, i32, *const *const c_char, *const f32, *mut *mut PvPorcupineT) -> i32>(b"pv_porcupine_init\0").ok()?;
        let delete = *lib.get::<unsafe extern "C" fn(*mut PvPorcupineT)>(b"pv_porcupine_delete\0").ok()?;
        let process = *lib.get::<unsafe extern "C" fn(*mut PvPorcupineT, *const i16, *mut i32) -> i32>(b"pv_porcupine_process\0").ok()?;
        let frame_length = *lib.get::<unsafe extern "C" fn() -> i32>(b"pv_porcupine_frame_length\0").ok()?;
        (init, delete, process, frame_length)
    };

    let porcupine_lib = PorcupineLib { _lib: lib, init: init_fn, delete: delete_fn, process: process_fn, frame_length: frame_length_fn };
    let frame_len = unsafe { (porcupine_lib.frame_length)() } as usize;

    let c_access_key = CString::new(access_key).ok()?;
    let c_model_path = CString::new(model_path).ok()?;
    let c_keyword_path = CString::new(keyword_path).ok()?;
    let keyword_paths = [c_keyword_path.as_ptr()];
    let sensitivities = [0.5f32];
    let mut handle: *mut PvPorcupineT = std::ptr::null_mut();

    let status = unsafe {
        (porcupine_lib.init)(
            c_access_key.as_ptr(), c_model_path.as_ptr(),
            1, keyword_paths.as_ptr(), sensitivities.as_ptr(),
            &mut handle,
        )
    };
    if status != 0 || handle.is_null() {
        error!("porcupine: init failed with status {status}");
        return None;
    }

    info!("porcupine: initialized, frame_length={frame_len}");
    Some(PorcupineEngine { lib: porcupine_lib, handle, frame_len })
});

static FRAME_BUFFER: LazyLock<Mutex<Vec<i16>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn is_available() -> bool {
    ENGINE.is_some()
}

pub fn feed_samples(samples: &[i16]) -> Option<i32> {
    let engine = ENGINE.as_ref()?;
    let mut buf = FRAME_BUFFER.lock().unwrap();
    buf.extend_from_slice(samples);
    let mut detected = None;
    while buf.len() >= engine.frame_len {
        let frame: Vec<i16> = buf.drain(..engine.frame_len).collect();
        if let Some(idx) = engine.process(&frame) {
            detected = Some(idx);
        }
    }
    detected
}

pub fn reset_buffer() {
    FRAME_BUFFER.lock().unwrap().clear();
}
```

Also add `mod voice_porcupine;` to `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/agents/mod.rs`.

Also add `libloading = "0.8"` to `[dependencies]` in `/home/entropybender/repos/lx/crates/lx-desktop/Cargo.toml`.

FFI safety notes:
- `PorcupineEngine` wraps a raw C pointer. `Send + Sync` is manually implemented because the C library is thread-safe (documented by Picovoice: "Porcupine is thread-safe for `process()` calls from different threads on the same instance").
- `PorcupineLib` holds the `Library` to keep the .so loaded for the lifetime of the engine.
- `ENGINE` is a `LazyLock<Option<...>>` — `None` when any env var is missing or init fails. All code paths check `is_available()` or `ENGINE.as_ref()?` before use.
- `FRAME_BUFFER` accumulates samples across audio_chunk messages and drains in 512-sample frames (Porcupine's required frame size).
- `feed_samples` returns `Some(keyword_index)` if a keyword was detected in any frame of this batch, `None` otherwise.

### Task 3: Update VoiceContext

In `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/agents/voice_context.rs`:

Remove `AwaitingQuery` from `PipelineStage` enum and its `Display` impl.

Remove `trigger_words: Signal<Vec<String>>` from `VoiceContext` struct and `provide()`.

Add to `VoiceContext`:
```rust
pub barge_in: Signal<bool>,
```

Initialize in `provide()`:
```rust
barge_in: Signal::new(false),
```

### Task 4: Add interruptible playback to voice_pipeline.rs

In `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/agents/voice_pipeline.rs`:

**Remove `match_trigger` function entirely** (lines 24-43).

**Add** after the existing `AUDIO_SINK` static:

```rust
static ACTIVE_PLAYER: LazyLock<std::sync::Mutex<Option<std::sync::Arc<rodio::Player>>>> =
    LazyLock::new(|| std::sync::Mutex::new(None));

pub fn stop_active_playback() {
    if let Some(player) = ACTIVE_PLAYER.lock().unwrap().take() {
        player.stop();
    }
}
```

**Add** `play_wav_interruptible` alongside the existing `play_wav`:

```rust
pub fn play_wav_interruptible(wav_bytes: Vec<u8>) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::task::spawn_blocking(move || {
        let cursor = std::io::Cursor::new(wav_bytes);
        let player = rodio::play(AUDIO_SINK.mixer(), cursor).map_err(|e| {
            error!("voice: rodio::play failed: {e}");
            e
        })?;
        let player = std::sync::Arc::new(player);
        *ACTIVE_PLAYER.lock().unwrap() = Some(std::sync::Arc::clone(&player));
        player.sleep_until_end();
        *ACTIVE_PLAYER.lock().unwrap() = None;
        info!("voice: playback finished");
        Ok(())
    })
}
```

**Modify `run_pipeline`** to use `play_wav_interruptible` instead of `play_wav`:

Replace the existing playback section (from `let wav_len` through `play_result??`) with:
```rust
let wav_len = wav_bytes.len();
info!("voice: TTS returned {wav_len} bytes, starting playback");
let play_result = play_wav_interruptible(wav_bytes).await;
match &play_result {
    Ok(Ok(())) => {},
    Ok(Err(e)) => error!("voice: playback error: {e}"),
    Err(e) => error!("voice: spawn_blocking panicked: {e}"),
}
play_result??;
```

### Task 5: Rewrite voice_banner.rs with Porcupine integration

Replace `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/agents/voice_banner.rs`:

```rust
use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
use super::{voice_pipeline, voice_porcupine};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use dioxus::logger::tracing::error;
use dioxus::prelude::*;
use dioxus_widget_bridge::use_ts_widget;

#[component]
pub fn VoiceBanner() -> Element {
    let (voice_element_id, voice_widget) = use_ts_widget("voice", serde_json::json!({}));
    let (agent_element_id, agent_widget) = use_ts_widget("agent", serde_json::json!({}));
    let mut ctx = use_context::<VoiceContext>();
    use_effect(move || {
        ctx.widget.set(Some(voice_widget));
    });

    use_future(move || async move {
        loop {
            let Ok(msg) = voice_widget.recv::<serde_json::Value>().await else {
                break;
            };
            match msg["type"].as_str() {
                Some("audio_chunk") => {
                    if let Some(data) = msg["data"].as_str()
                        && let Ok(bytes) = B64.decode(data)
                    {
                        if voice_porcupine::is_available() {
                            let samples: Vec<i16> = bytes
                                .chunks_exact(2)
                                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                                .collect();
                            if voice_porcupine::feed_samples(&samples).is_some() {
                                handle_keyword_detected(voice_widget, ctx);
                            }
                        }
                        if (ctx.status)() == VoiceStatus::Listening {
                            ctx.pcm_buffer.write().extend_from_slice(&bytes);
                        }
                    }
                },
                Some("silence_detected") => {
                    let stage = (ctx.pipeline_stage)();
                    if stage != PipelineStage::Idle {
                        ctx.pcm_buffer.write().clear();
                        continue;
                    }

                    let buffer = std::mem::take(&mut *ctx.pcm_buffer.write());
                    if buffer.is_empty() {
                        if (ctx.always_listen)() {
                            voice_widget.send_update(serde_json::json!({ "type": "resume_standby" }));
                        } else {
                            voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
                        }
                        continue;
                    }

                    spawn(async move {
                        let result = handle_utterance(buffer, agent_widget, ctx).await;
                        if (ctx.barge_in)() {
                            ctx.barge_in.set(false);
                            return;
                        }
                        if let Err(e) = &result {
                            error!("voice: pipeline error: {e}");
                            ctx.transcript.write().push(TranscriptEntry {
                                is_user: false,
                                text: format!("Error: {e}"),
                            });
                            ctx.pipeline_stage.set(PipelineStage::Idle);
                        }
                        if (ctx.always_listen)() {
                            voice_widget.send_update(serde_json::json!({ "type": "resume_standby" }));
                        } else {
                            voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
                        }
                    });
                },
                Some("rms") => {
                    if let Some(level) = msg["level"].as_f64() {
                        ctx.rms.set(level as f32);
                    }
                },
                Some("status_change") => match msg["status"].as_str() {
                    Some("idle") => ctx.status.set(VoiceStatus::Idle),
                    Some("standby") => ctx.status.set(VoiceStatus::Standby),
                    Some("listening") => ctx.status.set(VoiceStatus::Listening),
                    Some("processing") => ctx.status.set(VoiceStatus::Processing),
                    Some("speaking") => ctx.status.set(VoiceStatus::Speaking),
                    _ => {},
                },
                Some("start_standby") | Some("cancel") => ctx.pcm_buffer.write().clear(),
                _ => {},
            }
        }
    });

    use_future(move || async move {
        loop {
            let Ok(msg) = agent_widget.recv::<serde_json::Value>().await else {
                break;
            };
            if let Some("user_message") = msg["type"].as_str() {
                let content = msg["content"].as_str().unwrap_or("").to_owned();
                if content.is_empty() {
                    continue;
                }
                match crate::voice_backend::query_streaming(&content, |chunk| {
                    agent_widget.send_update(
                        serde_json::json!({ "type": "assistant_chunk", "text": chunk }),
                    );
                })
                .await
                {
                    Ok(_) => {
                        agent_widget.send_update(serde_json::json!({ "type": "assistant_done" }));
                    },
                    Err(e) => {
                        agent_widget.send_update(
                            serde_json::json!({ "type": "error", "message": format!("{e:#}") }),
                        );
                    },
                }
            }
        }
    });

    let current_status = (ctx.status)();
    let is_active = current_status != VoiceStatus::Idle;
    let status_text = current_status.to_string();
    let bar_glow = if is_active { "shadow-[0_0_12px_var(--primary)]" } else { "" };
    let icon = match current_status {
        VoiceStatus::Standby => "\u{1F7E2}",
        _ if is_active => "\u{1F534}",
        _ => "\u{1F512}",
    };
    let volume = ((ctx.rms)() / 0.3).min(1.0);
    let stage = (ctx.pipeline_stage)();
    let entries = ctx.transcript.read();
    let turn_count = entries.iter().filter(|e| e.is_user).count();
    drop(entries);
    let always_listen = (ctx.always_listen)();

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "bg-[var(--surface-container)] px-4 py-2 flex items-center gap-3 shrink-0 {bar_glow}",
                span { class: "text-[var(--primary)] text-sm", "{icon}" }
                span { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]",
                    "{status_text}"
                }
                if is_active {
                    div { class: "flex items-end gap-[2px] h-4 ml-1",
                        span {
                            class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
                            style: "height: {(volume * 40.0).max(2.0)}%;",
                        }
                        span {
                            class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
                            style: "height: {(volume * 70.0).max(2.0)}%;",
                        }
                        span {
                            class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
                            style: "height: {(volume * 100.0).max(2.0)}%;",
                        }
                        span {
                            class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
                            style: "height: {(volume * 60.0).max(2.0)}%;",
                        }
                    }
                    span { class: "text-[10px] text-[var(--outline)] uppercase tracking-wider",
                        "{stage}"
                    }
                    span { class: "text-[10px] text-[var(--outline)] uppercase tracking-wider",
                        "TURNS: {turn_count}"
                    }
                }
                div { class: "flex-1" }
                button {
                    class: "border border-[var(--outline-variant)] text-[var(--on-surface-variant)] rounded px-3 py-1.5 text-xs uppercase hover:bg-[var(--surface-container-high)] transition-colors duration-150 font-semibold",
                    onclick: move |_| {
                        if always_listen {
                            ctx.always_listen.set(false);
                            voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
                        } else {
                            ctx.always_listen.set(true);
                            voice_widget.send_update(serde_json::json!({ "type": "start_standby_listen" }));
                        }
                    },
                    if always_listen { "ALWAYS LISTEN: ON" } else { "ALWAYS LISTEN: OFF" }
                }
                if is_active {
                    button {
                        class: "border border-[var(--outline)] text-[var(--on-surface)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--surface-container-high)] transition-colors duration-150 font-semibold",
                        onclick: move |_| {
                            ctx.always_listen.set(false);
                            voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
                        },
                        "STOP"
                    }
                } else {
                    button {
                        class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150 font-semibold",
                        onclick: move |_| {
                            voice_widget.send_update(serde_json::json!({ "type": "start_capture" }));
                        },
                        "PUSH TO TALK"
                    }
                }
            }
            div {
                id: "{agent_element_id}",
                class: "flex-1 min-h-0 overflow-hidden",
            }
            div { id: "{voice_element_id}", class: "hidden" }
        }
    }
}

fn handle_keyword_detected(
    voice_widget: dioxus_widget_bridge::TsWidgetHandle,
    mut ctx: VoiceContext,
) {
    let status = (ctx.status)();
    if status != VoiceStatus::Standby && status != VoiceStatus::Speaking {
        return;
    }
    if status == VoiceStatus::Speaking {
        ctx.barge_in.set(true);
        voice_pipeline::stop_active_playback();
    }
    ctx.pcm_buffer.write().clear();
    voice_porcupine::reset_buffer();
    voice_widget.send_update(serde_json::json!({ "type": "start_recording" }));
    spawn(async move {
        let _ = voice_pipeline::play_wav(voice_pipeline::generate_ack_tone()).await;
    });
}

async fn handle_utterance(
    pcm: Vec<u8>,
    agent_widget: dioxus_widget_bridge::TsWidgetHandle,
    mut ctx: VoiceContext,
) -> anyhow::Result<()> {
    ctx.pipeline_stage.set(PipelineStage::Transcribing);
    let text = voice_pipeline::transcribe(&pcm).await?;
    if text.is_empty() {
        ctx.pipeline_stage.set(PipelineStage::Idle);
        return Ok(());
    }
    voice_pipeline::run_pipeline(&text, agent_widget, ctx).await
}
```

Key changes from current:

**`audio_chunk` handler has two paths.** First: if Porcupine is available, decode bytes to i16 samples and feed to `voice_porcupine::feed_samples()`. If keyword detected, call `handle_keyword_detected()`. Second: if status is Listening, accumulate in pcm_buffer for Whisper. Both paths run on every chunk.

**`silence_detected` simplified.** No `is_awaiting`/`is_standby` branching. The concurrent pipeline guard checks `stage != Idle` (no AwaitingQuery). `handle_utterance` just transcribes and runs the pipeline — no trigger matching.

**Barge-in coordination.** In the spawn block after `handle_utterance`, if `barge_in` is true, clear it and return — don't send resume_standby or stop_capture, because `handle_keyword_detected` already sent `start_recording`.

**`handle_keyword_detected`** runs synchronously in the message loop (called from audio_chunk handler). Only fires in Standby or Speaking. In Speaking: sets `barge_in = true`, calls `stop_active_playback()`. In both: clears pcm_buffer + Porcupine frame buffer, sends `start_recording` to JS, spawns ack tone playback.

**`handle_utterance` is simplified.** No trigger word matching. Transcribe → pipeline. The trigger detection already happened via Porcupine (or the user pressed PTT). Returns `anyhow::Result<()>` (no bool needed — the barge_in signal handles coordination).

**Whisper fallback.** When Porcupine is unavailable (`voice_porcupine::is_available()` returns false), the audio_chunk handler skips the Porcupine feed. The ALWAYS LISTEN button still works — in standby, silence_detected fires as it did before the Porcupine work, but without trigger word matching. Every utterance goes straight to the pipeline (no keyword gate). This is a simpler fallback than the full Whisper trigger matching — the user activates always-listen knowing that any speech triggers the pipeline. The Whisper `match_trigger` code is removed entirely.

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
mcp__workflow__load_work_item({ path: "work_items/VOICE_PORCUPINE_BARGE_IN.md" })
```

Then call `next_task` to begin.
