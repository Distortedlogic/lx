use dioxus::prelude::*;

#[component]
pub fn Voice() -> Element {
  rsx! {
    div { class: "flex items-center justify-center h-full text-[var(--on-surface)]", "Voice" }
  }
}
