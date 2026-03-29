use dioxus::prelude::*;

#[component]
pub fn Inbox() -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Inbox (stub)" }
  }
}
