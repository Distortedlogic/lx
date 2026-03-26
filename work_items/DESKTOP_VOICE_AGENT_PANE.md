# Goal

Replace the PTY-based voice terminal approach with an Agent pane driven by the existing `agent.ts` chat widget. The voice pipeline calls `ClaudeCliBackend.query()` for the response, sends display messages to the agent widget for rendering, and feeds the response to TTS. No PTY, no shell escaping, no sentinel parsing.

# Why

The PTY terminal approach required shell escaping, sentinel parsing, stty echo suppression, and produced raw monospace text output with formatting artifacts. The `agent.ts` widget already has a purpose-built chat UI with user/assistant message bubbles, auto-scrolling, and streaming support. `ClaudeCliBackend.query()` already captures claude's response as clean text via `--output-format text`. Using the Agent widget for display and the existing backend for execution eliminates all the PTY complexity.

# Architecture

VoiceBanner creates two widgets: the existing headless voice widget (audio capture/playback) and a new agent widget (conversation display). The agent widget mounts into a visible div in VoiceBanner's RSX. The voice pipeline receives both widget handles. On each voice turn:

1. Transcribe audio → text (unchanged)
2. Send `{ type: "user_display", text }` to agent widget — renders a user message bubble
3. Call `ClaudeCliBackend.query(&text)` — returns clean response string
4. Send `{ type: "assistant_chunk", text: response }` + `{ type: "assistant_done" }` to agent widget — renders assistant bubble
5. Feed response to TTS (unchanged)

The `user_display` message type is new in agent.ts. It renders a user bubble from an external source (the voice pipeline) instead of from the widget's built-in textarea. The existing `assistant_chunk` and `assistant_done` types are unchanged.

`ClaudeCliBackend.query()` already handles `--output-format text`, `--system-prompt`, `--session-id`/`--resume`, and `SESSION_CREATED` tracking internally. The voice pipeline does not access these directly.

# Files Affected

| File | Change |
|------|--------|
| `dioxus-common/ts/widget-bridge/widgets/agent.ts` | Add `user_display` message type |
| `lx/crates/lx-desktop/src/pages/agents/voice_banner.rs` | Add agent widget, rewrite run_pipeline |
| `lx/crates/lx-desktop/src/pages/agents/voice_context.rs` | Remove voice_turn_count |
| `lx/crates/lx-desktop/src/voice_backend.rs` | Revert SYSTEM_PROMPT and SESSION_CREATED to private |
| `lx/crates/lx-desktop/src/pages/agents/mod.rs` | Give VoiceBanner flex space for conversation |

# Task List

### Task 1: Add user_display message type to agent.ts

**Subject:** Handle externally-sourced user messages in the agent chat widget

**Description:** Edit `/home/entropybender/repos/dioxus-common/ts/widget-bridge/widgets/agent.ts`. In the `update` method's switch statement (line 158), add a new case after the `assistant_done` case (after line 172):

```typescript
      case "user_display": {
        const bubble = createBubble(state.messagesDiv, "user");
        bubble.textContent = msg.text ?? "";
        autoScroll(state);
        break;
      }
```

This renders a user message bubble identical to what `sendMessage()` creates (line 128-129), but without sending a `user_message` event back to Rust. The voice pipeline sends the text; the widget just displays it.

**ActiveForm:** Adding user_display message type to agent.ts

---

### Task 2: Rewrite VoiceBanner with agent widget and new run_pipeline

**Subject:** Replace PTY/terminal approach with agent widget display and ClaudeCliBackend

**Description:** Rewrite `crates/lx-desktop/src/pages/agents/voice_banner.rs`. This is a full file rewrite.

**Imports:** Replace the current imports (lines 1-12) with:

```rust
use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use common_inference::InferenceClient as _;
use common_kokoro::SpeechRequest;
use common_voice::AgentBackend as _;
use common_whisper::TranscribeRequest;
use dioxus::prelude::*;
use dioxus_widget_bridge::use_ts_widget;
```

Removed: `crate::layout::shell::TerminalSpawnRequest`, `crate::panes::DesktopPane`, `common_pane_tree::PaneNode`, `tokio::sync::mpsc`.
Added back: `common_voice::AgentBackend as _`.

**VoiceBanner component:** Replace the entire component (lines 14-149). The new component creates TWO widgets:

