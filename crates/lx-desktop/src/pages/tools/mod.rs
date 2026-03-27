mod mcp_panel;

use crate::styles::{FLEX_BETWEEN, PAGE_HEADING};
use dioxus::prelude::*;

use self::mcp_panel::McpPanel;

#[component]
pub fn Tools() -> Element {
  rsx! {
    div { class: "flex flex-col h-full gap-4 p-4 overflow-auto",
      div { class: FLEX_BETWEEN,
        h1 { class: PAGE_HEADING, "TOOLS" }
      }
      McpPanel {}
    }
  }
}
