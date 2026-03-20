# Goal

Cross-cutting integration work after all agentic feature work items are complete. Add sandbox deny backends for all new traits, update context files (INVENTORY, STDLIB, PRIORITIES), and run final integration verification.

**Depends on: All other agentic work items should be completed first.**

# Why

- New backend traits (Embed, Pane, Transcribe, Speech, ImageGen) each need deny backends for sandbox enforcement.
- Context files (INVENTORY.md, STDLIB.md, PRIORITIES.md) must reflect all new capabilities.
- A final pass ensures all new modules compile together and tests pass.

# What Changes

**Add deny backends for new traits in `crates/lx/src/backends/restricted.rs`:**

`DenyEmbedBackend`, `DenyPaneBackend`, `DenyTranscribeBackend`, `DenySpeechBackend`, `DenyImageGenBackend` — each returns a clear error message when called inside a sandbox scope that doesn't grant the capability.

**Update `crates/lx/src/stdlib/sandbox.rs` and `sandbox_scope.rs`:** Wire new deny backends into the scope enforcement. When a sandbox policy denies a capability, the corresponding deny backend is installed on the restricted RuntimeCtx.

**Update context files:** INVENTORY.md, STDLIB.md, PRIORITIES.md.

# Files Affected

- `crates/lx/src/backends/restricted.rs` — Add 5 deny backends for new traits
- `crates/lx/src/stdlib/sandbox_scope.rs` — Wire deny backends for new traits
- `agent/INVENTORY.md` — Update
- `agent/STDLIB.md` — Update
- `agent/PRIORITIES.md` — Update

# Task List

### Task 1: Add deny backends for new traits

**Subject:** Add DenyEmbedBackend, DenyPaneBackend, DenyTranscribeBackend, DenySpeechBackend, DenyImageGenBackend

**Description:** Edit `crates/lx/src/backends/restricted.rs`:

Add 5 deny backend structs. Each implements its respective trait and returns `Ok(Value::Err(...))` with a descriptive error:

- `DenyEmbedBackend` → `"embedding access denied by sandbox policy"`
- `DenyPaneBackend` → `"pane access denied by sandbox policy"` (open returns Err, update/close return LxError, list returns empty)
- `DenyTranscribeBackend` → `"transcription access denied by sandbox policy"`
- `DenySpeechBackend` → `"speech synthesis access denied by sandbox policy"`
- `DenyImageGenBackend` → `"image generation access denied by sandbox policy"`

Follow the same pattern as the existing `DenyShellBackend` and `DenyHttpBackend`.

**ActiveForm:** Adding deny backends for new traits

---

### Task 2: Wire deny backends into sandbox scope

**Subject:** Update sandbox_scope.rs to install deny backends for new traits

**Description:** Edit `crates/lx/src/stdlib/sandbox_scope.rs`:

In `build_restricted_ctx`, add checks for new capabilities:
- If `!policy.embed` → replace `embed` with `Arc::new(DenyEmbedBackend)`
- If `!policy.pane` → replace `pane` with `Arc::new(DenyPaneBackend)`
- If `!policy.transcribe` → replace `transcribe` with `Arc::new(DenyTranscribeBackend)`
- If `!policy.speech` → replace `speech` with `Arc::new(DenySpeechBackend)`
- If `!policy.image_gen` → replace `image_gen` with `Arc::new(DenyImageGenBackend)`

Update the `Policy` struct in `sandbox.rs` to include:
- `embed: bool`
- `pane: bool`
- `transcribe: bool`
- `speech: bool`
- `image_gen: bool`

Update preset policies:
- `:pure` → all false
- `:readonly` → all false
- `:local` → all false
- `:network` → embed: true, transcribe/speech/image_gen: false (these are local services, not network)
- `:full` → all true

Update `sandbox.permits` to handle new capability symbols.

**ActiveForm:** Wiring deny backends into sandbox scope

---

### Task 3: Update context files

**Subject:** Update INVENTORY.md, STDLIB.md, and PRIORITIES.md with all new capabilities

**Description:** Edit `agent/INVENTORY.md`:

Under **Stdlib**, add entries for:
- `std/diff` — 5 functions: unified, hunks, apply, edits, merge3
- `std/ws` — 5 functions: connect, send, recv, recv_json, close
- `std/pane` — 4 functions: open, update, close, list. PaneBackend trait. YieldPaneBackend default
- `std/sandbox` — 9 functions: policy, scope, exec, spawn, describe, permits, merge, attenuate

Under **AI extensions** (`std/ai`), add:
- `ai.embed`, `ai.embed_with` — EmbedBackend trait. VoyageEmbedBackend default
- `ai.transcribe`, `ai.transcribe_with` — TranscribeBackend trait. WhisperBackend default
- `ai.speak`, `ai.speak_with` — SpeechBackend trait. KokoroBackend default
- `ai.imagine`, `ai.imagine_with` — ImageGenBackend trait. FluxBackend default

Under **Runtime**, update RuntimeCtx backend list:
- `embed: Arc<dyn EmbedBackend>` — VoyageEmbedBackend default
- `pane: Arc<dyn PaneBackend>` — YieldPaneBackend default
- `transcribe: Arc<dyn TranscribeBackend>` — WhisperBackend default
- `speech: Arc<dyn SpeechBackend>` — KokoroBackend default
- `image_gen: Arc<dyn ImageGenBackend>` — FluxBackend default

Under **lx Packages**, add entries for:
- `pkg/kit/search` — structured code search wrapping rg --json
- `pkg/kit/notify` — structured notifications via emit
- `pkg/kit/template` — lightweight template engine
- `pkg/kit/canvas` — rich visual output via std/pane
- `pkg/connectors/cdp` — Chrome DevTools Protocol client over std/ws
- `pkg/data/vectors` — VectorIndex Class with embedding + cosine similarity

Edit `agent/STDLIB.md`: Add sections for new modules with API examples.

Edit `agent/PRIORITIES.md`: Mark all agentic features as shipped. Add follow-up items (OS-level sandbox enforcement, CDP event handling).

**ActiveForm:** Updating context files with new capabilities

---

### Task 4: Final integration verification

**Subject:** Run full test suite and diagnose, fix any cross-module issues

**Description:** Run `just diagnose` — fix any compilation errors or clippy warnings.

Run `just test` — verify all tests pass.

Verify the DX backend crate compiles: `cd backends/dx && cargo check`.

Check 300-line file limit for all new files. Split if exceeded.

**ActiveForm:** Running final integration verification

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AGENTIC_CROSS_CUTTING.md" })
```

Then call `next_task` to begin.
