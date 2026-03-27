use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn BottomNav() -> Element {
  rsx! {
    nav { class: "fixed bottom-0 left-0 right-0 bg-gray-800 border-t border-gray-700 flex items-center justify-around py-2 px-4 z-40",
      NavTab { to: Route::Status {}, label: "Status" }
      NavTab { to: Route::Events {}, label: "Events" }
      NavTab { to: Route::Approvals {}, label: "Approve" }
    }
  }
}

#[component]
fn NavTab(to: Route, label: &'static str) -> Element {
  rsx! {
    Link {
      to,
      class: "flex flex-col items-center gap-0.5 min-w-16 min-h-12 justify-center text-gray-400 active:text-blue-400",
      span { class: "text-[10px]", "{label}" }
    }
  }
}
