use dioxus::prelude::*;

use super::bottom_nav::BottomNav;
use crate::api_client::LxClient;
use crate::routes::Route;

#[component]
pub fn MobileShell() -> Element {
  let client = use_context_provider(|| {
    let url = std::env::var("LX_DESKTOP_URL").unwrap_or_else(|_| "http://localhost:3030".into());
    Signal::new(LxClient::new(&url))
  });
  let _client_ref = client.read();

  rsx! {
      div { class: "min-h-screen bg-gray-900 text-gray-100 flex flex-col",
          main { class: "flex-1 overflow-auto p-4 pb-20",
              div { class: "flex items-center gap-2 mb-3",
                  span { class: "text-xs text-gray-400", "lx mobile" }
              }
              Outlet::<Route> {}
          }
          BottomNav {}
      }
  }
}
