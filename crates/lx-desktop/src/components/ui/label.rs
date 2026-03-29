use dioxus::prelude::*;

use super::cn;

const BASE_LABEL_CLASS: &str = "flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50";

#[component]
pub fn Label(#[props(default)] class: String, #[props(default)] r#for: String, children: Element) -> Element {
  let classes = cn(&[BASE_LABEL_CLASS, &class]);
  rsx! {
    label { "data-slot": "label", r#for: "{r#for}", class: "{classes}", {children} }
  }
}
