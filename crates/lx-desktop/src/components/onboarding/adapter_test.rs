use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdapterTestState {
  Idle,
  Testing,
  Pass(String),
  Fail(String),
}

#[component]
pub fn AdapterTestButton(adapter: Signal<String>, test_state: Signal<AdapterTestState>) -> Element {
  let state = test_state.read().clone();
  let adapter_val = adapter.read().clone();
  rsx! {
    div { class: "mt-4 flex items-center gap-3",
      button {
        class: "px-3 py-1.5 text-xs font-medium border border-[var(--outline-variant)] text-[var(--on-surface)] hover:bg-[var(--surface-container-highest)] transition-colors",
        disabled: matches!(state, AdapterTestState::Testing),
        onclick: move |_| {
            let adapter_key = adapter_val.clone();
            test_state.set(AdapterTestState::Testing);
            spawn(async move {
                let result = test_adapter_env(&adapter_key);
                test_state.set(result);
            });
        },
        if matches!(state, AdapterTestState::Testing) {
          span { class: "material-symbols-outlined text-sm animate-spin mr-1",
            "progress_activity"
          }
          "Testing..."
        } else {
          span { class: "material-symbols-outlined text-sm mr-1", "science" }
          "Test Adapter"
        }
      }
      match state {
          AdapterTestState::Pass(ref msg) => rsx! {
            span { class: "text-xs text-[var(--success)] flex items-center gap-1",
              span { class: "material-symbols-outlined text-sm", "check_circle" }
              "{msg}"
            }
          },
          AdapterTestState::Fail(ref msg) => rsx! {
            span { class: "text-xs text-[var(--error)] flex items-center gap-1",
              span { class: "material-symbols-outlined text-sm", "error" }
              "{msg}"
            }
          },
          _ => rsx! {},
      }
    }
  }
}

fn test_adapter_env(adapter: &str) -> AdapterTestState {
  match adapter {
    "claude_local" => {
      if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        AdapterTestState::Pass("ANTHROPIC_API_KEY found".into())
      } else {
        AdapterTestState::Fail("ANTHROPIC_API_KEY not set".into())
      }
    },
    "gemini_local" => {
      if std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GOOGLE_API_KEY").is_ok() {
        AdapterTestState::Pass("Gemini API key found".into())
      } else {
        AdapterTestState::Fail("GEMINI_API_KEY / GOOGLE_API_KEY not set".into())
      }
    },
    "codex_local" => {
      if cli_exists("codex") {
        AdapterTestState::Pass("codex CLI found".into())
      } else {
        AdapterTestState::Fail("codex CLI not found in PATH".into())
      }
    },
    "cursor" => {
      if cli_exists("cursor") {
        AdapterTestState::Pass("cursor CLI found".into())
      } else {
        AdapterTestState::Fail("cursor CLI not found in PATH".into())
      }
    },
    "http" => AdapterTestState::Pass("HTTP adapter requires no local config".into()),
    "process" => AdapterTestState::Pass("Process adapter requires no local config".into()),
    _ => {
      let bin = adapter.strip_suffix("_local").or_else(|| adapter.strip_suffix("_gateway")).unwrap_or(adapter);
      if cli_exists(bin) { AdapterTestState::Pass(format!("{bin} CLI found")) } else { AdapterTestState::Fail(format!("{bin} CLI not found in PATH")) }
    },
  }
}

fn cli_exists(name: &str) -> bool {
  std::process::Command::new(name).arg("--version").output().is_ok()
}
