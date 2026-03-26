# Voice Pipeline — Trigger Word Activation

## Goal

Replace push-to-talk as the sole activation method with always-on standby listening that activates the LLM pipeline only when a configurable trigger word is detected. When the user says just the trigger word with no follow-up query, play an acknowledgment tone and start a fresh recording for the actual query (Option A UX).

## Why

Push-to-talk requires manual button clicks for every interaction. Trigger words enable hands-free voice interaction — the mic stays on in a standby loop, transcribing each silence-delimited utterance via Whisper, and only entering the full STT→LLM→TTS pipeline when the transcription starts with a trigger phrase. This matches the UX model of voice assistants (Alexa, Siri) while reusing the existing Whisper + VAD infrastructure with zero new dependencies.

## Architecture

### New state: Standby

The voice widget gains a `"standby"` status alongside the existing `"idle" | "listening" | "processing" | "speaking"`. In standby, the mic stays open and audio chunks flow continuously. The existing VAD silence detection fires `silence_detected` as before, but instead of stopping, the widget transitions to `"processing"` temporarily. Rust then either:

1. **No trigger match** → sends `resume_standby` → widget resets VAD and returns to `"standby"` (mic never stopped)
2. **Trigger match with query** → runs the full LLM→TTS pipeline → after playback, sends `resume_standby` → back to standby
3. **Trigger match, no query** → plays acknowledgment tone → starts fresh recording → waits for next `silence_detected` → runs pipeline on that utterance unconditionally (the `awaiting_query` flag)

### Standby listening loop

```
[Standby] mic on, VAD running
    │
    ▼ silence_detected
[Processing] Whisper transcribes utterance
    │
    ├─ no trigger match → resume_standby → [Standby]
    │
    ├─ trigger + query text → run pipeline → resume_standby → [Standby]
    │
    └─ trigger only (no query) → play ack tone → [AwaitingQuery]
                                                       │
                                                       ▼ silence_detected
                                                  run pipeline unconditionally
                                                       │
                                                       ▼
                                                  resume_standby → [Standby]
```

### Trigger matching

Case-insensitive prefix match against the configured trigger words list. After stripping the matched prefix, also strip leading punctuation and whitespace. Default trigger words: `["hey claude", "ok claude", "claude"]`.

Matching order: longest trigger first, to prevent `"claude"` from matching when the user said `"hey claude"` (which would leave `"hey"` as junk prefix on the query).

### Acknowledgment tone

A short synthesized sine-wave beep (440 Hz, 150ms, with 10ms fade-in/fade-out to avoid clicks). Generated programmatically as PCM → WAV at the TTS sample rate (24 kHz) and played through the existing rodio sink. No external audio file needed.

### UI changes

The VoiceBanner control bar adds:
- **ALWAYS LISTEN** toggle button alongside the existing PUSH TO TALK / STOP button
- When always-listen is active, the status shows `STANDBY` instead of `IDLE` after pipeline completes
- The PUSH TO TALK button still works as a manual override regardless of always-listen state
- A new `AWAITING_QUERY` pipeline stage displayed when the system heard a trigger word and is waiting for the follow-up

## Dioxus Signal API note

`Signal<T>` in Dioxus is `Copy` (interior mutability via generational box). Never pass `&mut Signal<T>`. Use `.set()`, `.write()`, `.read()`. VoiceContext fields are all `Signal<T>` and are captured by value in closures.

## Files affected

| File | Repo | Change |
|------|------|--------|
| `ts/widget-bridge/widgets/voice.ts` | dioxus-common | Add `"standby"` status, `resume_standby` command, keep mic alive in standby, `awaiting_query` command |
| `ts/audio-capture/src/capture.ts` | dioxus-common | Add `resetVad()` method that resets VAD without stopping the mic/worklet |
| `crates/lx-desktop/src/pages/agents/voice_context.rs` | lx | Add `Standby` to `VoiceStatus`, `AwaitingQuery` to `PipelineStage`, add `trigger_words` + `always_listen` + `awaiting_query` signals |
| `crates/lx-desktop/src/pages/agents/voice_banner.rs` | lx | Trigger word matching after transcription, ack tone generation/playback, `awaiting_query` flow, always-listen toggle UI, `resume_standby` instead of `stop_capture` |
| `crates/lx-desktop/src/voice_backend.rs` | lx | No changes |

## Task List

### Task 1: Add `resetVad()` to AudioCapture

In `dioxus-common/ts/audio-capture/src/capture.ts`, add a public `resetVad()` method to the `AudioCapture` class that calls `this.vad.reset()` without stopping the audio context, media stream, or worklet node. This allows the standby loop to restart silence detection after processing an utterance while keeping the mic hot.

```typescript
resetVad(): void {
  this.vad.reset();
}
```

Also export this from `dioxus-common/ts/audio-capture/src/index.ts` if not already re-exported (AudioCapture class should already be the main export).

### Task 2: Add standby mode to voice widget

In `dioxus-common/ts/widget-bridge/widgets/voice.ts`:

**Extend VoiceStatus type:**
```typescript
type VoiceStatus = "idle" | "standby" | "listening" | "processing" | "speaking";
```

**Add new commands in the `update` switch:**

