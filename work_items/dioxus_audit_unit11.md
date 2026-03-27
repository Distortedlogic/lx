# Unit 11: VoiceContext — Signal-per-field → Store

## Problem

`VoiceContext` in `crates/lx-desktop/src/pages/agents/voice_context.rs` wraps 8 independent fields in `Signal<T>`. Components subscribe to the entire struct when they only need 1-2 fields. Converting to Store provides field-level granularity.

## Design Decision: widget field

`dioxus_widget_bridge::TsWidgetHandle` does not derive `PartialEq` (it's `#[derive(Clone, Copy)]` only, defined at `dioxus-common/crates/dioxus-widget-bridge/src/hook.rs:35-38`). `#[derive(Store)]` requires all fields to implement `PartialEq`. The `widget` field must stay as a separate `Signal<Option<TsWidgetHandle>>` outside the store. The context struct becomes a wrapper holding both a `Store<VoiceData>` (7 reactive fields) and a `Signal<Option<TsWidgetHandle>>`.

## Current Code

```rust
// crates/lx-desktop/src/pages/agents/voice_context.rs lines 49-76
#[derive(Clone, Copy)]
pub struct VoiceContext {
  pub status: Signal<VoiceStatus>,
  pub transcript: Signal<Vec<TranscriptEntry>>,
  pub pcm_buffer: Signal<Vec<u8>>,
  pub rms: Signal<f32>,
  pub pipeline_stage: Signal<PipelineStage>,
  pub widget: Signal<Option<dioxus_widget_bridge::TsWidgetHandle>>,
  pub always_listen: Signal<bool>,
  pub barge_in: Signal<bool>,
}

impl VoiceContext {
  pub fn provide() -> Self { ... }
}
```

## Files

| File | Role |
|------|------|
| `crates/lx-desktop/src/pages/agents/voice_context.rs` | Definition — rewrite struct, split into VoiceData store + widget signal |
| `crates/lx-desktop/src/pages/agents/voice_banner.rs` | Heavy consumer — ~30 field accesses across reads/writes |
| `crates/lx-desktop/src/pages/agents/voice_pipeline.rs` | Consumer — `run_pipeline` takes VoiceContext, accesses ~7 fields |
| `crates/lx-desktop/src/pages/agents/mod.rs` | Provider — calls `VoiceContext::provide()` |

## Tasks

### 1. Rewrite `crates/lx-desktop/src/pages/agents/voice_context.rs`

Keep `VoiceStatus`, `PipelineStage`, `TranscriptEntry` types and their Display impls unchanged (lines 1-47). Replace lines 49-76 with:

```rust
#[derive(Store, Clone, PartialEq)]
pub struct VoiceData {
  pub status: VoiceStatus,
  pub transcript: Vec<TranscriptEntry>,
  pub pcm_buffer: Vec<u8>,
  pub rms: f32,
  pub pipeline_stage: PipelineStage,
  pub always_listen: bool,
  pub barge_in: bool,
}

#[derive(Clone, Copy)]
pub struct VoiceContext {
  pub data: Store<VoiceData>,
  pub widget: Signal<Option<dioxus_widget_bridge::TsWidgetHandle>>,
}
```

Remove the `impl VoiceContext` block entirely (the `provide()` method). Provider in `mod.rs` creates both the store and signal.

### 2. Update `crates/lx-desktop/src/pages/agents/mod.rs`

Replace the entire file content. Current file is 25 lines.

**Line 11**: Change import to add types needed for `use_store` initialization:
```rust
// OLD (line 11):
use self::voice_context::VoiceContext;

// NEW:
use self::voice_context::{PipelineStage, VoiceContext, VoiceData, VoiceStatus};
```

**Lines 14-15**: Replace provide call:
```rust
// OLD:
let _ctx = VoiceContext::provide();

// NEW:
let data = use_store(|| VoiceData {
  status: VoiceStatus::Idle,
  transcript: Vec::new(),
  pcm_buffer: Vec::new(),
  rms: 0.0,
  pipeline_stage: PipelineStage::Idle,
  always_listen: false,
  barge_in: false,
});
let ctx = VoiceContext { data, widget: Signal::new(None) };
use_context_provider(|| ctx);
```

### 3. Update `crates/lx-desktop/src/pages/agents/voice_banner.rs`

**Line 1**: No import changes needed. `VoiceData` is not referenced by name in this file — all access goes through `ctx.data.field()`. The existing imports stay as-is:
```rust
use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
```

**Line 13**: Context retrieval stays unchanged:
```rust
let mut ctx = use_context::<VoiceContext>();
```
`VoiceContext` is still the context type. It's the same `#[derive(Clone, Copy)]` struct — just with different internals.

**Line 14**: Widget set stays unchanged — `ctx.widget` is still `Signal<Option<TsWidgetHandle>>`:
```rust
use_hook(|| ctx.widget.set(Some(voice_widget)));
```

**Read patterns** — `(ctx.field)()` becomes `ctx.data.field().cloned()` for store fields:
| Old (line) | New |
|------------|-----|
| `(ctx.status)()` (line 32) | `ctx.data.status().cloned()` |
| `(ctx.pipeline_stage)()` (line 38) | `ctx.data.pipeline_stage().cloned()` |
| `(ctx.always_listen)()` (lines 46, 65) | `ctx.data.always_listen().cloned()` |
| `(ctx.barge_in)()` (line 56) | `ctx.data.barge_in().cloned()` |
| `(ctx.status)()` (line 119) | `ctx.data.status().cloned()` |
| `(ctx.rms)()` (line 128) | `ctx.data.rms().cloned()` |
| `(ctx.pipeline_stage)()` (line 129) | `ctx.data.pipeline_stage().cloned()` |
| `ctx.transcript.read()` (line 130) | `ctx.data.transcript().read()` |
| `(ctx.always_listen)()` (line 133) | `ctx.data.always_listen().cloned()` |

**Write patterns** — `ctx.field.set(v)` becomes `ctx.data.field().set(v)`:
| Old (line) | New |
|------------|-----|
| `ctx.pcm_buffer.write().extend_from_slice(...)` (line 33) | `ctx.data.pcm_buffer().write().extend_from_slice(...)` |
| `ctx.pcm_buffer.write().clear()` (lines 40, 86) | `ctx.data.pcm_buffer().write().clear()` |
| `std::mem::take(&mut *ctx.pcm_buffer.write())` (line 44) | `std::mem::take(&mut *ctx.data.pcm_buffer().write())` |
| `ctx.barge_in.set(false)` (line 57) | `ctx.data.barge_in().set(false)` |
| `ctx.transcript.write().push(...)` (line 62) | `ctx.data.transcript().push(...)` |
| `ctx.pipeline_stage.set(...)` (line 63) | `ctx.data.pipeline_stage().set(...)` |
| `ctx.rms.set(...)` (line 74) | `ctx.data.rms().set(...)` |
| `ctx.status.set(...)` (lines 78-82) | `ctx.data.status().set(...)` |
| `ctx.always_listen.set(...)` (lines 177, 180, 196) | `ctx.data.always_listen().set(...)` |

**Function signatures** (lines 220, 237) — type stays `VoiceContext` (not `Store<VoiceContext>`):
```rust
// OLD:
fn handle_keyword_detected(voice_widget: dioxus_widget_bridge::TsWidgetHandle, mut ctx: VoiceContext) {
async fn handle_utterance(pcm: Vec<u8>, agent_widget: dioxus_widget_bridge::TsWidgetHandle, mut ctx: VoiceContext) -> anyhow::Result<()> {
```
Signatures stay the same. The `VoiceContext` struct is still `Copy` and still passed by value. Only field access patterns change within these functions.

**Inside `handle_keyword_detected` (lines 220-235)**:
| Old | New |
|-----|-----|
| `(ctx.status)()` (line 221) | `ctx.data.status().cloned()` |
| `ctx.barge_in.set(true)` (line 226) | `ctx.data.barge_in().set(true)` |
| `ctx.pcm_buffer.write().clear()` (line 229) | `ctx.data.pcm_buffer().write().clear()` |

**Inside `handle_utterance` (lines 237-245)**:
| Old | New |
|-----|-----|
| `ctx.pipeline_stage.set(...)` (line 238) | `ctx.data.pipeline_stage().set(...)` |
| `ctx.pipeline_stage.set(...)` (line 241) | `ctx.data.pipeline_stage().set(...)` |

### 4. Update `crates/lx-desktop/src/pages/agents/voice_pipeline.rs`

**Line 3**: Update import:
```rust
// OLD:
use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};

// NEW:
use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
```
No change needed — still imports `VoiceContext`.

**Line 10**: `use dioxus::prelude::*;` is already present — `Store` is in scope.

**Line 85**: Function signature stays unchanged:
```rust
pub async fn run_pipeline(text: &str, agent_widget: dioxus_widget_bridge::TsWidgetHandle, mut ctx: VoiceContext) -> anyhow::Result<()> {
```

**Field access transformations in `run_pipeline`**:
| Old (line) | New |
|------------|-----|
| `ctx.transcript.write().push(...)` (line 86) | `ctx.data.transcript().push(...)` |
| `ctx.pipeline_stage.set(...)` (line 90) | `ctx.data.pipeline_stage().set(...)` |
| `ctx.pipeline_stage.set(...)` (line 98) | `ctx.data.pipeline_stage().set(...)` |
| `ctx.pipeline_stage.set(...)` (line 102) | `ctx.data.pipeline_stage().set(...)` |
| `ctx.status.set(...)` (line 105) | `ctx.data.status().set(...)` |
| `let mut t = ctx.transcript.write()` (line 118) | `let mut t = ctx.data.transcript().write()` |
| `ctx.pipeline_stage.set(...)` (line 124) | `ctx.data.pipeline_stage().set(...)` |

## Preconditions

- `VoiceData` field types all implement `PartialEq`:
  - `VoiceStatus` — derives `PartialEq` (line 3) ✓
  - `PipelineStage` — derives `PartialEq` (line 24) ✓
  - `TranscriptEntry` — derives `PartialEq` (line 43) ✓
  - `Vec<T>`, `bool`, `f32` — all `PartialEq` ✓
- `Store<T>` implements `Copy` (when inner lens is `Copy`) ✓
- `VoiceContext` wrapper struct with `Store<VoiceData>` + `Signal<Option<TsWidgetHandle>>` — both `Copy`, so `VoiceContext` remains `Copy` ✓

## Verification

`just diagnose` must pass with zero warnings.
