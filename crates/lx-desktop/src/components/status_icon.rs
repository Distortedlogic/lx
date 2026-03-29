use dioxus::prelude::*;

use super::status_colors::issue_status_icon_class;

#[component]
pub fn StatusIcon(status: String, #[props(optional)] class: Option<String>) -> Element {
  let color_class = issue_status_icon_class(&status);
  let extra = class.as_deref().unwrap_or("");
  let cls = format!("relative inline-flex h-4 w-4 rounded-full border-2 shrink-0 {color_class} {extra}");

  rsx! {
    span { class: "{cls}",
      if status == "done" {
        span { class: "absolute inset-0 m-auto h-2 w-2 rounded-full bg-current" }
      }
    }
  }
}
