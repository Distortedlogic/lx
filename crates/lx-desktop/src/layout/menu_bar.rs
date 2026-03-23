use dioxus::prelude::*;

const MENU_ITEMS: &[&str] = &["File", "Edit", "Selection", "View", "Go", "Run", "Terminal", "Help"];

#[component]
pub fn MenuBar() -> Element {
  rsx! {
    div {
      class: "flex items-center h-8 bg-[var(--surface-container-low)] text-xs uppercase tracking-wider shrink-0 select-none",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")]
          dioxus::desktop::window().drag();
      },
      span {
        class: "px-3 font-bold text-[var(--primary)] font-[var(--font-display)]",
        onmousedown: |evt| evt.stop_propagation(),
        "lx"
      }
      div {
        class: "flex items-center gap-0.5",
        onmousedown: |evt| evt.stop_propagation(),
        for item in MENU_ITEMS {
          span {
            class: "px-2 py-1 rounded cursor-pointer text-[var(--on-surface-variant)] hover:text-[var(--primary)] hover:bg-[var(--surface-container-high)] transition-colors duration-150",
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
              #[cfg(feature = "desktop")]
              dioxus::desktop::window().set_minimized(true);
          },
          "\u{2212}"
        }
        button {
          class: "px-3 py-1 hover:bg-[var(--surface-container-high)] text-[var(--on-surface-variant)] transition-colors duration-150",
          onclick: move |_| {
              #[cfg(feature = "desktop")]
              dioxus::desktop::window().toggle_maximized();
          },
          "\u{25A1}"
        }
        button {
          class: "px-3 py-1 hover:bg-[var(--error)]/80 text-[var(--on-surface-variant)] hover:text-white transition-colors duration-150",
          onclick: move |_| {
              #[cfg(feature = "desktop")]
              dioxus::desktop::window().close();
          },
          "\u{00D7}"
        }
      }
    }
  }
}
