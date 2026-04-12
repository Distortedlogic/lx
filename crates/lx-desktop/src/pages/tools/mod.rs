mod mcp_panel;

use dioxus::prelude::*;

use self::mcp_panel::McpPanel;

#[component]
pub fn Tools() -> Element {
  rsx! {
    div { class: "flex flex-col h-full gap-4 p-4 overflow-auto",
      div { class: "flex-between",
        h1 { class: "page-heading", "TOOLS" }
      }
      McpPanel {}
    }
  }
}
