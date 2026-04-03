use dioxus::prelude::*;

#[component]
fn ToggleSection(title: String, description: String, enabled: bool, on_toggle: EventHandler<bool>) -> Element {
  rsx! {
    div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)] p-5",
      div { class: "flex items-start justify-between gap-4",
        div { class: "space-y-1.5",
          h2 { class: "text-sm font-semibold text-[var(--on-surface)]", "{title}" }
          p { class: "max-w-2xl text-sm text-[var(--outline)]", "{description}" }
        }
        button {
          class: if enabled { "relative inline-flex h-5 w-9 items-center rounded-full bg-green-600 transition-colors" } else { "relative inline-flex h-5 w-9 items-center rounded-full bg-[var(--surface-container)] transition-colors" },
          onclick: move |_| on_toggle.call(!enabled),
          span { class: if enabled { "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform translate-x-4" } else { "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform translate-x-0.5" } }
        }
      }
    }
  }
}

#[component]
pub fn InstanceExperimental() -> Element {
  let mut isolated_workspaces = use_signal(|| false);
  let mut auto_restart = use_signal(|| false);

  rsx! {
    div { class: "max-w-4xl space-y-6 p-4 overflow-auto",
      div { class: "space-y-2",
        div { class: "flex items-center gap-2",
          span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
            "science"
          }
          h1 { class: "text-lg font-semibold text-[var(--on-surface)]", "Experimental" }
        }
        p { class: "text-sm text-[var(--outline)]",
          "Opt into features that are still being evaluated before they become default behavior."
        }
      }
      ToggleSection {
        title: "Enable Isolated Workspaces",
        description: "Show execution workspace controls in project configuration and allow isolated workspace behavior for new and existing issue runs.",
        enabled: isolated_workspaces(),
        on_toggle: move |v| isolated_workspaces.set(v),
      }
      ToggleSection {
        title: "Auto-Restart Dev Server When Idle",
        description: "Wait for all queued and running local agent runs to finish, then restart the server automatically when backend changes make the current boot stale.",
        enabled: auto_restart(),
        on_toggle: move |v| auto_restart.set(v),
      }
    }
  }
}
