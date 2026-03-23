use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn Sidebar() -> Element {
  rsx! {
    nav { class: "bg-[var(--surface-container-lowest)] border-r-2 border-[var(--outline)] flex flex-col items-center w-16 shrink-0 overflow-hidden py-3 gap-1 h-full",
      div { class: "flex-1 flex flex-col gap-1 items-center",
        NavItem {
          to: Route::Agents {},
          label: "AGENTS",
          icon: "smart_toy",
        }
        NavItem {
          to: Route::Activity {},
          label: "ACTIVITY",
          icon: "pulse_alert",
        }
        NavItem {
          to: Route::Terminals {},
          label: "PANES",
          icon: "dashboard",
        }
        NavItem { to: Route::Repos {}, label: "REPOS", icon: "database" }
        NavItem {
          to: Route::Settings {},
          label: "SETTINGS",
          icon: "settings",
        }
        div { class: "flex-1" }
        NavItem {
          to: Route::Accounts {},
          label: "ACCOUNTS",
          icon: "account_circle",
        }
      }
    }
  }
}

#[component]
fn NavItem(to: Route, label: &'static str, icon: &'static str) -> Element {
  let cls = "flex flex-col items-center justify-center gap-0.5 px-1 py-2 w-12 overflow-hidden text-[var(--outline)] hover:bg-[var(--surface-container-high)] hover:text-[var(--on-surface)] transition-colors duration-150 cursor-pointer";
  rsx! {
    Link {
      to,
      active_class: "!text-[var(--warning)] border-l-4 border-[var(--warning)]",
      class: "{cls}",
      span { class: "material-symbols-outlined text-2xl", "{icon}" }
      span { class: "text-[8px] uppercase tracking-[0.05em] leading-tight", "{label}" }
    }
  }
}
