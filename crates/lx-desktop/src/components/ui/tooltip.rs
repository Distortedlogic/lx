use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Tooltip(content: String, #[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div { "data-slot": "tooltip", class: "group relative inline-flex",
      {children}
      div {
        "data-slot": "tooltip-content",
        role: "tooltip",
        class: cn(
            &[
                "bg-foreground text-background pointer-events-none z-50 w-fit rounded-md px-3 py-1.5 text-xs text-balance absolute bottom-full left-1/2 -translate-x-1/2 mb-2 opacity-0 transition-opacity group-hover:opacity-100",
                &class,
            ],
        ),
        "{content}"
      }
    }
  }
}
