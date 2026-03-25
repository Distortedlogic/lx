mod mcp_panel;

use dioxus::prelude::*;

use self::mcp_panel::McpPanel;

#[component]
pub fn Tools() -> Element {
  rsx! {
    div { class: "flex flex-col h-full gap-4 p-4 overflow-auto",
      div { class: "flex items-center justify-between",
        h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]",
          "TOOLS"
        }
      }
      McpPanel {}
    }
  }
}
