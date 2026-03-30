use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct FilterValue {
  pub key: String,
  pub label: String,
  pub value: String,
}

#[component]
pub fn FilterBar(filters: Vec<FilterValue>, on_remove: EventHandler<String>, on_clear: EventHandler<()>) -> Element {
  if filters.is_empty() {
    return rsx! {};
  }

  rsx! {
    div { class: "flex items-center gap-2 flex-wrap",
      for filter in filters.iter() {
        span { class: "inline-flex items-center gap-1 rounded-full bg-[var(--surface-container-high)] px-2.5 py-0.5 text-xs pr-1",
          span { class: "text-[var(--on-surface-variant)]", "{filter.label}:" }
          span { "{filter.value}" }
          button {
            class: "ml-1 rounded-full hover:bg-[var(--surface-bright)] p-0.5",
            onclick: {
                let key = filter.key.clone();
                move |_| on_remove.call(key.clone())
            },
            span { class: "material-symbols-outlined text-xs", "close" }
          }
        }
      }
      button {
        class: "text-xs text-[var(--on-surface-variant)] hover:text-[var(--on-surface)] px-2 py-1 transition-colors",
        onclick: move |_| on_clear.call(()),
        "Clear all"
      }
    }
  }
}
