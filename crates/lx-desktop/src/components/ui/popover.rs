use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Popover(open: Signal<bool>, children: Element) -> Element {
  rsx! {
    div { "data-slot": "popover", class: "relative inline-block", {children} }
  }
}

#[component]
pub fn PopoverTrigger(open: Signal<bool>, children: Element) -> Element {
  let mut open = open;
  rsx! {
    button {
      "data-slot": "popover-trigger",
      onclick: move |_| {
          let v = open();
          open.set(!v);
      },
      {children}
    }
  }
}

#[component]
pub fn PopoverContent(open: Signal<bool>, #[props(default)] class: String, children: Element) -> Element {
  let mut open = open;
  if !open() {
    return rsx! {};
  }
  rsx! {
    div {
      class: "fixed inset-0 z-40",
      onclick: move |_| open.set(false),
    }
    div {
      "data-slot": "popover-content",
      class: cn(
          &[
              "bg-popover text-popover-foreground z-50 w-72 rounded-md border p-4 shadow-md outline-hidden absolute top-full mt-1",
              &class,
          ],
      ),
      {children}
    }
  }
}
