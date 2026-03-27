---
unit: 2
title: Inline PulseIndicator into Status
type: fix
depends_on: unit 1
rule: Component Design — single-use component
---

## Goal

Inline the single-use `PulseIndicator` component into `status.rs` and remove the now-empty `components.rs` module.

## Preconditions

- `PulseIndicator` is defined in `src/components.rs:13-32`
- `ExecutionState` is defined in `src/components.rs:4-11`
- Only `src/pages/status.rs` imports from `components.rs` (verified via `rg 'use crate::components' crates/lx-mobile/`)
- `status.rs` is currently 36 lines; after inlining it will be ~59 lines (well under 300)

## Changes

### File 1: `crates/lx-mobile/src/pages/status.rs` — replace entire contents

Replace the entire file with this exact content:

```rust
use dioxus::prelude::*;
use lx_api::run_api::get_run_status;
use lx_api::types::RunState;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionState {
  Idle,
  Running,
  Waiting,
  Done,
  Error,
}

#[component]
pub fn Status() -> Element {
  let status = use_loader(get_run_status)?;

  let status_ref = status.read();
  let state = match status_ref.status {
    RunState::Running => ExecutionState::Running,
    RunState::Completed => ExecutionState::Done,
    RunState::Failed => ExecutionState::Error,
    RunState::Waiting => ExecutionState::Waiting,
    RunState::Idle => ExecutionState::Idle,
  };
  let source_path = status_ref.source_path.as_deref().unwrap_or("none");
  let elapsed = status_ref.elapsed_ms.unwrap_or(0);
  let cost = status_ref.cost.unwrap_or(0.0);
  let error = status_ref.error.clone();

  let (color, animation, label) = match state {
    ExecutionState::Idle => ("bg-[var(--outline)]", "", "Ready"),
    ExecutionState::Running => ("bg-[var(--primary)]", "animate-[pulse_1.5s_infinite_ease-in-out]", "Running..."),
    ExecutionState::Waiting => ("bg-[var(--warning)]", "animate-pulse", "Waiting for input..."),
    ExecutionState::Done => ("bg-[var(--success)]", "", "Completed"),
    ExecutionState::Error => ("bg-[var(--error)]", "", "Error"),
  };

  rsx! {
    div { class: "flex flex-col items-center gap-6 pt-8",
      div { class: "flex flex-col items-center gap-2",
        div {
          class: "w-16 h-16 rounded-full opacity-90",
          class: "{color}",
          class: "{animation}",
        }
        span { class: "text-xs text-[var(--outline)] text-center", "{label}" }
      }
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

Note: The PulseIndicator RSX is inlined directly into the Status component's `rsx!` block rather than using a nested `rsx!` call inside a block expression. The `let (color, animation, label)` destructuring is hoisted into the function body alongside the existing `let state` binding.

### File 2: `crates/lx-mobile/src/components.rs` — delete

Delete the entire file. Both `ExecutionState` and `PulseIndicator` are now in `status.rs`.

### File 3: `crates/lx-mobile/src/main.rs` — remove mod declaration

Replace the entire file with:

```rust
mod app;
mod layout;
mod pages;
mod routes;

fn main() {
  dioxus::fullstack::set_server_url("http://127.0.0.1:8080");
  dioxus::launch(app::App);
}
```

The only change is removing `mod components;` (was line 2 in the original).

## Verification

After applying all three changes, run `just diagnose` to confirm the crate compiles without errors or warnings.

## Result

- `status.rs`: 59 lines, self-contained with `ExecutionState` enum and inline pulse indicator RSX
- `components.rs`: deleted
- `main.rs`: 9 lines, `mod components;` removed
- No other files affected (verified: no other file imports from `components`)
