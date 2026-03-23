use dioxus::prelude::*;

#[component]
pub fn Accounts() -> Element {
  rsx! {
    div { class: "flex items-center justify-center h-full text-[var(--on-surface)]",
      "Accounts"
    }
  }
}
