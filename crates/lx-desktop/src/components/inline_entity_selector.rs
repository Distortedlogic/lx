use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct InlineEntityOption {
  pub id: String,
  pub label: String,
}

#[component]
pub fn InlineEntitySelector(
  value: String,
  options: Vec<InlineEntityOption>,
  #[props(default = "Select...".to_string())] placeholder: String,
  on_change: EventHandler<String>,
  #[props(optional)] class: Option<String>,
) -> Element {
  let mut open = use_signal(|| false);
  let mut query = use_signal(String::new);
  let extra = class.as_deref().unwrap_or("");

  let q = query().to_lowercase();
  let filtered: Vec<&InlineEntityOption> = options.iter().filter(|o| o.label.to_lowercase().contains(&q)).collect();

  let current_option = options.iter().find(|o| o.id == value);

  rsx! {
    div { class: "relative inline-block",
      button {
        class: "inline-flex min-w-0 items-center gap-1 rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container)]/40 px-2 py-1 text-sm font-medium transition-colors hover:bg-[var(--on-surface)]/5 {extra}",
        onclick: move |_| open.set(!open()),
        if let Some(opt) = current_option {
          "{opt.label}"
        } else {
          span { class: "text-[var(--on-surface-variant)]", "{placeholder}" }
        }
      }
      if open() {
        div {
          class: "fixed inset-0 z-40",
          onclick: move |_| {
              open.set(false);
              query.set(String::new());
          },
        }
        div { class: "absolute z-50 mt-1 w-[min(20rem,calc(100vw-2rem))] rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container-high)] shadow-lg",
          input {
            class: "w-full border-b border-[var(--outline-variant)] bg-transparent px-2 py-1.5 text-sm outline-none placeholder:text-[var(--outline)]",
            placeholder: "Search...",
            oninput: move |evt: Event<FormData>| query.set(evt.value()),
          }
          div { class: "max-h-56 overflow-y-auto py-1",
            for option in filtered.iter() {
              {
                  let opt_id = option.id.clone();
                  let opt_label = option.label.clone();
                  let is_selected = option.id == value;
                  rsx! {
                    button {
                      class: "flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm hover:bg-[var(--on-surface)]/5",
                      onclick: move |_| {
                          on_change.call(opt_id.clone());
                          open.set(false);
                          query.set(String::new());
                      },
                      span { class: "truncate", "{opt_label}" }
                      if is_selected {
                        span { class: "material-symbols-outlined text-sm text-[var(--on-surface-variant)] ml-auto",
                          "check"
                        }
                      }
                    }
                  }
              }
            }
            if filtered.is_empty() {
              p { class: "px-2 py-2 text-xs text-[var(--on-surface-variant)]",
                "No results."
              }
            }
          }
        }
      }
    }
  }
}
