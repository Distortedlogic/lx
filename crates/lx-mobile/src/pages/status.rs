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
