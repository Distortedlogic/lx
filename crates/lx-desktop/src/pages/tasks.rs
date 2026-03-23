use dioxus::prelude::*;

#[component]
pub fn Tasks() -> Element {
  rsx! {
    div { class: "flex items-center justify-center h-full text-[var(--on-surface)]", "Tasks" }
  }
}
