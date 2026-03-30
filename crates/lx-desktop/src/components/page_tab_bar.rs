use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PageTabItem {
  pub value: String,
  pub label: String,
}

#[component]
pub fn PageTabBar(
  items: Vec<PageTabItem>,
  #[props(optional)] value: Option<String>,
  #[props(optional)] on_value_change: Option<EventHandler<String>>,
) -> Element {
  let current_value = value.unwrap_or_default();

  rsx! {
    div { class: "flex border-b border-[var(--outline-variant)]/50",
      for item in items.iter() {
        {
            let active_class = if item.value == current_value {
                "border-[var(--on-surface)] text-[var(--on-surface)]"
            } else {
                "border-transparent text-[var(--on-surface-variant)] hover:text-[var(--on-surface)] hover:border-[var(--outline)]"
            };
            let item_value = item.value.clone();
            let handler = on_value_change;
            rsx! {
              button {
                class: "px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px {active_class}",
                onclick: move |_| {
                    if let Some(ref h) = handler {
                        h.call(item_value.clone());
                    }
                },
                "{item.label}"
              }
            }
        }
      }
    }
  }
}
