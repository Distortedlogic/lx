use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn Sidebar() -> Element {
  rsx! {
    aside { class: "w-60 h-full min-h-0 border-r border-[var(--sidebar-border)]/50 bg-[var(--sidebar-background)] flex flex-col",
      div { class: "flex items-center gap-1 px-3 h-12 shrink-0",
        span { class: "flex-1 text-sm font-bold text-[var(--sidebar-foreground)] truncate pl-1",
          "lx workspace"
        }
      }
      nav { class: "flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 px-3 py-2",
        div { class: "flex flex-col gap-0.5",
          SidebarNavItem {
            to: Route::Dashboard {},
            label: "Dashboard",
            icon: "dashboard",
          }
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
          SidebarNavItem {
            to: Route::Issues {},
            label: "Issues",
            icon: "task_alt",
          }
          SidebarNavItem { to: Route::Inbox {}, label: "Inbox", icon: "inbox" }
          SidebarNavItem {
            to: Route::Costs {},
            label: "Costs",
            icon: "payments",
          }
          SidebarNavItem {
            to: Route::Approvals {},
            label: "Approvals",
            icon: "verified_user",
          }
        }
        SidebarSection { label: "System",
          SidebarNavItem { to: Route::Tools {}, label: "Tools", icon: "build" }
          SidebarNavItem {
            to: Route::Projects {},
            label: "Projects",
            icon: "hexagon",
          }
          SidebarNavItem { to: Route::Goals {}, label: "Goals", icon: "target" }
          SidebarNavItem {
            to: Route::Routines {},
            label: "Routines",
            icon: "repeat",
          }
          SidebarNavItem {
            to: Route::OrgChart {},
            label: "Org",
            icon: "account_tree",
          }
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
      div { class: "px-3 py-1.5 text-[10px] font-medium uppercase tracking-widest font-mono text-[var(--outline)]",
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
      active_class: "bg-[var(--sidebar-accent)] text-[var(--sidebar-foreground)]",
      class: "flex items-center gap-2.5 px-3 py-2 text-[13px] font-medium transition-colors text-[var(--sidebar-foreground)]/70 hover:bg-[var(--sidebar-accent)] hover:text-[var(--sidebar-foreground)]",
      span { class: "material-symbols-outlined text-sm", "{icon}" }
      span { class: "flex-1 truncate", "{label}" }
    }
  }
}
