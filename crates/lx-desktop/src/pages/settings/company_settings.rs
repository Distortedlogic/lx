use dioxus::prelude::*;

#[component]
pub fn CompanySettings() -> Element {
  let mut company_name = use_signal(|| "Default Company".to_string());
  let mut description = use_signal(String::new);
  let mut brand_color = use_signal(|| "#6366f1".to_string());
  let mut require_approval = use_signal(|| false);
  let mut invite_snippet: Signal<Option<String>> = use_signal(|| None);

  let general_dirty = true;

  rsx! {
    div { class: "max-w-2xl space-y-6 p-4 overflow-auto",
      div { class: "flex items-center gap-2",
        span { class: "material-symbols-outlined text-[var(--outline)]", "settings" }
        h1 { class: "text-lg font-semibold text-[var(--on-surface)]", "Company Settings" }
      }

      div { class: "space-y-4",
        div { class: "text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
          "General"
        }
        div { class: "space-y-3 rounded-md border border-[var(--outline-variant)] px-4 py-4",
          div { class: "space-y-1",
            label { class: "text-xs font-medium text-[var(--on-surface)]",
              "Company name"
            }
            input {
              class: "w-full rounded-md border border-[var(--outline-variant)] bg-transparent px-2.5 py-1.5 text-sm outline-none text-[var(--on-surface)]",
              r#type: "text",
              value: "{company_name}",
              oninput: move |evt| company_name.set(evt.value()),
            }
          }
          div { class: "space-y-1",
            label { class: "text-xs font-medium text-[var(--on-surface)]",
              "Description"
            }
            input {
              class: "w-full rounded-md border border-[var(--outline-variant)] bg-transparent px-2.5 py-1.5 text-sm outline-none text-[var(--on-surface)]",
              r#type: "text",
              value: "{description}",
              placeholder: "Optional company description",
              oninput: move |evt| description.set(evt.value()),
            }
          }
        }
      }

      div { class: "space-y-4",
        div { class: "text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
          "Appearance"
        }
        div { class: "space-y-3 rounded-md border border-[var(--outline-variant)] px-4 py-4",
          div { class: "space-y-1",
            label { class: "text-xs font-medium text-[var(--on-surface)]",
              "Brand color"
            }
            div { class: "flex items-center gap-2",
              input {
                r#type: "color",
                value: "{brand_color}",
                class: "h-8 w-8 cursor-pointer rounded border border-[var(--outline-variant)] bg-transparent p-0",
                oninput: move |evt| brand_color.set(evt.value()),
              }
              input {
                r#type: "text",
                value: "{brand_color}",
                class: "w-28 rounded-md border border-[var(--outline-variant)] bg-transparent px-2.5 py-1.5 text-sm font-mono outline-none text-[var(--on-surface)]",
                oninput: move |evt| brand_color.set(evt.value()),
              }
            }
          }
        }
      }

      if general_dirty {
        div { class: "flex items-center gap-2",
          button { class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-1.5 text-xs font-semibold",
            "Save changes"
          }
        }
      }

      div { class: "space-y-4",
        div { class: "text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
          "Hiring"
        }
        div { class: "rounded-md border border-[var(--outline-variant)] px-4 py-3",
          div { class: "flex items-center justify-between",
            div {
              span { class: "text-sm text-[var(--on-surface)]",
                "Require board approval for new hires"
              }
            }
            button {
              class: if require_approval() { "relative inline-flex h-5 w-9 items-center rounded-full bg-green-600" } else { "relative inline-flex h-5 w-9 items-center rounded-full bg-[var(--surface-container)]" },
              onclick: move |_| {
                  let current = require_approval();
                  require_approval.set(!current);
              },
              span { class: if require_approval() { "inline-block h-3.5 w-3.5 rounded-full bg-white translate-x-4" } else { "inline-block h-3.5 w-3.5 rounded-full bg-white translate-x-0.5" } }
            }
          }
        }
      }

      div { class: "space-y-4",
        div { class: "text-xs font-medium text-[var(--outline)] uppercase tracking-wide",
          "Invites"
        }
        div { class: "space-y-3 rounded-md border border-[var(--outline-variant)] px-4 py-4",
          p { class: "text-xs text-[var(--outline)]", "Generate an agent invite snippet." }
          button {
            class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-1.5 text-xs font-semibold",
            onclick: move |_| {
                invite_snippet.set(Some("Invite snippet placeholder".to_string()));
            },
            "Generate Invite Prompt"
          }
          if let Some(ref snippet) = invite_snippet() {
            div { class: "rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container)]/30 p-2",
              textarea {
                class: "h-48 w-full rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)] px-2 py-1.5 font-mono text-xs outline-none text-[var(--on-surface)]",
                readonly: true,
                value: "{snippet}",
              }
            }
          }
        }
      }

      div { class: "space-y-4",
        div { class: "text-xs font-medium text-red-500 uppercase tracking-wide",
          "Danger Zone"
        }
        div { class: "space-y-3 rounded-md border border-red-500/40 bg-red-500/5 px-4 py-4",
          p { class: "text-sm text-[var(--outline)]",
            "Archive this company to hide it from the sidebar."
          }
          button { class: "bg-red-600 text-white rounded px-4 py-1.5 text-xs font-semibold hover:bg-red-500",
            "Archive company"
          }
        }
      }
    }
  }
}
