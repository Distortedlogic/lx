mod agent_card;
mod mcp_panel;
mod voice_banner;

use dioxus::prelude::*;

use self::agent_card::{AgentCard, AgentStatus};
use self::mcp_panel::McpPanel;
use self::voice_banner::VoiceBanner;

#[component]
pub fn Agents() -> Element {
  rsx! {
    div { class: "flex flex-col h-full gap-4 p-4 overflow-auto",
      VoiceBanner {}
      div { class: "flex items-center justify-between",
        div {
          h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]",
            "MCP_MANAGER"
          }
          p { class: "text-xs text-[var(--outline)] uppercase tracking-wider mt-1",
            "ENVIRONMENT: PRODUCTION // ACCESS_LEVEL: ROOT"
          }
        }
        span { class: "text-xs text-[var(--outline)] uppercase tracking-wider",
          "UPTIME: 1842:12:04"
        }
      }
      div { class: "flex gap-4",
        AgentCard {
          agent_name: "AGENT_DELTA_9",
          status: AgentStatus::Active,
          current_task: Some("REFACTORING_AUTH_FLOW_V2"),
          resources: Some("CPU: 42% // RAM: 1.2GB"),
          live_output: Some(
              vec![
                  "Analysing module dependency graph...",
                  "Identifying bottlenecks in JWT validation...",
                  "Patching core controller at line 412...",
                  "Unit testing initialize [SUCCESS]",
              ],
          ),
          last_active: None,
          memory_load: None,
        }
        AgentCard {
          agent_name: "AGENT_ZETA_0",
          status: AgentStatus::Idle,
          current_task: None,
          resources: None,
          live_output: None,
          last_active: Some("14:02:55 UTC"),
          memory_load: Some(0.2),
        }
      }
      McpPanel {}
    }
  }
}
