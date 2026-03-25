# Goal

Replace the opaque `ClaudeCliBackend.query()` call in the voice pipeline with PTY-based execution that creates a visible terminal pane for each voice turn. The user sees the full `claude` CLI interaction in real time. The pipeline captures the PTY output, strips ANSI codes, and feeds the cleaned text to TTS.

# Why

The current voice pipeline calls `ClaudeCliBackend.query()` which runs `tokio::process::Command::new("claude").output().await` — a one-shot opaque execution. The user sees nothing until the TTS response plays. By running `claude` through a PTY terminal pane, the user sees the full interaction: thinking indicators, tool use, streaming output. The terminal pane is the execution surface AND the display surface.

# Prerequisites

- WU-A (common_pty shell command support) must be completed — `sh -c cmd` execution is needed
- WU-B (route restructure) must be completed — routes are stable
- The spawn channel in `shell.rs` (`TerminalSpawnRequest` via `mpsc::UnboundedSender`) must be available via `use_context` — this is pre-existing infrastructure

# How it works

1. Audio is transcribed to text (unchanged)
2. The text is shell-escaped and embedded in a `claude` command string
3. A PTY session is created via `common_pty::get_or_create(id, 80, 24, Some("."), Some(&cmd))`
4. The pipeline subscribes to the PTY's output broadcast channel
5. A `TerminalSpawnRequest` is sent to create a terminal tab — the TerminalView component mounts and connects to the same PTY session (because `get_or_create` returns the existing session by ID)
6. Both the pipeline and the terminal widget receive the same output stream
7. The pipeline accumulates output bytes until `broadcast::error::RecvError::Closed` (the `sh -c` process exited, meaning `claude` exited)
8. The accumulated bytes are stripped of ANSI escape codes and fed to TTS

# Shell escaping

The transcribed text is embedded in a shell command: `claude -p '{escaped}'`. The escaping strategy: wrap in single quotes, replace every `'` in the text with `'\''` (end quote, escaped quote, start quote). This is the POSIX-standard approach. The system prompt constant (`SYSTEM_PROMPT` in `voice_backend.rs`) contains no single quotes and is safe to single-quote directly.

# Session continuity

The existing `SESSION_ID` (LazyLock UUID) and `SESSION_CREATED` (AtomicBool) in `voice_backend.rs` are reused. First turn uses `--session-id {SID}`, subsequent turns use `--resume {SID}`. After a successful turn, `SESSION_CREATED` is set to true.

# No --output-format text

The `--output-format text` flag is NOT used. Without it, `claude` detects the PTY and produces rich formatted output (colors, streaming, tool use display). The terminal pane renders this beautifully. For TTS, `strip-ansi-escapes` removes the terminal formatting. The system prompt already tells claude to produce TTS-friendly content, so the stripped text is clean.

# Files Affected

| File | Change |
|------|--------|
| `Cargo.toml` | Add strip-ansi-escapes dependency |
| `src/pages/agents/voice_context.rs` | Add voice_turn_count signal |
| `src/pages/agents/voice_banner.rs` | Rewrite run_pipeline to use PTY |

# Task List

### Task 1: Add strip-ansi-escapes dependency

**Subject:** Add the ANSI stripping crate to Cargo.toml

**Description:** Edit `crates/lx-desktop/Cargo.toml`. Add this line in the `[dependencies]` section, after the `base64` line:

```toml
strip-ansi-escapes = "0.2"
```

**ActiveForm:** Adding strip-ansi-escapes dependency

---

### Task 2: Add voice_turn_count to VoiceContext

**Subject:** Track voice turn count for terminal tab naming

**Description:** Edit `crates/lx-desktop/src/pages/agents/voice_context.rs`. Add a new field to the `VoiceContext` struct:

```rust
pub voice_turn_count: Signal<u32>,
```

Add it after the existing `widget` field.

In the `provide()` method, initialize it:

```rust
voice_turn_count: Signal::new(0),
```

Add it after the `widget: Signal::new(None),` line.

**ActiveForm:** Adding voice_turn_count to VoiceContext

---

### Task 3: Rewrite run_pipeline to use PTY terminal pane

**Subject:** Replace ClaudeCliBackend.query() with PTY-based execution piped to a terminal pane

**Description:** Edit `crates/lx-desktop/src/pages/agents/voice_banner.rs`. This task has 4 sequential steps.

**Step A — Make voice_backend items public.** Edit `crates/lx-desktop/src/voice_backend.rs`. Change line 5 from `const SYSTEM_PROMPT` to `pub const SYSTEM_PROMPT`. Change line 13 from `static SESSION_CREATED` to `pub static SESSION_CREATED`. These are accessed by the new `run_pipeline`.

**Step B — Update imports in voice_banner.rs.** Remove `use common_voice::AgentBackend as _;`. Add these new imports after the existing ones:

