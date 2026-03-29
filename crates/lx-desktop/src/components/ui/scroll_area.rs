use dioxus::prelude::*;

use super::cn;

#[component]
pub fn ScrollArea(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "scroll-area",
      class: cn(&["relative flex flex-col overflow-hidden", &class]),
      div {
        "data-slot": "scroll-area-viewport",
        class: "flex-1 min-h-0 w-full rounded-[inherit] overflow-y-auto overflow-x-hidden",
        {children}
      }
    }
  }
}
