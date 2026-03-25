use dioxus::prelude::*;

fn load_mcp_server_names() -> Vec<String> {
  let Ok(content) = std::fs::read_to_string(".mcp.json") else {
    return Vec::new();
  };
  let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
    return Vec::new();
  };
  let Some(servers) = json.get("mcpServers").and_then(|v| v.as_object()) else {
    return Vec::new();
  };
  servers.keys().cloned().collect()
}

#[component]
pub fn McpPanel() -> Element {
  let servers = use_hook(load_mcp_server_names);

  rsx! {
    div { class: "flex items-center gap-3 mb-4",
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
        "MCP_EXTENSIONS"
      }
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
    }
    div { class: "grid grid-cols-4 gap-3",
      for name in &servers {
        div { class: "bg-[var(--surface-container-low)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex flex-col gap-2",
          span { class: "text-2xl text-[var(--primary)]", "\u{1F5C4}" }
          span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
            "{name}"
          }
          span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
            "CONFIGURED"
          }
        }
      }
    }
  }
}
