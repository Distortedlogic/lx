use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
  rsx! {
    div { class: "flex items-center justify-center h-full text-[var(--on-surface)]", "Dashboard" }
  }
}
