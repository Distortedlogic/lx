mod budget_tab;
mod config_form;
pub(crate) mod detail;
pub mod list;
pub mod live_run_widget;
mod new_agent;
mod overview;
mod run_detail;
pub mod run_types;
mod runs_tab;
mod skills_tab;
mod transcript;
mod transcript_blocks;
mod transcript_groups;
pub mod types;

use self::detail::AgentDetailShell;
use self::list::AgentList;
use self::new_agent::{NewAgentDialog, NewAgentPayload};
use self::types::{AgentDetail, AgentSummary};
use crate::routes::Route;
use crate::runtime::{status_label, use_desktop_runtime};
use dioxus::prelude::*;

#[component]
pub fn Agents() -> Element {
  let runtime = use_desktop_runtime();
  let navigator = use_navigator();
  let mut selected_agent_id = use_signal(|| Option::<String>::None);
  let mut show_new_dialog = use_signal(|| false);
  let agents: Vec<AgentSummary> = runtime
    .registry
    .all_agents()
    .into_iter()
    .map(|agent| AgentSummary {
      id: agent.id.clone(),
      name: agent.name.clone(),
      role: "general".to_string(),
      title: Some(agent.task_summary.clone()),
      status: status_label(&agent.status).to_string(),
      adapter_type: "pi_rpc".to_string(),
      icon: Some("smart_toy".to_string()),
      last_heartbeat_at: Some(agent.last_event_at.clone()),
      reports_to: agent.parent_id.clone(),
      created_at: agent.created_at.clone(),
    })
    .collect();

  let selected_detail: Option<AgentDetail> = selected_agent_id.read().as_ref().and_then(|id| runtime.registry.find_agent(id)).map(|agent| AgentDetail {
    id: agent.id.clone(),
    name: agent.name.clone(),
    role: "general".to_string(),
    title: Some(agent.task_summary.clone()),
    status: status_label(&agent.status).to_string(),
    adapter_type: "pi_rpc".to_string(),
    icon: Some("smart_toy".to_string()),
    last_heartbeat_at: Some(agent.last_event_at.clone()),
    reports_to: agent.parent_id.clone(),
    created_at: agent.created_at.clone(),
    budget_monthly_cents: 0,
    spent_monthly_cents: 0,
    adapter_config: serde_json::json!({ "model": agent.model }),
    runtime_config: serde_json::json!({ "session_id": agent.session_id, "flow_id": agent.flow_id, "flow_run_id": agent.flow_run_id, "cwd": agent.cwd }),
    pause_reason: None,
  });

  rsx! {
    match selected_detail {
        Some(agent) => {
            let run_id = agent.id.clone();
            let pause_id = agent.id.clone();
            let resume_id = agent.id.clone();
            let terminate_id = agent.id.clone();
            rsx! {
              AgentDetailShell {
                agent: agent.clone(),
                on_back: move |_| selected_agent_id.set(None),
                on_run: {
                    let runtime = runtime.clone();
                    move |_| runtime.prompt(run_id.clone(), "Continue the active task.".to_string())
                },
                on_pause: {
                    let runtime = runtime.clone();
                    move |_| runtime.pause(pause_id.clone())
                },
                on_resume: {
                    let runtime = runtime.clone();
                    move |_| runtime.resume(resume_id.clone())
                },
                on_terminate: {
                    let runtime = runtime.clone();
                    move |_| runtime.abort(terminate_id.clone())
                },
              }
            }
        }
        None => rsx! {
          AgentList {
            agents,
            on_select: move |id: String| selected_agent_id.set(Some(id)),
            on_new_agent: move |_| show_new_dialog.set(true),
            on_open_widget: move |agent_id: String| {
                navigator.push(Route::PiAgentPage { agent_id });
            },
          }
        },
    }
    NewAgentDialog {
      open: *show_new_dialog.read(),
      on_close: move |_| show_new_dialog.set(false),
      on_create: move |_payload: NewAgentPayload| {
          show_new_dialog.set(false);
      },
    }
  }
}
