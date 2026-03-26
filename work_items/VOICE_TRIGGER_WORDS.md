# Voice Pipeline — Trigger Word Activation

## Goal

Add always-on standby listening that activates the LLM pipeline only when a configurable trigger word is detected. When the user says just the trigger word with no follow-up query, play an acknowledgment tone and record a fresh utterance for the query. Push-to-talk continues working as-is when always-listen is off.

## Why

Push-to-talk requires a button click for every interaction. Trigger words enable hands-free voice — the mic stays on in a standby loop, transcribing each silence-delimited utterance via Whisper, and only entering the LLM→TTS pipeline when the transcription starts with a trigger phrase.

## State machine

```
                  ┌─────────────────────────────────────────────────┐
                  │           silence_detected (no trigger match)   │
                  │           OR empty buffer                       │
                  ▼           resume_standby                        │
 [Idle] ──start_standby_listen──▶ [Standby] ──silence_detected──▶ [Processing]
   ▲                                  ▲                             │
   │ stop_capture                     │ resume_standby              │ trigger + query
   │                                  │                             ▼
   │                                  └──────────────────── run_pipeline
   │
   │                               trigger + empty query
   │                              [Processing] ──ack_tone──▶ [AwaitingQuery]
   │                                                              │
   │                                                              │ silence_detected
   │                                                              ▼
   │                                                     run_pipeline (unconditional)
   │                                                              │
   │                                                              │ resume_standby
   │                                                              ▼
   │                                                          [Standby]
   │
   │ ── Push-to-talk (separate flow, unchanged) ──
   │
 [Idle] ──start_capture──▶ [Listening] ──silence_detected──▶ [Processing]
   ▲                                                            │
   └───────────── stop_capture ◄── run_pipeline ◄──────────────┘
```

Two modes, mutually exclusive:
- **Always-listen ON**: standby → trigger word activation only. STOP exits to idle.
- **Always-listen OFF**: push-to-talk only. Unchanged from today.

No PTT-while-in-standby. Pressing STOP during standby stops everything and turns off always-listen. This avoids the complexity of tracking whether a silence_detected came from standby or PTT.

## Trigger matching: word-by-word with punctuation stripping

Whisper frequently inserts commas between words: `"Hey, Claude, what time is it?"`. A naive `strip_prefix("hey claude")` fails on `"Hey, Claude"` because of the comma. The matching function must compare word-by-word, stripping non-alphanumeric characters from each word before comparing:

```rust
fn match_trigger(text: &str, triggers: &[String]) -> Option<String> {
    let text_words: Vec<&str> = text.split_whitespace().collect();
    let mut sorted: Vec<&String> = triggers.iter().collect();
    sorted.sort_by(|a, b| b.len().cmp(&a.len()));
    for trigger in sorted {
        let trigger_words: Vec<&str> = trigger.split_whitespace().collect();
        let n = trigger_words.len();
        if text_words.len() >= n {
            let matches = text_words[..n].iter().zip(trigger_words.iter()).all(|(a, b)| {
                let a_clean: String = a.chars().filter(|c| c.is_alphanumeric()).collect();
                let b_clean: String = b.chars().filter(|c| c.is_alphanumeric()).collect();
                a_clean.eq_ignore_ascii_case(&b_clean)
            });
            if matches {
                return Some(text_words[n..].join(" "));
            }
        }
    }
    None
}
```

Examples:
- `"Hey, Claude, what's the weather?"` → trigger `"hey claude"` (2 words) → compare `"Hey," ≈ "hey"` ✓, `"Claude," ≈ "claude"` ✓ → remainder `"what's the weather?"`
- `"Claude what time"` → trigger `"hey claude"` fails, `"claude"` (1 word) matches → remainder `"what time"`
- `"Hey Claude"` → match → remainder `""` (empty → ack tone flow)

Sorted longest-first so `"hey claude"` matches before `"claude"`, preventing `"claude"` from consuming just the first word and leaving `"what's the weather?"` with a stale `"Hey,"` prefix.

Default trigger words: `["hey claude", "ok claude", "claude"]`.

## Preventing idle polling in standby

