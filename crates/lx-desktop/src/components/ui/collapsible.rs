use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Collapsible(open: Signal<bool>, #[props(default)] class: String, children: Element) -> Element {
  let data_state = if open() { "open" } else { "closed" };
  rsx! {
    div {
      "data-slot": "collapsible",
      "data-state": data_state,
      class: cn(&[&class]),
      {children}
    }
  }
}

#[component]
pub fn CollapsibleTrigger(open: Signal<bool>, children: Element) -> Element {
  let mut open = open;
  rsx! {
    button {
      "data-slot": "collapsible-trigger",
      onclick: move |_| {
          let v = open();
          open.set(!v);
      },
      {children}
    }
  }
}

#[component]
pub fn CollapsibleContent(open: Signal<bool>, #[props(default)] class: String, children: Element) -> Element {
  if !open() {
    return rsx! {};
  }
  rsx! {
    div { "data-slot": "collapsible-content", class: cn(&[&class]), {children} }
  }
}
