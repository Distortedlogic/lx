# Voice Context Extraction

## Goal

Extract voice pipeline state from VoiceBanner into a shared `VoiceContext` provided via Dioxus `use_context_provider`. VoiceBanner and AgentCard both consume the context. No visual changes — pure refactor that preserves all current behavior while enabling cross-component state access.

## Why

- VoiceBanner currently owns all voice state (status, transcript, pending, pcm_buffer) as local signals — no other component can read them
- The Agents page needs AgentCard to display the voice transcript as live output, and needs mic volume for visualization
- Prop drilling signals through the component tree is not idiomatic Dioxus — context providers are the standard pattern for shared state
- The voice types (`VoiceStatus`, `TranscriptEntry`) are defined inside `voice_banner.rs` but will be needed by `agent_card.rs` and `mod.rs` — they belong in a shared module

## Files affected

| File | Change |
|------|--------|
| `crates/lx-desktop/src/pages/agents/voice_context.rs` | New file: `VoiceContext` struct, `VoiceStatus` enum, `TranscriptEntry` struct, `PipelineStage` enum |
| `crates/lx-desktop/src/pages/agents/voice_banner.rs` | Remove type definitions (moved to context), remove local signals, read/write via `use_context::<VoiceContext>()` |
| `crates/lx-desktop/src/pages/agents/mod.rs` | Add `mod voice_context`, call `use_context_provider` with `VoiceContext::new()` before rendering children |

## Dioxus context API

`use_context_provider(|| value)` is called in the parent component. Child components call `use_context::<T>()` to get a clone of the provided value. Since `VoiceContext` contains only `Signal<T>` fields (which are `Copy`), cloning the context struct is cheap — it copies signal IDs, not data. The context must be provided before any child component that consumes it is rendered.

## Dioxus Signal access patterns

- `signal()` — call the signal as a function to get a clone of the inner value. Works when `T: Clone`. Use for reads in RSX and conditionals.
- `signal.read()` — returns `Ref<T>` for borrowing without clone. Use for iterating collections.
- `signal.write()` — returns `RefMut<T>` for in-place mutation.
- `signal.set(val)` — replace the entire value.
- Signals are `Copy`. Structs containing only Signal fields can derive `Clone, Copy`.

## Task List

### Task 1: Create voice_context.rs

Create `crates/lx-desktop/src/pages/agents/voice_context.rs` containing:

1. `VoiceStatus` enum — move verbatim from `voice_banner.rs` lines 11-27 (the enum and its `Display` impl). Derive `Clone, Copy, PartialEq`. Make it `pub`.

2. `PipelineStage` enum with variants: `Idle`, `Transcribing`, `QueryingLlm`, `SynthesizingSpeech`. Derive `Clone, Copy, PartialEq`. Make it `pub`. Add a `Display` impl:
   - `Idle` → `""`
   - `Transcribing` → `"TRANSCRIBING"`
   - `QueryingLlm` → `"QUERYING_LLM"`
   - `SynthesizingSpeech` → `"SYNTHESIZING_SPEECH"`

3. `TranscriptEntry` struct — move verbatim from `voice_banner.rs` lines 30-33. Make it `pub` with `pub` fields. Keep the `Clone` derive.

4. `VoiceContext` struct. Derive `Clone, Copy`. All fields `pub`:
   - `status: Signal<VoiceStatus>`
   - `transcript: Signal<Vec<TranscriptEntry>>`
   - `pending: Signal<HashMap<String, String>>`
   - `pcm_buffer: Signal<Vec<u8>>`
   - `rms: Signal<f32>`
   - `pipeline_stage: Signal<PipelineStage>`
   - `widget: Signal<Option<dioxus_widget_bridge::TsWidgetHandle>>`

5. `impl VoiceContext` with `pub fn new() -> Self` that calls `use_signal` for each field:
   - `status: use_signal(|| VoiceStatus::Idle)`
   - `transcript: use_signal(Vec::new)`
   - `pending: use_signal(HashMap::new)`
   - `pcm_buffer: use_signal(Vec::new)`
   - `rms: use_signal(|| 0.0)`
   - `pipeline_stage: use_signal(|| PipelineStage::Idle)`
   - `widget: use_signal(|| None)`

Required imports at the top of the file: `use dioxus::prelude::*;`, `use std::collections::HashMap;`.

### Task 2: Update mod.rs to provide context

In `crates/lx-desktop/src/pages/agents/mod.rs`:

1. Add `mod voice_context;` after the existing `mod` declarations (after line 3).

2. Inside the `Agents` component function, add this line before the `rsx!` block:
   ```
   use_context_provider(voice_context::VoiceContext::new);
   ```

Do NOT remove any existing imports or change the existing card rendering. The mocked `AgentCard` calls with `AgentStatus` props remain unchanged — they are replaced in a later work item.