Without a guard, standby creates a 2-second polling loop: nobody speaks → VAD times out → `silence_detected` fires → empty buffer → `resume_standby` → 2s later → repeat. This is wasteful message churn.

Fix: add a `hasSpeech: boolean` flag to the JS voice widget state. Set `true` when `onChunk` sends a chunk. In `onSilence`, only fire `silence_detected` if `hasSpeech` is true (otherwise just reset VAD and stay in standby). Reset to `false` on `resume_standby` and `awaiting_query`.

## Preventing concurrent pipeline runs

In standby, the mic stays on during processing. A new `silence_detected` could fire while Whisper is still transcribing the previous utterance, spawning an overlapping pipeline. Guard: in the `silence_detected` handler, check `ctx.pipeline_stage() != PipelineStage::Idle` before starting work. If the pipeline is busy, discard the buffer and continue the message loop (the audio is lost, which is acceptable — the alternative is queuing utterances which adds complexity for no UX benefit).

## Awaiting-query timeout

If the user says "Hey Claude" and walks away, the system is stuck in `AwaitingQuery` forever, running the next ambient noise through the LLM unconditionally. Fix: after playing the ack tone and entering `AwaitingQuery`, spawn a timeout future:

```rust
spawn(async move {
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    if ctx.pipeline_stage() == PipelineStage::AwaitingQuery {
        ctx.pipeline_stage.set(PipelineStage::Idle);
        voice_widget.send_update(serde_json::json!({ "type": "resume_standby" }));
    }
});
```

If the user speaks before timeout, the pipeline stage changes away from `AwaitingQuery` and the timeout future becomes a no-op. Dioxus `spawn` futures run on the same async executor — no true race condition, one completes its synchronous block before the other executes.

## Ack tone: audio feedback from speakers to mic

The ack tone (150ms, 440Hz) plays through speakers while the mic is live. The mic picks up the beep. With the 2-second VAD silence timeout this cannot cause a false trigger — the beep is too short to keep the silence timer active past the 2s threshold. If the VAD silence timeout is ever shortened below ~500ms, this would need revisiting (e.g., muting chunk delivery during playback).

## 300-line file split

`voice_banner.rs` is currently 238 lines. Adding trigger matching, ack tone, awaiting_query flow, timeout, and the always-listen toggle will exceed 300. Split into:

- **`voice_banner.rs`** (~165 lines): `VoiceBanner` component, RSX, `use_future` message loop, always-listen toggle UI
- **`voice_pipeline.rs`** (~135 lines): `run_pipeline`, `transcribe`, `match_trigger`, `generate_ack_tone`, `play_wav`

Extract `tts()` and `run_pipeline()` (which already exist in voice_banner.rs) plus the new functions into `voice_pipeline.rs`. The `AUDIO_SINK` lazy static also moves there since only pipeline code uses it.

## Build system

`crates/lx-desktop/build.rs` already watches `ts/audio-capture/src/*.ts` and `ts/widget-bridge/{src,widgets}/*.ts` for changes, runs `pnpm build` in the widget-bridge directory, and copies the output to `crates/lx-desktop/assets/widget-bridge.js`. No manual JS build step is needed — editing the TS source is sufficient. The pnpm workspace at `dioxus-common/pnpm-workspace.yaml` includes `ts/*`, so `@dioxus-common/audio-capture` resolves as a workspace dependency of widget-bridge.

## Files affected

| File | Repo | Change |
|------|------|--------|
| `ts/audio-capture/src/capture.ts` | dioxus-common | Add `resetVad()` public method |
| `ts/widget-bridge/widgets/voice.ts` | dioxus-common | Add `"standby"` status, `hasSpeech` flag, three new commands, modified callbacks |
| `crates/lx-desktop/src/pages/agents/voice_context.rs` | lx | Add `Standby` to `VoiceStatus`, `AwaitingQuery` to `PipelineStage`, add `trigger_words` + `always_listen` signals |
| `crates/lx-desktop/src/pages/agents/voice_pipeline.rs` | lx | New file: extracted `run_pipeline` + `tts` + `AUDIO_SINK` from voice_banner, plus `transcribe`, `match_trigger`, `generate_ack_tone`, `play_wav` |
| `crates/lx-desktop/src/pages/agents/voice_banner.rs` | lx | Remove pipeline functions (moved to voice_pipeline), add always-listen toggle UI, rewrite silence_detected handler with trigger/standby/awaiting_query logic, add concurrent pipeline guard |
| `crates/lx-desktop/src/pages/agents/mod.rs` | lx | Add `mod voice_pipeline;` |

