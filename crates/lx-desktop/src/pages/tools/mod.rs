use dioxus::prelude::*;

#[component]
pub fn Tools() -> Element {
  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]",
        "TOOLS"
      }
    }
  }
}
