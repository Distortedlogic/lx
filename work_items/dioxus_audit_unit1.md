# Unit 1: Replace polling with use_loader in Status page

## Violation

Rules 14 (`use_loader` for data loading) and 18 (no polling loops) in `crates/lx-mobile/src/pages/status.rs`.

The `Status` component uses `use_action` + `use_future` with a 2-second polling loop to call `get_run_status`. This must be replaced with `use_loader`.

## Prerequisites

`RunStatus` in `crates/lx-api/src/types.rs` (line 8) currently derives `Clone, Debug, Default, Serialize, Deserialize` but NOT `PartialEq`. `use_loader` requires `T: PartialEq + Serialize + DeserializeOwned`. You must add `PartialEq` to the derive list.

## Step 1: Add PartialEq to RunStatus

File: `crates/lx-api/src/types.rs`, line 8.

Current:
```rust
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct RunStatus {
```

Replace with:
```rust
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunStatus {
```

## Step 2: Replace the entire contents of status.rs

File: `crates/lx-mobile/src/pages/status.rs`

Current file is 57 lines. Replace the entire contents with the following.

### New imports (replace lines 1-4)

Current:
```rust
use dioxus::prelude::*;
use lx_api::run_api::get_run_status;

use crate::components::pulse_indicator::{ExecutionState, PulseIndicator};
```

Replace with:
```rust
use dioxus::prelude::*;
use lx_api::run_api::get_run_status;

use crate::components::pulse_indicator::{ExecutionState, PulseIndicator};
```

No import changes needed. `use_loader` is in `dioxus::prelude::*`. `use_action` and `use_future` are removed by removing their call sites (no explicit import for either existed).

### Remove polling infrastructure (lines 7-20)

Current lines 7-20:
```rust
pub fn Status() -> Element {
  let mut action = use_action(get_run_status);
  let mut exec_state = use_signal(|| ExecutionState::Idle);
  let mut source_path = use_signal(|| "none".to_string());
  let mut elapsed = use_signal(|| 0u64);
  let mut cost = use_signal(|| 0.0f64);
  let mut error_msg: Signal<Option<String>> = use_signal(|| None);

  use_future(move || async move {
    loop {
      action.call();
      tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
  });
```

Replace with:
```rust
pub fn Status() -> Element {
  let status = use_loader(|| get_run_status())?;
```

This single line replaces:
- The `use_action` call (line 8)
- All 5 `use_signal` declarations (lines 9-13)
- The entire `use_future` polling loop (lines 15-20)

The `?` operator propagates `Loading::Pending` (suspends the component) and `Loading::Failed` (propagates the error up the tree) via the `From<Loading> for RenderError` impl.

### Replace result handling (lines 22-42)

Current lines 22-42:
```rust
  if let Some(Ok(status)) = action.value() {
    let status = status.read();
    let state = match status.status.as_str() {
      "running" => ExecutionState::Running,
      "completed" => ExecutionState::Done,
      "failed" => ExecutionState::Error,
      "waiting" => ExecutionState::Waiting,
      _ => ExecutionState::Idle,
    };
    exec_state.set(state);
    if let Some(ref path) = status.source_path {
      source_path.set(path.clone());
    }
    if let Some(ms) = status.elapsed_ms {
      elapsed.set(ms);
    }
    if let Some(c) = status.cost {
      cost.set(c);
    }
    error_msg.set(status.error.clone());
  }
```

Replace with direct reads from the `Loader<RunStatus>`. The `Loader<T>` implements `Readable`, so you access fields via `.read()`:

```rust
  let state = match status.read().status.as_str() {
    "running" => ExecutionState::Running,
    "completed" => ExecutionState::Done,
    "failed" => ExecutionState::Error,
    "waiting" => ExecutionState::Waiting,
    _ => ExecutionState::Idle,
  };
  let source_path = status.read().source_path.as_deref().unwrap_or("none");
  let elapsed = status.read().elapsed_ms.unwrap_or(0);
  let cost = status.read().cost.unwrap_or(0.0);
  let error = status.read().error.clone();
```

### Update RSX block (lines 44-55)

Current:
```rust
  rsx! {
    div { class: "flex flex-col items-center gap-6 pt-8",
      PulseIndicator { state: exec_state() }
      div { class: "text-center space-y-2",
        p { class: "text-sm text-[var(--on-surface-variant)]", "{source_path}" }
        p { class: "text-xs text-[var(--outline)]", "elapsed: {elapsed}ms | cost: ${cost:.4}" }
        if let Some(ref err) = *error_msg.read() {
          p { class: "text-xs text-[var(--error)]", "{err}" }
        }
      }
    }
  }
```

Replace with:
```rust
  rsx! {
    div { class: "flex flex-col items-center gap-6 pt-8",
      PulseIndicator { state }
      div { class: "text-center space-y-2",
        p { class: "text-sm text-[var(--on-surface-variant)]", "{source_path}" }
        p { class: "text-xs text-[var(--outline)]", "elapsed: {elapsed}ms | cost: ${cost:.4}" }
        if let Some(ref err) = error {
          p { class: "text-xs text-[var(--error)]", "{err}" }
        }
      }
    }
  }
```

Changes in the RSX:
- `exec_state()` becomes `state` (it's now a local variable, not a signal)
- `*error_msg.read()` becomes `error` (it's now a local `Option<String>`, not a signal)

### Complete final file

```rust
use dioxus::prelude::*;
use lx_api::run_api::get_run_status;

use crate::components::pulse_indicator::{ExecutionState, PulseIndicator};

#[component]
pub fn Status() -> Element {
  let status = use_loader(|| get_run_status())?;

  let state = match status.read().status.as_str() {
    "running" => ExecutionState::Running,
    "completed" => ExecutionState::Done,
    "failed" => ExecutionState::Error,
    "waiting" => ExecutionState::Waiting,
    _ => ExecutionState::Idle,
  };
  let source_path = status.read().source_path.as_deref().unwrap_or("none");
  let elapsed = status.read().elapsed_ms.unwrap_or(0);
  let cost = status.read().cost.unwrap_or(0.0);
  let error = status.read().error.clone();

  rsx! {
    div { class: "flex flex-col items-center gap-6 pt-8",
      PulseIndicator { state }
      div { class: "text-center space-y-2",
        p { class: "text-sm text-[var(--on-surface-variant)]", "{source_path}" }
        p { class: "text-xs text-[var(--outline)]", "elapsed: {elapsed}ms | cost: ${cost:.4}" }
        if let Some(ref err) = error {
          p { class: "text-xs text-[var(--error)]", "{err}" }
        }
      }
    }
  }
}
```

## Verification

After making the changes, run `just diagnose` and confirm no errors related to `status.rs` or `types.rs`.
