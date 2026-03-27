use dioxus::prelude::*;
use lx_api::run_api::get_run_status;

use crate::components::pulse_indicator::{ExecutionState, PulseIndicator};

#[component]
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
}
