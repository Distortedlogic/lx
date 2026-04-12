use dioxus::prelude::*;

use super::ui::badge::{Badge, BadgeVariant};
use super::ui::button::{Button, ButtonSize, ButtonVariant};

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
        Badge {
          variant: BadgeVariant::Secondary,
          class: "pr-1 gap-1".to_string(),
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
      Button {
        variant: ButtonVariant::Ghost,
        size: ButtonSize::Xs,
        onclick: move |_| on_clear.call(()),
        "Clear all"
      }
    }
  }
}
