use dioxus::prelude::*;

use crate::components::page_skeleton::PageSkeleton;
use crate::pages::agents::types::AgentDetail as AgentDetailData;

use super::agents::detail::AgentDetailShell;

#[component]
pub fn AgentDetail(agent_id: String) -> Element {
  rsx! {
    SuspenseBoundary {
      fallback: |_| rsx! { PageSkeleton { variant: "detail".to_string() } },
      AgentDetailInner { agent_id }
    }
  }
}

#[component]
fn AgentDetailInner(agent_id: String) -> Element {
  let agent = AgentDetailData {
    id: agent_id.clone(),
    name: agent_id,
    role: "general".to_string(),
    title: None,
    status: "active".to_string(),
    adapter_type: "claude_local".to_string(),
    icon: None,
    last_heartbeat_at: None,
    reports_to: None,
    created_at: String::new(),
    budget_monthly_cents: 0,
    spent_monthly_cents: 0,
    adapter_config: serde_json::Value::Object(Default::default()),
    runtime_config: serde_json::Value::Object(Default::default()),
    pause_reason: None,
  };

  let nav = navigator();
  rsx! {
    AgentDetailShell {
      agent,
      on_back: move |_| {
          nav.go_back();
      },
      on_run: move |_| {},
      on_pause: move |_| {},
      on_resume: move |_| {},
      on_terminate: move |_| {},
    }
  }
}
