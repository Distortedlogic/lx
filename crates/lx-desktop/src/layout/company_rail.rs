use dioxus::prelude::*;

#[component]
pub fn CompanyRail() -> Element {
  rsx! {
    div { class: "flex flex-col items-center w-[72px] shrink-0 h-full bg-[var(--surface-container-lowest)] border-r border-gray-700/50",
      div { class: "flex items-center justify-center h-12 w-full shrink-0",
        span { class: "text-xl font-bold text-[var(--primary)]", "lx" }
      }
      div { class: "flex-1" }
      div { class: "w-8 h-px bg-gray-700/50 mx-auto shrink-0" }
      div { class: "flex items-center justify-center py-2 shrink-0",
        span { class: "text-xs text-gray-500", "v0.1" }
      }
    }
  }
}
