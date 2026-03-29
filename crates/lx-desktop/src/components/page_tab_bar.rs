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
    div { class: "flex border-b border-gray-700/50",
      for item in items.iter() {
        {
            let active_class = if item.value == current_value {
                "border-white text-white"
            } else {
                "border-transparent text-gray-400 hover:text-white hover:border-gray-500"
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
