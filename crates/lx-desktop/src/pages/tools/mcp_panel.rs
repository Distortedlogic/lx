use dioxus::prelude::*;

async fn load_mcp_servers() -> Result<Vec<String>, std::io::Error> {
  let content = tokio::fs::read_to_string(".mcp.json").await?;
  let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
  Ok(json.get("mcpServers").and_then(|v| v.as_object()).map(|obj| obj.keys().cloned().collect()).unwrap_or_default())
}

#[component]
pub fn McpPanel() -> Element {
  let mut refresh = use_signal(|| 0u32);
  let servers = use_loader(move || {
    let _ = refresh();
    load_mcp_servers()
  })?;

  rsx! {
    div { class: "flex items-center gap-3 mb-4",
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
        "MCP_EXTENSIONS"
      }
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      button {
        class: "text-xs text-[var(--outline)] hover:text-[var(--primary)] transition-colors duration-150",
        onclick: move |_| refresh += 1,
        span { class: "material-symbols-outlined text-sm", "refresh" }
      }
    }
    div { class: "grid grid-cols-4 gap-3",
      for name in servers.read().iter() {
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