### Task 3: Refactor VoiceBanner to use context

In `crates/lx-desktop/src/pages/agents/voice_banner.rs`:

1. Delete the `VoiceStatus` enum (lines 11-17), its `Display` impl (lines 19-27), and the `TranscriptEntry` struct (lines 30-33).

2. Add this import after the existing imports:
   ```
   use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
   ```

3. At the start of the `VoiceBanner` component function, replace these four lines:
   ```
   let mut status: Signal<VoiceStatus> = use_signal(|| VoiceStatus::Idle);
   let mut pcm_buffer: Signal<Vec<u8>> = use_signal(Vec::new);
   let mut transcript: Signal<Vec<TranscriptEntry>> = use_signal(Vec::new);
   let mut pending: Signal<HashMap<String, String>> = use_signal(HashMap::new);
   ```
   With:
   ```
   let ctx = use_context::<VoiceContext>();
   ```
   Keep the `use_ts_widget` call unchanged. After it, add:
   ```
   use_effect(move || { ctx.widget.set(Some(widget)); });
   ```

4. In the `use_future` message loop, update every signal reference:
   - `pcm_buffer.write()` → `ctx.pcm_buffer.write()`
   - `status.set(...)` → `ctx.status.set(...)`
   - `pending.write()` → `ctx.pending.write()`
   - The `spawn` call becomes:
     ```
     spawn(async move {
       if let Err(e) = run_pipeline(buffer, widget, ctx).await {
         ctx.transcript.write().push(TranscriptEntry { is_user: false, text: format!("Error: {e}") });
         ctx.pipeline_stage.set(PipelineStage::Idle);
         widget.send_update(serde_json::json!({ "type": "stop_capture" }));
       }
     });
     ```

5. In the `audio_playing` handler, replace `pending.write()` and `transcript.write()` with `ctx.pending.write()` and `ctx.transcript.write()`.

6. In the RSX rendering block:
   - `status()` → `ctx.status()`
   - `transcript.read()` → `ctx.transcript.read()`
   - Remove `let entries = transcript.read().clone();` and replace with `let entries = ctx.transcript.read().clone();`

7. Change `run_pipeline` signature from:
   ```
   async fn run_pipeline(
     pcm: Vec<u8>,
     widget: dioxus_widget_bridge::TsWidgetHandle,
     mut transcript: Signal<Vec<TranscriptEntry>>,
     mut pending: Signal<HashMap<String, String>>,
   ) -> anyhow::Result<()> {
   ```
   To:
   ```
   async fn run_pipeline(
     pcm: Vec<u8>,
     widget: dioxus_widget_bridge::TsWidgetHandle,
     ctx: VoiceContext,
   ) -> anyhow::Result<()> {
   ```

8. In the `run_pipeline` body:
   - `transcript.write()` → `ctx.transcript.write()`
   - `pending.write()` → `ctx.pending.write()`
   - Add pipeline stage tracking:
     - Before Whisper infer: `ctx.pipeline_stage.set(PipelineStage::Transcribing);`
     - Before Claude CLI query: `ctx.pipeline_stage.set(PipelineStage::QueryingLlm);`
     - Before the TTS loop: `ctx.pipeline_stage.set(PipelineStage::SynthesizingSpeech);`
     - At function end (after TTS loop): `ctx.pipeline_stage.set(PipelineStage::Idle);`
     - In the early return for empty text: `ctx.pipeline_stage.set(PipelineStage::Idle);` before returning
     - In the early return for empty sentences: `ctx.pipeline_stage.set(PipelineStage::Idle);` before returning

9. Remove `use std::collections::HashMap;` from the imports — it is no longer needed in this file since `pending` is accessed through context.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

- **Signal is Copy.** Pass `Signal<T>` by value, never by reference. `VoiceContext` derives `Clone, Copy` because all its fields are Signals.
- **`use_context_provider` must be called before any child accesses the context.** Place it at the top of the Agents component, before `rsx!`.
- **Do not change any visual output.** The UI must render identically before and after this refactor. The only difference is where state lives.
- **Do not modify agent_card.rs or mcp_panel.rs.** Those changes happen in a separate work item.
- **Do not modify any TypeScript files.** The TS changes for RMS happen in a separate work item.
- **Do not remove `AgentStatus` import from mod.rs.** The mocked agent cards still use it. It is removed in the AGENT_PAGE_WIRING work item.
- **`VoiceContext::new()` calls `use_signal` internally.** It must be called inside a Dioxus component context (which `use_context_provider` guarantees).
- **300 line file limit.** voice_banner.rs is currently 222 lines and will shrink (type definitions removed). voice_context.rs should be under 60 lines.

---

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/VOICE_CONTEXT_EXTRACTION.md" })
```

Then call `next_task` to begin.
