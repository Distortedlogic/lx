use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Label(#[props(default)] class: String, #[props(default)] r#for: String, children: Element) -> Element {
  let classes = cn(&["label", &class]);
  rsx! {
    label { "data-slot": "label", r#for: "{r#for}", class: "{classes}", {children} }
  }
}
