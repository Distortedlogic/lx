use dioxus::prelude::*;

struct McpServer {
  name: &'static str,
  info: &'static str,
  status: &'static str,
}

const SERVERS: &[McpServer] = &[
  McpServer { name: "FILESYSTEM", info: "localhost:3001", status: "connected" },
  McpServer { name: "GIT", info: "localhost:3002", status: "connected" },
  McpServer { name: "WEB-SEARCH", info: "api.search:443", status: "degraded" },
  McpServer { name: "DATABASE", info: "db.internal:5432", status: "disconnected" },
];

#[component]
pub fn McpPanel() -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container)] rounded-lg p-4",
      p { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)] mb-3", "MCP SERVER CLUSTER" }
      div { class: "flex flex-col gap-2",
        for server in SERVERS {
          {
              let dot_color = match server.status {
                  "connected" => "text-[var(--success)]",
                  "degraded" => "text-[var(--warning)]",
                  _ => "text-[var(--error)]",
              };
              rsx! {
                div { class: "flex items-center gap-2 text-xs",
                  span { class: "{dot_color}", "\u{25CF}" }
                  span { class: "font-medium text-[var(--on-surface)] uppercase tracking-wider", "{server.name}" }
                  div { class: "flex-1" }
                  span { class: "text-[var(--outline)]", "{server.info}" }
                  button { class: "text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors duration-150 ml-1", "\u{2699}" }
                }
              }
          }
        }
      }
      button { class: "w-full mt-3 py-2 text-xs text-[var(--outline)] hover:text-[var(--primary)] border border-dashed border-[var(--outline-variant)] rounded hover:border-[var(--primary)] transition-colors duration-150 uppercase tracking-wider",
        "+ CONNECT NEW MCP SERVER"
      }
    }
  }
}
