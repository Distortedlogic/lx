use dioxus::prelude::*;

#[component]
pub fn Repos() -> Element {
  rsx! {
    div { class: "flex items-center justify-center h-full text-[var(--on-surface)]", "Repos" }
  }
}
