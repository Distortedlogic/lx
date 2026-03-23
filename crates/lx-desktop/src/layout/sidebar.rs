use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn Sidebar() -> Element {
  rsx! {
    nav { class: "bg-[var(--surface-container-low)] flex flex-col items-center w-16 py-3 gap-1 h-full",
      div { class: "flex-1 flex flex-col gap-1 items-center",
        NavItem { to: Route::Agents {}, label: "AGENTS", icon: "\u{2B21}" }
        NavItem {
          to: Route::Activity {},
          label: "ACTIVITY",
          icon: "\u{25C8}",
        }
        NavItem {
          to: Route::Terminals {},
          label: "TERMINALS",
          icon: "\u{25A3}",
        }
        NavItem { to: Route::Repos {}, label: "REPOS", icon: "\u{229F}" }
        NavItem {
          to: Route::Settings {},
          label: "SETTINGS",
          icon: "\u{2699}",
        }
        div { class: "flex-1" }
        NavItem {
          to: Route::Accounts {},
          label: "ACCOUNTS",
          icon: "\u{1F464}",
        }
      }
    }
  }
}

#[component]
fn NavItem(to: Route, label: &'static str, icon: &'static str) -> Element {
  let cls = "flex flex-col items-center justify-center gap-0.5 px-1 py-2 rounded-lg w-12 text-[var(--outline)] hover:bg-[var(--surface-container-high)] hover:text-[var(--on-surface)] transition-colors duration-150 cursor-pointer";
  rsx! {
    Link {
      to,
      active_class: "!text-[var(--primary)] bg-[var(--surface-container-high)]",
      class: "{cls}",
      span { class: "text-base", "{icon}" }
      span { class: "text-[8px] uppercase tracking-[0.05em] leading-tight", "{label}" }
    }
  }
}
