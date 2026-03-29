use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Skeleton(#[props(default)] class: String) -> Element {
  let classes = cn(&["bg-accent/75 rounded-md animate-pulse", &class]);
  rsx! {
    div { "data-slot": "skeleton", class: "{classes}" }
  }
}