```rust
use common_pane_tree::PaneNode;
use tokio::sync::mpsc;
use crate::layout::shell::TerminalSpawnRequest;
use crate::panes::DesktopPane;
```

**Step C — Capture spawn channel in VoiceBanner component body.** In the `VoiceBanner` component function, after the existing `let mut ctx = use_context::<VoiceContext>();` line, add:

```rust
let spawn_tx = use_context::<mpsc::UnboundedSender<TerminalSpawnRequest>>();
```

The spawn channel is provided as context by `shell.rs` line 51 (`use_context_provider(|| spawn_channel.0.clone())`). `use_context` is called in the component body (not inside async), which is required by Dioxus.

Then in the `use_future` loop, inside the `Some("silence_detected")` arm, change:

```rust
if let Err(e) = run_pipeline(buffer, widget, ctx).await {
```

to:

```rust
if let Err(e) = run_pipeline(buffer, widget, ctx, spawn_tx.clone()).await {
```

**Step D — Add shell_escape and rewrite run_pipeline.** Add this function before `run_pipeline`:

```rust
fn shell_escape(s: &str) -> String {
  let mut escaped = String::with_capacity(s.len() + 10);
  escaped.push('\'');
  for ch in s.chars() {
    if ch == '\'' {
      escaped.push_str("'\\''");
    } else {
      escaped.push(ch);
    }
  }
  escaped.push('\'');
  escaped
}
```

Replace the entire `run_pipeline` function with:

```rust
async fn run_pipeline(
  pcm: Vec<u8>,
  widget: dioxus_widget_bridge::TsWidgetHandle,
  mut ctx: VoiceContext,
  spawn_tx: mpsc::UnboundedSender<TerminalSpawnRequest>,
) -> anyhow::Result<()> {
  ctx.pipeline_stage.set(PipelineStage::Transcribing);
  let wav = common_audio::wrap_pcm_as_wav(&pcm, common_audio::SAMPLE_RATE, common_audio::CHANNELS, common_audio::BITS_PER_SAMPLE);
  let transcription = common_whisper::WHISPER.infer(&TranscribeRequest { audio_data: B64.encode(&wav), language: Some("en".into()) }).await?;
  let text = transcription.text.trim().to_owned();
  if text.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    widget.send_update(serde_json::json!({ "type": "stop_capture" }));
    return Ok(());
  }
  ctx.transcript.write().push(TranscriptEntry { is_user: true, text: text.clone() });

  ctx.pipeline_stage.set(PipelineStage::QueryingLlm);

  let escaped = shell_escape(&text);
  let system_escaped = shell_escape(crate::voice_backend::SYSTEM_PROMPT);
  let session_id = &*crate::voice_backend::SESSION_ID;
  let session_created = crate::voice_backend::SESSION_CREATED.load(std::sync::atomic::Ordering::Relaxed);
  let cmd = if session_created {
    format!("claude -p {escaped} --system-prompt {system_escaped} --resume {session_id}")
  } else {
    format!("claude -p {escaped} --system-prompt {system_escaped} --session-id {session_id}")
  };

  let terminal_id = uuid::Uuid::new_v4().to_string();
  let session = common_pty::get_or_create(&terminal_id, 80, 24, Some("."), Some(&cmd))
    .map_err(|e| anyhow::anyhow!("pty create failed: {e}"))?;
  let (_initial, mut rx) = session.subscribe();

  let mut turn_count = ctx.voice_turn_count;
  let count = turn_count() + 1;
  turn_count.set(count);
  let title = format!("Voice Turn {count}");
  let pane = PaneNode::Leaf(DesktopPane::Terminal {
    id: terminal_id.clone(),
    working_dir: ".".into(),
    command: Some(cmd.clone()),
    name: Some(title.clone()),
  });

  let _ = spawn_tx.send(TerminalSpawnRequest {
    id: terminal_id.clone(),
    title,
    pane,
  });

  let mut accumulated = Vec::new();
  loop {
    match rx.recv().await {
      Ok(bytes) => accumulated.extend_from_slice(&bytes),
      Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
      Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
    }
  }

  crate::voice_backend::SESSION_CREATED.store(true, std::sync::atomic::Ordering::Relaxed);

  let stripped = strip_ansi_escapes::strip(&accumulated);
  let response = String::from_utf8_lossy(&stripped).trim().to_owned();
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
    widget.send_update(serde_json::json!({
        "type": "audio_response",
        "data": B64.encode(&wav_bytes),
        "id": id,
    }));
  }
  ctx.pipeline_stage.set(PipelineStage::Idle);
  Ok(())
}
```

The transcription → text → PTY → accumulate → TTS flow replaces the old transcription → ClaudeCliBackend.query() → TTS flow. The TTS portion (split_sentences → tts → audio_response) is unchanged from the old code.

**ActiveForm:** Rewriting run_pipeline to use PTY terminal pane

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_VOICE_TERMINAL_PIPE.md" })
```
