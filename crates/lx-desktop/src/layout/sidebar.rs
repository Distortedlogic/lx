use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn Sidebar() -> Element {
  rsx! {
    aside { class: "w-60 h-full min-h-0 border-r border-gray-700/50 bg-[var(--surface-container-lowest)] flex flex-col",
      div { class: "flex items-center gap-1 px-3 h-12 shrink-0",
        span { class: "flex-1 text-sm font-bold text-white truncate pl-1", "lx workspace" }
      }
      nav { class: "flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 px-3 py-2",
        div { class: "flex flex-col gap-0.5",
          SidebarNavItem {
            to: Route::Agents {},
            label: "Agents",
            icon: "smart_toy",
          }
          SidebarNavItem {
            to: Route::Activity {},
            label: "Activity",
            icon: "pulse_alert",
          }
        }
        SidebarSection { label: "System",
          SidebarNavItem { to: Route::Tools {}, label: "Tools", icon: "build" }
          SidebarNavItem {
            to: Route::Settings {},
            label: "Settings",
            icon: "settings",
          }
        }
        SidebarSection { label: "Account",
          SidebarNavItem {
            to: Route::Accounts {},
            label: "Accounts",
            icon: "account_circle",
          }
        }
      }
    }
  }
}

#[component]
fn SidebarSection(label: &'static str, children: Element) -> Element {
  rsx! {
    div {
      div { class: "px-3 py-1.5 text-[10px] font-medium uppercase tracking-widest font-mono text-gray-500",
        "{label}"
      }
      div { class: "flex flex-col gap-0.5 mt-0.5", {children} }
    }
  }
}

#[component]
fn SidebarNavItem(to: Route, label: &'static str, icon: &'static str) -> Element {
  rsx! {
    Link {
      to,
      active_class: "bg-white/10 text-white",
      class: "flex items-center gap-2.5 px-3 py-2 text-[13px] font-medium transition-colors text-gray-400 hover:bg-white/5 hover:text-white",
      span { class: "material-symbols-outlined text-base", "{icon}" }
      span { class: "flex-1 truncate", "{label}" }
    }
  }
}
