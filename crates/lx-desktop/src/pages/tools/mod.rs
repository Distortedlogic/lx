mod mcp_panel;
pub mod pi_page;

use dioxus::prelude::*;

use self::mcp_panel::McpPanel;
pub use self::pi_page::PiPage;

#[component]
pub fn Tools() -> Element {
  rsx! {
    div { class: "flex flex-col h-full gap-4 p-4 overflow-auto",
      div { class: "flex-between",
        h1 { class: "page-heading", "TOOLS" }
        Link {
          class: "btn-outline-sm",
          to: crate::routes::Route::PiPage {},
          "Open Pi Runtime"
        }
      }
      McpPanel {}
    }
  }
}

#[component]
pub fn PiAgentPage(agent_id: String) -> Element {
  rsx! {
    PiPage { agent_id: Some(agent_id) }
  }
}
