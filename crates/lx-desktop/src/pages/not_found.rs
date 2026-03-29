use dioxus::prelude::*;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
  rsx! {
    div { class: "p-4 text-sm text-destructive", "404 -- Page not found" }
  }
}
