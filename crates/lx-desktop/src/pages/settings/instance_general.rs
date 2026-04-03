use dioxus::prelude::*;

#[component]
pub fn InstanceGeneral() -> Element {
  let mut censor_username = use_signal(|| false);

  rsx! {
    div { class: "max-w-4xl space-y-6 p-4 overflow-auto",
      div { class: "space-y-2",
        div { class: "flex items-center gap-2",
          span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
            "tune"
          }
          h1 { class: "text-lg font-semibold text-[var(--on-surface)]", "General" }
        }
        p { class: "text-sm text-[var(--outline)]",
          "Configure instance-wide defaults that affect how operator-visible logs are displayed."
        }
      }
      div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)] p-5",
        div { class: "flex items-start justify-between gap-4",
          div { class: "space-y-1.5",
            h2 { class: "text-sm font-semibold text-[var(--on-surface)]",
              "Censor username in logs"
            }
            p { class: "max-w-2xl text-sm text-[var(--outline)]",
              "Hide the username segment in home-directory paths and similar operator-visible log output."
            }
          }
          button {
            class: if censor_username() { "relative inline-flex h-5 w-9 items-center rounded-full bg-green-600 transition-colors" } else { "relative inline-flex h-5 w-9 items-center rounded-full bg-[var(--surface-container)] transition-colors" },
            onclick: move |_| {
                let current = censor_username();
                censor_username.set(!current);
            },
            span { class: if censor_username() { "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform translate-x-4" } else { "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform translate-x-0.5" } }
          }
        }
      }
    }
  }
}