```rust
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
      let Ok(msg) = voice_widget.recv::<serde_json::Value>().await else { break };
      match msg["type"].as_str() {
        Some("audio_chunk") => {
          if let Some(data) = msg["data"].as_str()
            && let Ok(bytes) = B64.decode(data)
          {
            ctx.pcm_buffer.write().extend_from_slice(&bytes);
          }
        },
        Some("silence_detected") => {
          let buffer = std::mem::take(&mut *ctx.pcm_buffer.write());
          if buffer.is_empty() {
            voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            continue;
          }
          spawn(async move {
            if let Err(e) = run_pipeline(buffer, voice_widget, agent_widget, ctx).await {
              ctx.transcript.write().push(TranscriptEntry { is_user: false, text: format!("Error: {e}") });
              ctx.pipeline_stage.set(PipelineStage::Idle);
              voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
            }
          });
        },
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
        Some("rms") => {
          if let Some(level) = msg["level"].as_f64() {
            ctx.rms.set(level as f32);
          }
        },
        Some("status_change") => match msg["status"].as_str() {
          Some("idle") => ctx.status.set(VoiceStatus::Idle),
          Some("listening") => ctx.status.set(VoiceStatus::Listening),
          Some("processing") => ctx.status.set(VoiceStatus::Processing),
          Some("speaking") => ctx.status.set(VoiceStatus::Speaking),
          _ => {},
        },
        Some("start_standby") | Some("cancel") => ctx.pcm_buffer.write().clear(),
        Some("playback_complete") => {},
        _ => {},
      }
    }
  });

  use_future(move || async move {
    loop {
      let Ok(msg) = agent_widget.recv::<serde_json::Value>().await else { break };
      match msg["type"].as_str() {
        Some("user_message") => {
          let content = msg["content"].as_str().unwrap_or("").to_owned();
          if content.is_empty() { continue; }
          match crate::voice_backend::ClaudeCliBackend.query(&content).await {
            Ok(response) => {
              agent_widget.send_update(serde_json::json!({ "type": "assistant_chunk", "text": response }));
              agent_widget.send_update(serde_json::json!({ "type": "assistant_done" }));
            }
            Err(e) => {
              agent_widget.send_update(serde_json::json!({ "type": "error", "message": format!("{e:#}") }));
            }
          }
        }
        _ => {}
      }
    }
  });

  let current_status = (ctx.status)();
  let is_active = current_status != VoiceStatus::Idle;
  let status_text = current_status.to_string();
  let bar_glow = if is_active { "shadow-[0_0_12px_var(--primary)]" } else { "" };
  let icon = if is_active { "\u{1F534}" } else { "\u{1F512}" };
  let volume = ((ctx.rms)() / 0.3).min(1.0);
  let stage = (ctx.pipeline_stage)();
  let entries = ctx.transcript.read();
  let turn_count = entries.iter().filter(|e| e.is_user).count();
  drop(entries);

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "bg-[var(--surface-container)] px-4 py-2 flex items-center gap-3 shrink-0 {bar_glow}",
        span { class: "text-[var(--primary)] text-sm", "{icon}" }
        span { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]", "{status_text}" }
        if is_active {
          div { class: "flex items-end gap-[2px] h-4 ml-1",
            span { class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75", style: "height: {(volume * 40.0).max(2.0)}%;" }
            span { class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75", style: "height: {(volume * 70.0).max(2.0)}%;" }
            span { class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75", style: "height: {(volume * 100.0).max(2.0)}%;" }
            span { class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75", style: "height: {(volume * 60.0).max(2.0)}%;" }
          }
          span { class: "text-[10px] text-[var(--outline)] uppercase tracking-wider", "{stage}" }
          span { class: "text-[10px] text-[var(--outline)] uppercase tracking-wider", "TURNS: {turn_count}" }
        }
        div { class: "flex-1" }
        if is_active {
          button {
            class: "border border-[var(--outline)] text-[var(--on-surface)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--surface-container-high)] transition-colors duration-150 font-semibold",
            onclick: move |_| { voice_widget.send_update(serde_json::json!({ "type": "stop_capture" })); },
            "STOP"
          }
        } else {
          button {
            class: "border border-[var(--primary)] text-[var(--primary)] rounded px-4 py-1.5 text-sm uppercase hover:bg-[var(--primary)]/10 transition-colors duration-150 font-semibold",
            onclick: move |_| { voice_widget.send_update(serde_json::json!({ "type": "start_capture" })); },
            "PUSH TO TALK"
          }
        }
      }
      div { id: "{agent_element_id}", class: "flex-1 min-h-0 overflow-hidden" }
      div { id: "{voice_element_id}", class: "hidden" }
    }
  }
}
```

The RSX has three sections: the voice control bar (shrink-0), the agent conversation area (flex-1, visible), and the hidden voice widget mount point.

**Delete `shell_escape` function** (lines 176-188). It is no longer used.

**Rewrite `run_pipeline`** (lines 190-280). Replace the entire function:

