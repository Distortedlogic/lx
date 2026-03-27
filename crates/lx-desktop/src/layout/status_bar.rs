use dioxus::prelude::*;

use crate::contexts::status_bar::{StatusBarState, StatusBarStateStoreExt as _};

#[component]
pub fn StatusBar() -> Element {
  let state = use_context::<Store<StatusBarState>>();
  use_future(move || async move {
    if let Ok(output) = tokio::process::Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output().await
      && output.status.success()
    {
      let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
      state.branch().set(branch);
    }
  });
  let pane_label = state.pane_label().cloned();
  let branch = state.branch().cloned();
  let line = state.line().cloned();
  let col = state.col().cloned();
  let encoding = state.encoding().cloned();
  let notif_count = state.notification_count().cloned();
  rsx! {
    div { class: "flex items-center justify-between h-6 px-3 bg-[var(--surface-container-lowest)] border-t-2 border-[var(--outline)] text-xs uppercase tracking-[0.05em] font-mono shrink-0",
      div { class: "flex items-center gap-3",
        span { class: "text-white font-bold", "{pane_label}" }
        span { class: "text-[var(--primary)]", "\u{25A0}" }
        span { class: "text-[var(--primary)]", "{branch}" }
        span { class: "text-[var(--outline)]", "Ln {line}, Col {col}" }
      }
      div { class: "flex items-center gap-3",
        span { class: "text-[var(--outline)]", "{encoding}" }
        span { class: "flex items-center gap-1 text-[var(--outline)]",
          span { class: "text-[var(--success)] text-[8px]", "\u{25CF}" }
          "Notifications ({notif_count})"
        }
      }
    }
  }
}
