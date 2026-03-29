use dioxus::prelude::*;

use super::status_colors::{status_badge_class, status_label};

#[component]
pub fn StatusBadge(status: String) -> Element {
  let color_class = status_badge_class(&status);
  let label = status_label(&status);
  let cls = format!("inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium whitespace-nowrap shrink-0 {color_class}");

  rsx! {
    span { class: "{cls}", "{label}" }
  }
}
