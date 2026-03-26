# Goal

Fix the widget bridge build pipeline so TypeScript changes propagate to the running app. Replace the fake-streaming `query_streaming` with real streaming via `--output-format stream-json --verbose`. Remove the typewriter animation hack from agent.ts.

# Why

Three problems:

1. **Build pipeline is broken.** `build.rs` `rerun-if-changed` only watches `widget-bridge/src` and `widget-bridge/widgets`. Changes to `audio-playback/src/` and `audio-capture/src/` are invisible to Cargo. The bundle never rebuilds. Every audio-playback edit made in this conversation was never loaded by the running app.

2. **Text doesn't stream.** `query_streaming` uses `--output-format text` which makes claude buffer internally and write everything at once on exit. The 256-byte read loop gets the entire response in one chunk. The typewriter animation masks this but adds artificial delay.

3. **Typewriter is a hack.** With real streaming deltas, text arrives progressively on its own. The typewriter queue adds latency on top of already-streaming data.

# Research-backed decisions

- `--output-format stream-json --verbose` emits one JSON line per event, line-buffered through pipes (Claude CLI is Node.js, writes directly to fd via libuv). Verified by running `claude -p "say ok" --output-format stream-json --verbose --system-prompt "..." --session-id {uuid}` — each line flushes immediately.
- `--include-partial-messages` adds `stream_event` type events with `content_block_delta` deltas containing per-token text. Without it, the full assistant message arrives as one `assistant` event after the turn completes.
- The `result` event (always the last line) has a `.result` field containing the full assembled response text — used for TTS.
- Session ID must be a valid UUID. `--session-id` for first turn, `--resume` for subsequent turns. Both work with `stream-json`.
- `rerun-if-changed` on a directory path only fires when files are added/removed, not when existing file contents change. Individual files must be enumerated.

# Files Affected

| File | Change |
|------|--------|
| `lx/crates/lx-desktop/build.rs` | Add rerun-if-changed for audio-playback and audio-capture source files |
| `lx/crates/lx-desktop/src/voice_backend.rs` | Replace `query_streaming` with stream-json based implementation |
| `lx/crates/lx-desktop/src/pages/agents/voice_banner.rs` | Update `run_pipeline` to use new streaming function |
| `dioxus-common/ts/widget-bridge/widgets/agent.ts` | Remove typewriter animation, restore direct text append |

# Task List

### Task 1: Fix build.rs to detect audio-playback and audio-capture changes

**Subject:** Add rerun-if-changed directives so the bundle rebuilds when TS source changes

**Description:** Edit `crates/lx-desktop/build.rs`. The current `rerun-if-changed` loop (lines 11-13) watches `widget-bridge/src` and `widget-bridge/widgets`. Directory-level watching only detects file additions/removals, not content changes. Add individual file watches for the audio packages.

After line 13 (`}`), add:

```rust
for pkg in &["audio-playback", "audio-capture"] {
  let pkg_src = dioxus_common.join(format!("ts/{pkg}/src"));
  if pkg_src.exists() {
    for entry in std::fs::read_dir(&pkg_src).unwrap() {
      let entry = entry.unwrap();
      if entry.path().extension().is_some_and(|e| e == "ts") {
        println!("cargo:rerun-if-changed={}", entry.path().display());
      }
    }
  }
}
```

Also enumerate individual `.ts` files in the widget-bridge directories for reliable content-change detection. Replace lines 11-13:

```rust
for dir in &["src", "widgets"] {
  println!("cargo:rerun-if-changed={}", widget_bridge_dir.join(dir).display());
}
```

With:

```rust
for dir in &["src", "widgets"] {
  let d = widget_bridge_dir.join(dir);
  if d.exists() {
    for entry in std::fs::read_dir(&d).unwrap() {
      let entry = entry.unwrap();
      if entry.path().extension().is_some_and(|e| e == "ts") {
        println!("cargo:rerun-if-changed={}", entry.path().display());
      }
    }
  }
}
```

This enumerates every `.ts` file in `widget-bridge/src/`, `widget-bridge/widgets/`, `audio-playback/src/`, and `audio-capture/src/`. Content changes to any of these files trigger a rebuild.

**ActiveForm:** Fixing build.rs change detection for TypeScript packages

---

### Task 2: Replace query_streaming with stream-json implementation

**Subject:** Use --output-format stream-json --verbose for real streaming with per-token deltas

**Description:** Edit `crates/lx-desktop/src/voice_backend.rs`. Replace the entire `query_streaming` function (lines 44-65) and update `build_args`.

First, change `build_args` to support both output formats. Rename it and add a parameter:

Replace lines 17-25:

```rust
fn build_args(text: &str) -> Vec<&str> {
  let mut args = vec!["-p", text, "--output-format", "text", "--system-prompt", SYSTEM_PROMPT];
  if SESSION_CREATED.load(Ordering::Relaxed) {
    args.extend(["--resume", &SESSION_ID]);
  } else {
    args.extend(["--session-id", &SESSION_ID]);
  }
  args
}
```

