use dioxus::prelude::*;

use crate::components::page_skeleton::PageSkeleton;
use crate::pages::agents::types::AgentDetail as AgentDetailData;
use crate::runtime::{status_label, use_desktop_runtime};

use super::agents::detail::AgentDetailShell;

#[component]
pub fn AgentDetail(agent_id: String) -> Element {
  rsx! {
    SuspenseBoundary {
      fallback: |_| rsx! {
        PageSkeleton { variant: "detail".to_string() }
      },
      AgentDetailInner { agent_id }
    }
  }
}

#[component]
fn AgentDetailInner(agent_id: String) -> Element {
  let runtime = use_desktop_runtime();
  let nav = navigator();
  let agent = runtime.registry.find_agent(&agent_id).map(|agent| AgentDetailData {
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

  let Some(agent) = agent else {
    return rsx! {
      div { class: "p-4 text-sm text-[var(--outline)]", "Runtime agent not found." }
    };
  };
  let run_id = agent.id.clone();
  let pause_id = agent.id.clone();
  let resume_id = agent.id.clone();
  let terminate_id = agent.id.clone();

  rsx! {
    AgentDetailShell {
      agent: agent.clone(),
      on_back: move |_| {
          nav.go_back();
      },
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