## Task List

### Task 1: Add `resetVad()` to AudioCapture

In `/home/entropybender/repos/dioxus-common/ts/audio-capture/src/capture.ts`, add this public method to the `AudioCapture` class, after the existing `dispose()` method:

```typescript
resetVad(): void {
    this.vad.reset();
}
```

`index.ts` already re-exports the `AudioCapture` class — no changes needed there (verified: `export { AudioCapture, ... } from "./capture"`).

### Task 2: Add standby mode to voice widget

Rewrite `/home/entropybender/repos/dioxus-common/ts/widget-bridge/widgets/voice.ts` to this exact content:

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
  hasSpeech: boolean;
}

const states = new Map<string, VoiceState>();

function transition(state: VoiceState, status: VoiceStatus): void {
  state.status = status;
  state.dx.send({ type: "status_change", status });
}

const voiceWidget: Widget = {
  mount(elementId: string, _config: unknown, dx: Dioxus) {
    const capture = new AudioCapture({ sampleRate: 16000 });

    const state: VoiceState = { capture, status: "idle", dx, hasSpeech: false };
    states.set(elementId, state);

    capture.onChunk = (b64pcm: string) => {
      if (state.status === "listening" || state.status === "standby") {
        state.hasSpeech = true;
        dx.send({ type: "audio_chunk", data: b64pcm, seq: capture.currentSeq });
      }
    };

    capture.onSilence = () => {
      if (state.status === "standby") {
        if (!state.hasSpeech) {
          capture.resetVad();
          return;
        }
        state.hasSpeech = false;
        transition(state, "processing");
        dx.send({ type: "silence_detected" });
      } else if (state.status === "listening") {
        state.hasSpeech = false;
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

    const msg = data as { type: string; data?: string; id?: string };

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
        state.hasSpeech = false;
        state.capture.start().then(() => {
          transition(state, "standby");
        });
        break;
      case "resume_standby":
        state.hasSpeech = false;
        state.capture.resetVad();
        transition(state, "standby");
        break;
      case "awaiting_query":
        state.hasSpeech = false;
        state.capture.resetVad();
        transition(state, "listening");
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

Key behavioral differences from the current file:

**`onSilence` is split by status.** In `"standby"`, the mic stays on (no `capture.stop()`), `hasSpeech` prevents idle polling, and the guard `if (!state.hasSpeech)` just resets VAD and returns without firing `silence_detected`. In `"listening"` (push-to-talk), the mic stops as before — push-to-talk behavior is unchanged.

**`onChunk` sets `state.hasSpeech = true`** so that `onSilence` knows whether any real audio was captured since the last reset.

**`start_standby_listen`** starts capture and transitions to `"standby"` instead of `"listening"`. The `hasSpeech` flag is cleared.

**`resume_standby`** resets VAD and clears `hasSpeech` without touching the mic (already running). Transitions from `"processing"` back to `"standby"`.

**`awaiting_query`** resets VAD, clears `hasSpeech`, and transitions to `"listening"`. The next `onSilence` will fire with `capture.stop()` (listening path). After the pipeline runs, Rust sends `resume_standby` to return to standby.

**`onRms` fires unconditionally** (as before) — the volume meter works in all states.

### Task 3: Add standby state and trigger word signals to VoiceContext

Replace the contents of `crates/lx-desktop/src/pages/agents/voice_context.rs` with:

```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum VoiceStatus {
    Idle,
    Standby,
    Listening,
    Processing,
    Speaking,
}

impl std::fmt::Display for VoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "IDLE"),
            Self::Standby => write!(f, "STANDBY"),
            Self::Listening => write!(f, "LISTENING"),
            Self::Processing => write!(f, "PROCESSING"),
            Self::Speaking => write!(f, "SPEAKING"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PipelineStage {
    Idle,
    Transcribing,
    QueryingLlm,
    SynthesizingSpeech,
    AwaitingQuery,
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, ""),
            Self::Transcribing => write!(f, "TRANSCRIBING"),
            Self::QueryingLlm => write!(f, "QUERYING_LLM"),
            Self::SynthesizingSpeech => write!(f, "SYNTHESIZING_SPEECH"),
            Self::AwaitingQuery => write!(f, "AWAITING_QUERY"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct TranscriptEntry {
    pub is_user: bool,
    pub text: String,
}

#[derive(Clone, Copy)]
pub struct VoiceContext {
    pub status: Signal<VoiceStatus>,
    pub transcript: Signal<Vec<TranscriptEntry>>,
    pub pcm_buffer: Signal<Vec<u8>>,
    pub rms: Signal<f32>,
    pub pipeline_stage: Signal<PipelineStage>,
    pub widget: Signal<Option<dioxus_widget_bridge::TsWidgetHandle>>,
    pub trigger_words: Signal<Vec<String>>,
    pub always_listen: Signal<bool>,
}

impl VoiceContext {
    pub fn provide() -> Self {
        let ctx = Self {
            status: Signal::new(VoiceStatus::Idle),
            transcript: Signal::new(Vec::new()),
            pcm_buffer: Signal::new(Vec::new()),
            rms: Signal::new(0.0),
            pipeline_stage: Signal::new(PipelineStage::Idle),
            widget: Signal::new(None),
            trigger_words: Signal::new(vec![
                "hey claude".into(),
                "ok claude".into(),
                "claude".into(),
            ]),
            always_listen: Signal::new(false),
        };
        use_context_provider(|| ctx);
        ctx
    }
}
```

No `awaiting_query: Signal<bool>` — tracked solely via `PipelineStage::AwaitingQuery` to avoid dual sources of truth.

### Task 4: Create voice_pipeline.rs

Create `crates/lx-desktop/src/pages/agents/voice_pipeline.rs` with the functions extracted from `voice_banner.rs` plus the new trigger/ack functions.

Add `mod voice_pipeline;` to `crates/lx-desktop/src/pages/agents/mod.rs` alongside the existing `mod voice_banner;`.

The new file contains:

```rust
use std::sync::LazyLock;

use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use common_inference::InferenceClient as _;
use common_kokoro::SpeechRequest;
use common_whisper::TranscribeRequest;
use dioxus::logger::tracing::{error, info};

pub static AUDIO_SINK: LazyLock<rodio::MixerDeviceSink> = LazyLock::new(|| {
    let mut sink = rodio::DeviceSinkBuilder::open_default_sink()
        .unwrap_or_else(|e| panic!("audio device: {e}"));
    sink.log_on_drop(false);
    sink
});

pub async fn transcribe(pcm: &[u8]) -> anyhow::Result<String> {
    let wav = common_audio::wrap_pcm_as_wav(
        pcm,
        common_audio::SAMPLE_RATE,
        common_audio::CHANNELS,
        common_audio::BITS_PER_SAMPLE,
    );
    let transcription = common_whisper::WHISPER
        .infer(&TranscribeRequest {
            audio_data: B64.encode(&wav),
            language: Some("en".into()),
        })
        .await?;
    Ok(transcription.text.trim().to_owned())
}

pub fn match_trigger(text: &str, triggers: &[String]) -> Option<String> {
    let text_words: Vec<&str> = text.split_whitespace().collect();
    let mut sorted: Vec<&String> = triggers.iter().collect();
    sorted.sort_by(|a, b| b.len().cmp(&a.len()));
    for trigger in sorted {
        let trigger_words: Vec<&str> = trigger.split_whitespace().collect();
        let n = trigger_words.len();
        if text_words.len() >= n {
            let matches = text_words[..n]
                .iter()
                .zip(trigger_words.iter())
                .all(|(a, b)| {
                    let a_clean: String = a.chars().filter(|c| c.is_alphanumeric()).collect();
                    let b_clean: String = b.chars().filter(|c| c.is_alphanumeric()).collect();
                    a_clean.eq_ignore_ascii_case(&b_clean)
                });
            if matches {
                return Some(text_words[n..].join(" "));
            }
        }
    }
    None
}

pub fn generate_ack_tone() -> Vec<u8> {
    let sample_rate = 24000u32;
    let duration_samples = (sample_rate as f64 * 0.15) as usize;
    let fade_samples = (sample_rate as f64 * 0.01) as usize;
    let freq = 440.0f64;
    let mut pcm = Vec::with_capacity(duration_samples * 2);
    for i in 0..duration_samples {
        let t = i as f64 / sample_rate as f64;
        let mut sample = (t * freq * 2.0 * std::f64::consts::PI).sin() * 0.3;
        if i < fade_samples {
            sample *= i as f64 / fade_samples as f64;
        } else if i >= duration_samples - fade_samples {
            sample *= (duration_samples - 1 - i) as f64 / fade_samples as f64;
        }
        let s = (sample * 32767.0) as i16;
        pcm.extend_from_slice(&s.to_le_bytes());
    }
    common_audio::wrap_pcm_as_wav(&pcm, sample_rate, 1, 16)
}

pub fn play_wav(wav_bytes: Vec<u8>) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::task::spawn_blocking(move || {
        let cursor = std::io::Cursor::new(wav_bytes);
        let player = rodio::play(AUDIO_SINK.mixer(), cursor).map_err(|e| {
            error!("voice: rodio::play failed: {e}");
            e
        })?;
        player.sleep_until_end();
        Ok(())
    })
}

async fn tts(text: &str) -> anyhow::Result<Vec<u8>> {
    let req = SpeechRequest {
        text: text.to_owned(),
        voice: "am_michael".into(),
        lang_code: "a".into(),
        speed: 1.2,
    };
    common_kokoro::KOKORO.infer(&req).await
}

pub async fn run_pipeline(
    text: &str,
    voice_widget: dioxus_widget_bridge::TsWidgetHandle,
    agent_widget: dioxus_widget_bridge::TsWidgetHandle,
    mut ctx: VoiceContext,
) -> anyhow::Result<()> {
    ctx.transcript
        .write()
        .push(TranscriptEntry { is_user: true, text: text.to_owned() });

    agent_widget.send_update(serde_json::json!({ "type": "user_display", "text": text }));

    ctx.pipeline_stage.set(PipelineStage::QueryingLlm);
    let response = crate::voice_backend::query_streaming(text, |chunk| {
        agent_widget
            .send_update(serde_json::json!({ "type": "assistant_chunk", "text": chunk }));
    })
    .await?;
    agent_widget.send_update(serde_json::json!({ "type": "assistant_done" }));

    if response.is_empty() {
        ctx.pipeline_stage.set(PipelineStage::Idle);
        return Ok(());
    }

    ctx.pipeline_stage.set(PipelineStage::SynthesizingSpeech);
    let wav_bytes = tts(&response).await?;

    ctx.status.set(VoiceStatus::Speaking);

    let wav_len = wav_bytes.len();
    info!("voice: TTS returned {wav_len} bytes, starting playback");
    let play_result = play_wav(wav_bytes).await;
    match &play_result {
        Ok(Ok(())) => info!("voice: playback finished"),
        Ok(Err(e)) => error!("voice: playback error: {e}"),
        Err(e) => error!("voice: spawn_blocking panicked: {e}"),
    }
    play_result??;

    let transcript_entry = response;
    let mut t = ctx.transcript.write();
    match t.last_mut() {
        Some(entry) if !entry.is_user => entry.text.push_str(&format!(" {transcript_entry}")),
        _ => t.push(TranscriptEntry { is_user: false, text: transcript_entry }),
    }

    ctx.pipeline_stage.set(PipelineStage::Idle);
    Ok(())
}
```

Key changes from the original `run_pipeline`:
- **Takes `text: &str` instead of `pcm: Vec<u8>`** — transcription is pulled out to a separate `transcribe()` function called by the message loop before trigger matching.
- **`play_wav` is extracted** as a reusable function — used by both `run_pipeline` (TTS playback) and the ack tone flow in voice_banner.

### Task 5: Rewrite voice_banner.rs with standby/trigger logic

Replace the contents of `crates/lx-desktop/src/pages/agents/voice_banner.rs` with:

```rust
use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
use super::voice_pipeline;
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
                        ctx.pcm_buffer.write().extend_from_slice(&bytes);
                    }
                }
                Some("silence_detected") => {
                    if ctx.pipeline_stage() != PipelineStage::Idle
                        && ctx.pipeline_stage() != PipelineStage::AwaitingQuery
                    {
                        ctx.pcm_buffer.write().clear();
                        continue;
                    }

                    let buffer = std::mem::take(&mut *ctx.pcm_buffer.write());
                    if buffer.is_empty() {
                        if (ctx.always_listen)() {
                            voice_widget.send_update(
                                serde_json::json!({ "type": "resume_standby" }),
                            );
                        } else {
                            voice_widget
                                .send_update(serde_json::json!({ "type": "stop_capture" }));
                        }
                        continue;
                    }

                    let is_awaiting = ctx.pipeline_stage() == PipelineStage::AwaitingQuery;
                    let is_standby = (ctx.always_listen)() && !is_awaiting;

                    spawn(async move {
                        let result = handle_utterance(
                            buffer,
                            is_awaiting,
                            is_standby,
                            voice_widget,
                            agent_widget,
                            ctx,
                        )
                        .await;
                        if let Err(e) = result {
                            error!("voice: pipeline error: {e}");
                            ctx.transcript.write().push(TranscriptEntry {
                                is_user: false,
                                text: format!("Error: {e}"),
                            });
                            ctx.pipeline_stage.set(PipelineStage::Idle);
                        }
                        if (ctx.always_listen)() {
                            voice_widget
                                .send_update(serde_json::json!({ "type": "resume_standby" }));
                        } else {
                            voice_widget
                                .send_update(serde_json::json!({ "type": "stop_capture" }));
                        }
                    });
                }
                Some("rms") => {
                    if let Some(level) = msg["level"].as_f64() {
                        ctx.rms.set(level as f32);
                    }
                }
                Some("status_change") => match msg["status"].as_str() {
                    Some("idle") => ctx.status.set(VoiceStatus::Idle),
                    Some("standby") => ctx.status.set(VoiceStatus::Standby),
                    Some("listening") => ctx.status.set(VoiceStatus::Listening),
                    Some("processing") => ctx.status.set(VoiceStatus::Processing),
                    Some("speaking") => ctx.status.set(VoiceStatus::Speaking),
                    _ => {}
                },
                Some("start_standby") | Some("cancel") => ctx.pcm_buffer.write().clear(),
                _ => {}
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
                        agent_widget
                            .send_update(serde_json::json!({ "type": "assistant_done" }));
                    }
                    Err(e) => {
                        agent_widget.send_update(
                            serde_json::json!({ "type": "error", "message": format!("{e:#}") }),
                        );
                    }
                }
            }
        }
    });

    let current_status = (ctx.status)();
    let is_active = current_status != VoiceStatus::Idle;
    let status_text = current_status.to_string();
    let bar_glow = if is_active {
        "shadow-[0_0_12px_var(--primary)]"
    } else {
        ""
    };
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

async fn handle_utterance(
    pcm: Vec<u8>,
    is_awaiting: bool,
    is_standby: bool,
    voice_widget: dioxus_widget_bridge::TsWidgetHandle,
    agent_widget: dioxus_widget_bridge::TsWidgetHandle,
    mut ctx: VoiceContext,
) -> anyhow::Result<()> {
    ctx.pipeline_stage.set(PipelineStage::Transcribing);
    let text = voice_pipeline::transcribe(&pcm).await?;
    if text.is_empty() {
        ctx.pipeline_stage.set(PipelineStage::Idle);
        return Ok(());
    }

    if is_awaiting {
        voice_pipeline::run_pipeline(&text, voice_widget, agent_widget, ctx).await?;
        return Ok(());
    }

    if is_standby {
        let triggers = ctx.trigger_words.read().clone();
        match voice_pipeline::match_trigger(&text, &triggers) {
            None => {
                ctx.pipeline_stage.set(PipelineStage::Idle);
            }
            Some(query) if query.is_empty() => {
                ctx.pipeline_stage.set(PipelineStage::AwaitingQuery);
                let ack_wav = voice_pipeline::generate_ack_tone();
                let _ = voice_pipeline::play_wav(ack_wav).await;
                voice_widget
                    .send_update(serde_json::json!({ "type": "awaiting_query" }));
                spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    if ctx.pipeline_stage() == PipelineStage::AwaitingQuery {
                        ctx.pipeline_stage.set(PipelineStage::Idle);
                        voice_widget.send_update(
                            serde_json::json!({ "type": "resume_standby" }),
                        );
                    }
                });
                return Ok(());
            }
            Some(query) => {
                voice_pipeline::run_pipeline(&query, voice_widget, agent_widget, ctx)
                    .await?;
            }
        }
        return Ok(());
    }

    voice_pipeline::run_pipeline(&text, voice_widget, agent_widget, ctx).await?;
    Ok(())
}
```

Key design decisions:

**Concurrent pipeline guard:** The `silence_detected` handler checks `pipeline_stage != Idle && pipeline_stage != AwaitingQuery` — if the pipeline is busy (Transcribing, QueryingLlm, SynthesizingSpeech), the buffer is discarded. `AwaitingQuery` is allowed through because it's waiting for exactly this event.

**`handle_utterance` is the branching point.** It transcribes first, then branches:
- `is_awaiting` → run pipeline unconditionally (user already said trigger word)
- `is_standby` → check trigger match → no match (discard), empty query (ack tone + timeout), or has query (run pipeline)
- Neither (push-to-talk) → run pipeline unconditionally

**Post-pipeline resume.** The `spawn` block in `silence_detected` always sends either `resume_standby` or `stop_capture` after `handle_utterance` returns, based on `always_listen`. Exception: the ack-tone path returns early from `handle_utterance` without reaching the post-pipeline code, because the JS widget transitions to `"listening"` via `awaiting_query` and handles its own silence_detected → which re-enters this handler with `is_awaiting = true`.

**Awaiting-query early return.** When the ack tone path fires, `handle_utterance` returns `Ok(())`. The caller's `spawn` block then sends `resume_standby` (since `always_listen` is true). But by this point, JS already received `awaiting_query` and is in `"listening"` state. The `resume_standby` that follows transitions JS to `"standby"`. This is a problem — it would cancel the awaiting-query flow.

Fix: the `awaiting_query` ack tone path must NOT fall through to the caller's resume/stop logic. The `handle_utterance` returns early, and the caller must detect this. Use a return type: `handle_utterance` returns `Ok(true)` for "pipeline ran, do resume/stop" and `Ok(false)` for "handled internally, don't send anything."

**Correction to the code above:** Change `handle_utterance` return type to `anyhow::Result<bool>`:
- Return `Ok(true)` at the end of paths that run the pipeline (awaiting, standby+query, push-to-talk)
- Return `Ok(false)` in the standby no-match path and the ack-tone path

In the caller:
```rust
let result = handle_utterance(...).await;
match result {
    Ok(true) | Err(_) => {
        // pipeline ran or errored — resume/stop
        if (ctx.always_listen)() {
            voice_widget.send_update(serde_json::json!({ "type": "resume_standby" }));
        } else {
            voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
        }
    }
    Ok(false) => {} // handled internally (ack tone or no-match)
}
```

And in `handle_utterance`:
- No trigger match: send `resume_standby` directly, return `Ok(false)`
- Ack tone: send `awaiting_query` + spawn timeout, return `Ok(false)`
- Pipeline ran: return `Ok(true)`

**Standby icon.** Green circle `\u{1F7E2}` for standby, red `\u{1F534}` for active (listening/processing/speaking), lock `\u{1F512}` for idle. Provides visual distinction between "passively listening for trigger" and "actively processing."

**ALWAYS LISTEN button.** Toggle button in the control bar. When ON, sends `start_standby_listen` and sets `ctx.always_listen` to true. When OFF (or STOP pressed), sends `stop_capture` and sets `ctx.always_listen` to false. Both buttons (`ALWAYS LISTEN` toggle and `STOP`) set `always_listen` to false and send `stop_capture` when stopping.

**PUSH TO TALK still works** when always-listen is off. It sends `start_capture` as before. The `is_active` guard hides it when any mode is active.

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
mcp__workflow__load_work_item({ path: "work_items/VOICE_TRIGGER_WORDS.md" })
```

Then call `next_task` to begin.
