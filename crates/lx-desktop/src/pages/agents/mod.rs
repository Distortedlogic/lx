mod agent_card;
mod mcp_panel;
mod throughput_panel;
mod voice_banner;

use dioxus::prelude::*;

use self::agent_card::{AgentCard, AgentStatus};
use self::mcp_panel::McpPanel;
use self::throughput_panel::ThroughputPanel;
use self::voice_banner::VoiceBanner;

#[component]
pub fn Agents() -> Element {
  rsx! {
    div { class: "flex h-full gap-4 p-4 overflow-auto",
      div { class: "flex-[7] flex flex-col gap-4 min-w-0",
        VoiceBanner {}
        div {
          p { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)] mb-3", "ACTIVE CODE AGENTS" }
          div { class: "flex flex-col gap-3",
            AgentCard {
              agent_name: "CLAUDE-PRIMARY",
              status: AgentStatus::Active,
              pid: "4291",
              memory: "2.1 GB",
              action_text: "Analyzing src/compiler/parser.rs — restructuring AST node types",
              task_items: vec![
                ("done", "Parse token stream"),
                ("active", "Restructure AST nodes"),
                ("pending", "Update type checker"),
              ],
            }
            AgentCard {
              agent_name: "SEARCH-INDEXER",
              status: AgentStatus::Running,
              pid: "4305",
              memory: "0.8 GB",
              action_text: "Indexing repository files — 1,247 / 3,891 processed",
              task_items: vec![
                ("done", "Scan directory tree"),
                ("active", "Build search index"),
                ("pending", "Generate embeddings"),
              ],
            }
            AgentCard {
              agent_name: "TASK-PLANNER",
              status: AgentStatus::Idle,
              pid: "4312",
              memory: "0.3 GB",
              action_text: "",
              task_items: vec![],
            }
          }
        }
      }
      div { class: "flex-[3] flex flex-col gap-4 min-w-0",
        McpPanel {}
        ThroughputPanel {}
      }
    }
  }
}