- `"start_standby_listen"`: If idle, start capture, then transition to `"standby"` (not `"listening"`). The `onSilence` callback needs to fire in standby too.
- `"resume_standby"`: Reset VAD via `capture.resetVad()`, transition back to `"standby"`. Does NOT restart the mic (it's already running).
- `"awaiting_query"`: Transition to `"listening"` status (the mic is already on, VAD was just reset). Next `silence_detected` will fire normally.

**Modify `onSilence` callback** to also fire when `state.status === "standby"`:
```typescript
capture.onSilence = () => {
  if (state.status === "listening" || state.status === "standby") {
    transition(state, "processing");
    dx.send({ type: "silence_detected" });
  }
};
```

Key difference from `"listening"`: in standby, we do NOT call `capture.stop()` before transitioning to processing. The mic stays on. In `"listening"` mode (push-to-talk or awaiting_query), we also keep the mic on now — stopping/restarting is expensive. Instead, we just pause chunk delivery by checking `this.running` (which remains true).

**Modify `onChunk` callback** to also send chunks in standby:
```typescript
capture.onChunk = (b64pcm: string) => {
  if (state.status === "listening" || state.status === "standby") {
    dx.send({ type: "audio_chunk", data: b64pcm, seq: capture.currentSeq });
  }
};
```

**`stop_capture` still works as before** — full stop, transition to idle. This is the kill switch.

### Task 3: Build the widget-bridge JS bundle

Run the widget-bridge build so `crates/lx-desktop/assets/widget-bridge.js` reflects the TS changes from Tasks 1–2:

```bash
cd ~/repos/dioxus-common/ts/widget-bridge && pnpm build
cp dist/widget-bridge.js ~/repos/lx/crates/lx-desktop/assets/widget-bridge.js
```

### Task 4: Add standby state and trigger word signals to VoiceContext

In `crates/lx-desktop/src/pages/agents/voice_context.rs`:

**Add `Standby` variant to `VoiceStatus`:**
```rust
pub enum VoiceStatus {
  Idle,
  Standby,
  Listening,
  Processing,
  Speaking,
}
```
Update the `Display` impl to show `"STANDBY"`.

**Add `AwaitingQuery` variant to `PipelineStage`:**
```rust
pub enum PipelineStage {
  Idle,
  Transcribing,
  QueryingLlm,
  SynthesizingSpeech,
  AwaitingQuery,
}
```
Update the `Display` impl to show `"AWAITING_QUERY"`.

**Add new signals to `VoiceContext`:**
```rust
pub struct VoiceContext {
  // ... existing fields ...
  pub trigger_words: Signal<Vec<String>>,
  pub always_listen: Signal<bool>,
  pub awaiting_query: Signal<bool>,
}
```

Initialize in `provide()`:
```rust
trigger_words: Signal::new(vec![
  "hey claude".into(),
  "ok claude".into(),
  "claude".into(),
]),
always_listen: Signal::new(false),
awaiting_query: Signal::new(false),
```

### Task 5: Add trigger word matching and ack tone to voice_banner.rs

In `crates/lx-desktop/src/pages/agents/voice_banner.rs`:

**Add acknowledgment tone generator function:**
```rust
fn generate_ack_tone() -> Vec<u8> {
  let sample_rate = 24000u32;
  let duration_samples = (sample_rate as f64 * 0.15) as usize; // 150ms
  let fade_samples = (sample_rate as f64 * 0.01) as usize;     // 10ms fade
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
```

**Add trigger word matching function:**
```rust
fn match_trigger_word(text: &str, trigger_words: &[String]) -> Option<String> {
  let lower = text.to_lowercase();
  // Sort by length descending so "hey claude" matches before "claude"
  let mut sorted: Vec<&String> = trigger_words.iter().collect();
  sorted.sort_by(|a, b| b.len().cmp(&a.len()));
  for trigger in sorted {
    let trigger_lower = trigger.to_lowercase();
    if let Some(remainder) = lower.strip_prefix(&trigger_lower) {
      let query = remainder.trim_start_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace());
      return Some(query.to_owned());
    }
  }
  None
}
```

**Modify `silence_detected` handler** in the `use_future` message loop. When `silence_detected` arrives:

1. If `ctx.awaiting_query` is true — this is the follow-up utterance after a trigger-only activation. Run the pipeline unconditionally (no trigger check). Clear the `awaiting_query` flag.

2. If `ctx.always_listen` is true (standby mode) — transcribe, then:
   - No trigger match → send `resume_standby`, return early
   - Trigger + non-empty query → run pipeline with the query text, then send `resume_standby`
   - Trigger + empty query → play ack tone, set `awaiting_query = true`, send `awaiting_query` command to JS widget, return

3. If `ctx.always_listen` is false (push-to-talk mode) — existing behavior unchanged, run pipeline directly.

**Modify pipeline completion.** After `run_pipeline` finishes successfully and `ctx.always_listen` is true, send `resume_standby` to the voice widget instead of `stop_capture`.

**Add always-listen toggle to the UI** in the RSX control bar. Place an "ALWAYS LISTEN" / "STOP LISTENING" button next to the existing PUSH TO TALK / STOP button. When clicked:
- If enabling: send `start_standby_listen` to voice widget, set `ctx.always_listen` to true
- If disabling: send `stop_capture` to voice widget, set `ctx.always_listen` to false

**Handle `"standby"` in `status_change`:**
```rust
Some("standby") => ctx.status.set(VoiceStatus::Standby),
```

**Update `is_active`** to include `Standby`:
```rust
let is_active = current_status != VoiceStatus::Idle;
```
This already works since `Standby != Idle`.

### Task 6: Build the widget-bridge JS bundle (final)

Rebuild the widget-bridge bundle one final time to capture any iteration from Task 5 if JS was touched:

```bash
cd ~/repos/dioxus-common/ts/widget-bridge && pnpm build
cp dist/widget-bridge.js ~/repos/lx/crates/lx-desktop/assets/widget-bridge.js
```

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
