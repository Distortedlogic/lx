use dioxus::prelude::*;

const MENU_ITEMS: &[&str] = &["FILE", "EDIT", "SELECTION", "VIEW", "GO", "RUN", "TERMINAL", "HELP"];

#[component]
pub fn MenuBar() -> Element {
  rsx! {
    div {
      class: "flex items-center h-10 bg-[var(--surface-container-lowest)] border-b-2 border-[var(--outline)] text-xs uppercase tracking-wider shrink-0 select-none",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")] dioxus::desktop::window().drag();
      },
      span {
        class: "px-3 font-bold text-[var(--primary)] font-[var(--font-display)]",
        onmousedown: |evt| evt.stop_propagation(),
        "TERMINAL_MONOLITH"
      }
      div {
        class: "flex items-center gap-0.5",
        onmousedown: |evt| evt.stop_propagation(),
        for item in MENU_ITEMS {
          span { class: if *item == "RUN" { "px-2 py-1 rounded cursor-pointer text-[var(--primary)] hover:bg-[var(--surface-container-high)] transition-colors duration-150" } else { "px-2 py-1 rounded cursor-pointer text-[var(--on-surface-variant)] hover:text-[var(--primary)] hover:bg-[var(--surface-container-high)] transition-colors duration-150" },
            "{item}"
          }
        }
      }
      div { class: "flex-1" }
      div {
        class: "flex items-center",
        onmousedown: |evt| evt.stop_propagation(),
        button {
          class: "px-3 py-1 hover:bg-[var(--surface-container-high)] text-[var(--on-surface-variant)] transition-colors duration-150",
          onclick: move |_| {
              #[cfg(feature = "desktop")] dioxus::desktop::window().set_minimized(true);
          },
          span { class: "material-symbols-outlined text-sm", "remove" }
        }
        button {
          class: "px-3 py-1 hover:bg-[var(--surface-container-high)] text-[var(--on-surface-variant)] transition-colors duration-150",
          onclick: move |_| {
              #[cfg(feature = "desktop")] dioxus::desktop::window().toggle_maximized();
          },
          span { class: "material-symbols-outlined text-sm", "content_copy" }
        }
        button {
          class: "px-3 py-1 hover:bg-[var(--error)]/80 text-[var(--on-surface-variant)] hover:text-white transition-colors duration-150",
          onclick: move |_| {
              #[cfg(feature = "desktop")] dioxus::desktop::window().close();
          },
          span { class: "material-symbols-outlined text-sm", "close" }
        }
      }
    }
  }
}
