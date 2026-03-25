use dioxus::prelude::*;

async fn load_mcp_servers() -> Vec<String> {
  let Ok(content) = tokio::fs::read_to_string(".mcp.json").await else {
    return Vec::new();
  };
  let json: serde_json::Value = match serde_json::from_str(&content) {
    Ok(v) => v,
    Err(_) => return Vec::new(),
  };
  json.get("mcpServers").and_then(|v| v.as_object()).map(|obj| obj.keys().cloned().collect()).unwrap_or_default()
}

#[component]
pub fn McpPanel() -> Element {
  let mut servers = use_resource(|| async { load_mcp_servers().await });

  rsx! {
    div { class: "flex items-center gap-3 mb-4",
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
        "MCP_EXTENSIONS"
      }
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      button {
        class: "text-xs text-[var(--outline)] hover:text-[var(--primary)] transition-colors duration-150",
        onclick: move |_| servers.restart(),
        span { class: "material-symbols-outlined text-sm", "refresh" }
      }
    }
    match &*servers.value().read() {
        Some(names) => rsx! {
          div { class: "grid grid-cols-4 gap-3",
            for name in names {
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
        },
        None => rsx! {
          div { class: "text-xs text-[var(--outline)] py-4 text-center", "Loading MCP servers..." }
        },
    }
  }
}
