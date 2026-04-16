use dioxus::prelude::*;

use crate::pages::agents::live_run_widget::{LiveRunInfo, LiveRunWidget};
use crate::routes::Route;
use crate::runtime::{status_label, use_desktop_runtime};

#[component]
pub fn ActiveAgentsPanel() -> Element {
  let runtime = use_desktop_runtime();
  let navigator = use_navigator();
  let runs: Vec<LiveRunInfo> = runtime
    .registry
    .all_agents()
    .into_iter()
    .map(|agent| LiveRunInfo {
      id: agent.id.clone(),
      agent_id: agent.id.clone(),
      agent_name: agent.name,
      status: status_label(&agent.status).to_string(),
      invocation_source: if agent.flow_id.is_some() { "automation".to_string() } else { "on_demand".to_string() },
      started_at: Some(agent.created_at.clone()),
      created_at: agent.created_at,
    })
    .collect();

  rsx! {
    div {
      h3 { class: "mb-3 text-sm font-semibold uppercase tracking-wide text-[var(--on-surface-variant)]",
        "Agents"
      }
      if runs.is_empty() {
        div { class: "rounded-xl border border-[var(--outline-variant)] p-4",
          p { class: "text-sm text-[var(--on-surface-variant)]", "No recent agent runs." }
        }
      } else {
        LiveRunWidget {
          runs,
          on_cancel: {
              let runtime = runtime.clone();
              move |agent_id: String| runtime.abort(agent_id)
          },
          on_open_run: move |(agent_id, _run_id): (String, String)| {
              navigator.push(Route::PiAgentPage { agent_id });
          },
        }
      }
    }
  }
}