With:

```rust
fn build_args_text(text: &str) -> Vec<&str> {
  let mut args = vec!["-p", text, "--output-format", "text", "--system-prompt", SYSTEM_PROMPT];
  if SESSION_CREATED.load(Ordering::Relaxed) {
    args.extend(["--resume", &SESSION_ID]);
  } else {
    args.extend(["--session-id", &SESSION_ID]);
  }
  args
}

fn build_args_stream(text: &str) -> Vec<&str> {
  let mut args = vec![
    "-p", text,
    "--output-format", "stream-json",
    "--verbose",
    "--include-partial-messages",
    "--system-prompt", SYSTEM_PROMPT,
  ];
  if SESSION_CREATED.load(Ordering::Relaxed) {
    args.extend(["--resume", &SESSION_ID]);
  } else {
    args.extend(["--session-id", &SESSION_ID]);
  }
  args
}
```

Update `ClaudeCliBackend::query` to use `build_args_text`:

```rust
let args = build_args_text(text);
```

Replace the entire `query_streaming` function with:

```rust
pub async fn query_streaming(text: &str, mut on_chunk: impl FnMut(&str)) -> anyhow::Result<String> {
  use tokio::io::AsyncBufReadExt;
  let args = build_args_stream(text);
  let mut child = tokio::process::Command::new("claude")
    .args(&args)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()?;
  let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?;
  let mut lines = tokio::io::BufReader::new(stdout).lines();
  let mut result_text = String::new();
  while let Some(line) = lines.next_line().await? {
    let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
    match event["type"].as_str() {
      Some("stream_event") => {
        if event["event"]["type"].as_str() == Some("content_block_delta")
          && event["event"]["delta"]["type"].as_str() == Some("text_delta")
        {
          if let Some(text) = event["event"]["delta"]["text"].as_str() {
            on_chunk(text);
          }
        }
      }
      Some("result") => {
        if let Some(text) = event["result"].as_str() {
          result_text = text.trim().to_owned();
        }
      }
      _ => {}
    }
  }
  let status = child.wait().await?;
  if !status.success() && result_text.is_empty() {
    anyhow::bail!("claude cli failed");
  }
  SESSION_CREATED.store(true, Ordering::Relaxed);
  Ok(result_text)
}
```

This reads stdout line-by-line via `BufReader::lines()`. Each line is parsed as JSON. `stream_event` events with `content_block_delta` type yield per-token text deltas via `on_chunk`. The `result` event (always last) provides the full assembled response for TTS. `BufReader::lines()` yields each line as soon as `\n` arrives in the pipe — no buffering delay since Claude CLI flushes each JSON line immediately.

The `AsyncReadExt` import at the top of the file (line 5) can be removed — it's no longer used. Replace with `tokio::io::AsyncBufReadExt` inside the function (already included via the `use` statement in the function body).

**ActiveForm:** Replacing query_streaming with stream-json implementation

---

### Task 3: Remove typewriter animation from agent.ts

**Subject:** Restore direct text append since real streaming provides progressive updates

**Description:** Edit `/home/entropybender/repos/dioxus-common/ts/widget-bridge/widgets/agent.ts`.

Remove `pendingText` and `typeTimer` from the `AgentState` interface (lines 10-11). Remove their initialization in the state constructor (lines 114-115).

Replace the `assistant_chunk` handler (lines 163-184) with:

```typescript
      case "assistant_chunk": {
        if (!state.currentBubble) {
          state.currentBubble = createBubble(state.messagesDiv, "assistant");
          state.currentText = "";
        }
        state.currentText += msg.text ?? "";
        state.currentBubble.textContent = state.currentText;
        autoScroll(state);
        break;
      }
```

Replace the `assistant_done` handler (lines 186-198) with:

```typescript
      case "assistant_done": {
        state.currentBubble = null;
        state.currentText = "";
        break;
      }
```

This is the original pre-typewriter code. Each `assistant_chunk` directly appends text and updates the DOM. With `stream-json` delivering per-token deltas, each chunk is 1-5 characters arriving every ~50-100ms (LLM token generation rate). The text appears progressively without artificial animation.

**ActiveForm:** Removing typewriter animation from agent.ts

---

### Task 4: Remove unused AsyncReadExt import from voice_backend.rs

**Subject:** Remove unused import left over from the old raw-byte read approach

**Description:** Edit `crates/lx-desktop/src/voice_backend.rs`. Remove line 5:

```rust
use tokio::io::AsyncReadExt;
```

This import was used by the old `query_streaming` which read raw bytes. The new implementation uses `AsyncBufReadExt` imported inside the function body.

**ActiveForm:** Cleaning up unused imports

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/VOICE_STREAMING_AND_BUILD_FIX.md" })
```
