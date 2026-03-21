use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn Sidebar(collapsed: Signal<bool>) -> Element {
  rsx! {
      nav {
          class: "bg-gray-800 flex flex-col transition-all duration-200",
          class: if collapsed() { "w-12 px-1 py-4" } else { "w-56 p-4" },
          if !collapsed() {
              div { class: "mb-6",
                  h1 { class: "text-xl font-bold text-blue-400", "lx" }
              }
          }
          div { class: "flex-1 flex flex-col gap-1",
              NavItem { to: Route::Run {}, label: "Run", collapsed: collapsed() }
              NavItem { to: Route::Terminals {}, label: "Terminals", collapsed: collapsed() }
              NavItem { to: Route::Events {}, label: "Events", collapsed: collapsed() }
          }
          button {
              class: "mt-4 text-gray-500 hover:text-gray-300 text-sm",
              onclick: move |_| collapsed.set(!collapsed()),
              if collapsed() { ">" } else { "<" }
          }
      }
  }
}

#[component]
fn NavItem(to: Route, label: &'static str, collapsed: bool) -> Element {
  rsx! {
      Link {
          to,
          class: "block px-3 py-2 rounded text-sm text-gray-300 hover:bg-gray-700 hover:text-white",
          if collapsed {
              "{&label[..1]}"
          } else {
              "{label}"
          }
      }
  }
}