```rust
async fn run_pipeline(
  pcm: Vec<u8>,
  voice_widget: dioxus_widget_bridge::TsWidgetHandle,
  agent_widget: dioxus_widget_bridge::TsWidgetHandle,
  mut ctx: VoiceContext,
) -> anyhow::Result<()> {
  ctx.pipeline_stage.set(PipelineStage::Transcribing);
  let wav = common_audio::wrap_pcm_as_wav(&pcm, common_audio::SAMPLE_RATE, common_audio::CHANNELS, common_audio::BITS_PER_SAMPLE);
  let transcription = common_whisper::WHISPER.infer(&TranscribeRequest { audio_data: B64.encode(&wav), language: Some("en".into()) }).await?;
  let text = transcription.text.trim().to_owned();
  if text.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    voice_widget.send_update(serde_json::json!({ "type": "stop_capture" }));
    return Ok(());
  }
  ctx.transcript.write().push(TranscriptEntry { is_user: true, text: text.clone() });

  agent_widget.send_update(serde_json::json!({ "type": "user_display", "text": text }));

  ctx.pipeline_stage.set(PipelineStage::QueryingLlm);
  let response = crate::voice_backend::ClaudeCliBackend.query(&text).await?;

  agent_widget.send_update(serde_json::json!({ "type": "assistant_chunk", "text": response }));
  agent_widget.send_update(serde_json::json!({ "type": "assistant_done" }));

  if response.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    return Ok(());
  }

  let sentences = split_sentences(&response);
  if sentences.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    return Ok(());
  }

  ctx.pipeline_stage.set(PipelineStage::SynthesizingSpeech);
  let mut next_tts: Option<tokio::task::JoinHandle<anyhow::Result<Vec<u8>>>> = {
    let s = sentences[0].clone();
    Some(tokio::spawn(async move { tts(&s).await }))
  };

  for (i, sentence) in sentences.iter().enumerate() {
    let Some(handle) = next_tts.take() else { break };
    let wav_bytes = handle.await??;

    if i + 1 < sentences.len() {
      let s = sentences[i + 1].clone();
      next_tts = Some(tokio::spawn(async move { tts(&s).await }));
    }

    let id = format!("s{i}");
    ctx.pending.write().insert(id.clone(), sentence.clone());
    voice_widget.send_update(serde_json::json!({
        "type": "audio_response",
        "data": B64.encode(&wav_bytes),
        "id": id,
    }));
  }
  ctx.pipeline_stage.set(PipelineStage::Idle);
  Ok(())
}
```

Keep `split_sentences` and `tts` functions unchanged.

**ActiveForm:** Rewriting VoiceBanner with agent widget display

---

### Task 3: Remove voice_turn_count from VoiceContext

**Subject:** Clean up the per-turn counter that is no longer used

**Description:** Edit `crates/lx-desktop/src/pages/agents/voice_context.rs`.

Remove the `voice_turn_count: Signal<u32>` field from the `VoiceContext` struct (line 57).

Remove the `voice_turn_count: Signal::new(0)` line from the `provide()` method (line 70).

**ActiveForm:** Removing voice_turn_count from VoiceContext

---

### Task 4: Revert voice_backend.rs visibility to private

**Subject:** SYSTEM_PROMPT and SESSION_CREATED no longer need to be pub

**Description:** Edit `crates/lx-desktop/src/voice_backend.rs`.

Change line 5 from `pub const SYSTEM_PROMPT` to `const SYSTEM_PROMPT`.
Change line 12 from `pub static SESSION_ID` to `static SESSION_ID`.
Change line 13 from `pub static SESSION_CREATED` to `static SESSION_CREATED`.

All three are only referenced inside `ClaudeCliBackend::query()` in the same file. No external code accesses them. `ClaudeCliBackend` itself stays `pub` — it's used by `terminal/view.rs`.

**ActiveForm:** Reverting voice_backend.rs visibility to private

---

### Task 5: Update Agents page layout for conversation area

**Subject:** Give VoiceBanner flex space so the conversation area fills available height

**Description:** Edit `crates/lx-desktop/src/pages/agents/mod.rs`. Change the VoiceBanner container from `shrink-0` to `flex-1 min-h-0`:

Replace:
```rust
div { class: "shrink-0 border-b border-[var(--outline-variant)]/15", VoiceBanner {} }
```

With:
```rust
div { class: "flex-1 min-h-0 border-b border-[var(--outline-variant)]/15", VoiceBanner {} }
```

Both VoiceBanner and PaneArea now have `flex-1 min-h-0`, splitting the page height evenly. VoiceBanner contains the voice control bar (shrink-0) and the agent conversation (flex-1), so the conversation fills the top half. PaneArea fills the bottom half.

**ActiveForm:** Updating Agents page layout for conversation area

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_VOICE_AGENT_PANE.md" })
```
