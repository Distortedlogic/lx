use dioxus::prelude::*;
use lx_api::run_api::get_run_status;
use lx_api::types::RunState;

use crate::components::pulse_indicator::{ExecutionState, PulseIndicator};

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
