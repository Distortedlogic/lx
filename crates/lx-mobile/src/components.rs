use dioxus::prelude::*;
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
pub fn PulseIndicator(state: ExecutionState) -> Element {
  let (color, animation, label) = match state {
    ExecutionState::Idle => ("bg-[var(--outline)]", "", "Ready"),
    ExecutionState::Running => ("bg-[var(--primary)]", "animate-[pulse_1.5s_infinite_ease-in-out]", "Running..."),
    ExecutionState::Waiting => ("bg-[var(--warning)]", "animate-pulse", "Waiting for input..."),
    ExecutionState::Done => ("bg-[var(--success)]", "", "Completed"),
    ExecutionState::Error => ("bg-[var(--error)]", "", "Error"),
  };
  rsx! {
    div { class: "flex flex-col items-center gap-2",
      div {
        class: "w-16 h-16 rounded-full opacity-90",
        class: "{color}",
        class: "{animation}",
      }
      span { class: "text-xs text-[var(--outline)] text-center", "{label}" }
    }
  }
}
