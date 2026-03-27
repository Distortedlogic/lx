use dioxus::prelude::*;

use super::bottom_nav::BottomNav;
use crate::routes::Route;

#[component]
pub fn MobileShell() -> Element {
  rsx! {
    div { class: "min-h-screen bg-[var(--surface)] text-[var(--on-surface)] flex flex-col",
      main { class: "flex-1 overflow-auto p-4 pb-20",
        div { class: "flex items-center gap-2 mb-3",
          span { class: "text-xs text-[var(--outline)]", "lx mobile" }
        }
        Outlet::<Route> {}
      }
      BottomNav {}
    }
  }
}
