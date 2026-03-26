# Voice Audio Playback — Accumulated Knowledge

## Architecture

Voice pipeline: microphone → AudioCapture (JS) → PCM chunks → Whisper STT → text → ClaudeCliBackend (claude CLI via tokio::process) → response text → Kokoro TTS → WAV bytes → base64 → voice widget (JS) → AudioPlayback → speaker

Display: response text sent to agent.ts widget via `assistant_chunk`/`assistant_done` messages. User transcription sent via `user_display`.

## AudioPlayback Implementation

File: `dioxus-common/ts/audio-playback/src/playback.ts`

### HTMLAudioElement approach (current, working)
- Creates `new Audio(url)` from a blob URL of the base64-decoded WAV
- Waits for `oncanplaythrough` before calling `play()`
- Queue-based: items enqueued, played sequentially, `onended` triggers next
- Works in WebKit2GTK

### Web Audio API approach (attempted, FAILED)
- `AudioContext` + `decodeAudioData` + `AudioBufferSourceNode`
- `decodeAudioData` silently failed in WebKit2GTK — no audio played at all
- The catch block swallowed the error and called `playNext()`, so the pipeline completed but no sound was produced
- DO NOT USE Web Audio API for audio playback in this WebKit2GTK desktop app

## Audio Start Clipping Issue

### Symptom
First ~50ms of voice audio is clipped/distorted. Audio sounds "popped" or "cut off" at the beginning before normalizing.

### Root cause
Kokoro TTS output starts at full amplitude with no ramp-up. The abrupt transition from silence to signal creates a pop/click artifact. This is inherent to the TTS output, not the playback mechanism.

### Fix applied
50ms of zero-valued PCM silence prepended to the WAV data on the Rust side (`prepend_silence` function in `voice_banner.rs`). Parses the WAV header to determine sample rate, channels, and bit depth, then inserts the correct number of zero bytes before the PCM data and updates the WAV header size fields.

### Things that did NOT fix it
- `oncanplaythrough` wait — addresses load timing, not audio content
- Web Audio API rewrite — broke playback entirely in WebKit2GTK
- `tokio::time::sleep(150ms)` / `sleep(300ms)` delay before sending audio — hacky, unreliable, removed

## Text Streaming

### Problem
`ClaudeCliBackend.query()` uses `tokio::process::Command::output().await` which waits for the entire process to complete. The full response arrives at once.

### query_streaming approach (current)
`voice_backend::query_streaming()` spawns claude with `Stdio::piped()` stdout and reads 256-byte chunks. Each chunk is sent to the agent widget as an `assistant_chunk`. However, `--output-format text` causes claude to internally buffer and write everything at once when done. The pipe buffering means chunks arrive as one block, not progressively.

### Typewriter animation (current workaround)
`agent.ts` queues incoming text in `pendingText` and drains 1 character per frame at 60fps (~60 chars/sec). This creates progressive text appearance regardless of whether data arrives in one chunk or many. The `assistant_done` message flushes remaining pending text immediately.

### What would give true streaming
- `--output-format stream-json` — outputs one JSON event per line, flushed immediately. Would need to parse JSON events to extract text content. Format not verified.
- Running through a PTY (forces line-buffered output) — but PTY approach was abandoned due to shell escaping, sentinel parsing, and formatting complexity.

## TTS Configuration

- Engine: Kokoro (local, via HTTP at KOKORO_URL, default localhost:8094)
- Voice: `am_michael`
- Language code: `a`
- Speed: `1.2` (1.0 was too slow, 1.4 was too fast)
- Full response synthesized as one audio chunk (not per-sentence)

## Approaches Tried and Abandoned

### Terminal pane approach
Ran `claude` via PTY in a terminal pane. Required shell escaping of transcribed text, sentinel parsing (`<<<END>>>`), `stty -echo` for suppressing command echo, `printf` for formatting. Raw monospace output with formatting artifacts. Multiple tabs created per voice turn. Abandoned for the agent widget approach.

### Per-sentence TTS
Split response into sentences, synthesized each separately, sent as individual `audio_response` messages. Produced audible gaps between sentences. Replaced with single full-response TTS call.

### sleep() delays for audio clipping
Added `tokio::time::sleep(150ms)` then `300ms` before sending audio. Unreliable hack. Replaced with WAV silence prepending.

## Key WebKit2GTK Constraints

- Web Audio API `decodeAudioData` silently fails — use HTMLAudioElement instead
- HTMLAudioElement with blob URLs works reliably
- `oncanplaythrough` should be used before `play()` to avoid race conditions
- Pipe buffering prevents true streaming — `--output-format text` with piped stdout delivers data all at once

## Files

| File | Role |
|------|------|
| `dioxus-common/ts/audio-playback/src/playback.ts` | AudioPlayback class (HTMLAudioElement, queue, callbacks) |
| `dioxus-common/ts/widget-bridge/widgets/voice.ts` | Voice widget (AudioCapture, AudioPlayback, status transitions) |
| `dioxus-common/ts/widget-bridge/widgets/agent.ts` | Agent chat widget (message bubbles, typewriter animation, user_display) |
| `lx/crates/lx-desktop/src/pages/agents/voice_banner.rs` | VoiceBanner component, voice/agent widgets, run_pipeline, TTS, prepend_silence |
| `lx/crates/lx-desktop/src/voice_backend.rs` | ClaudeCliBackend, query_streaming, session management |
| `dioxus-common/crates/common-kokoro/src/lib.rs` | Kokoro TTS client (SpeechRequest, BinaryInferenceClient) |
| `dioxus-common/crates/common-audio/src/lib.rs` | WAV utilities (wrap_pcm_as_wav, chunk_wav) |
