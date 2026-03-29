mod budget_tab;
mod config_form;
pub(crate) mod detail;
pub mod list;
mod live_run_widget;
mod new_agent;
mod overview;
mod run_detail;
pub mod run_types;
mod runs_tab;
mod skills_tab;
mod transcript;
pub mod types;

use self::detail::AgentDetailShell;
use self::list::AgentList;
use self::new_agent::{NewAgentDialog, NewAgentPayload};
use self::types::{AgentDetail, AgentSummary};
use dioxus::prelude::*;

#[component]
pub fn Agents() -> Element {
  let mut selected_agent_id = use_signal(|| Option::<String>::None);
  let mut show_new_dialog = use_signal(|| false);
  let agents: Vec<AgentSummary> = Vec::new();

  let selected_detail: Option<AgentDetail> = selected_agent_id.read().as_ref().and_then(|id| agents.iter().find(|a| a.id == *id)).map(|s| AgentDetail {
    id: s.id.clone(),
    name: s.name.clone(),
    role: s.role.clone(),
    title: s.title.clone(),
    status: s.status.clone(),
    adapter_type: s.adapter_type.clone(),
    icon: s.icon.clone(),
    last_heartbeat_at: s.last_heartbeat_at.clone(),
    reports_to: s.reports_to.clone(),
    created_at: s.created_at.clone(),
    budget_monthly_cents: 0,
    spent_monthly_cents: 0,
    adapter_config: serde_json::Value::Object(Default::default()),
    runtime_config: serde_json::Value::Object(Default::default()),
    pause_reason: None,
  });

  rsx! {
    match selected_detail {
        Some(agent) => rsx! {
          AgentDetailShell {
            agent,
            on_back: move |_| selected_agent_id.set(None),
            on_run: move |_| {},
            on_pause: move |_| {},
            on_resume: move |_| {},
            on_terminate: move |_| {},
          }
        },
        None => rsx! {
          AgentList {
            agents,
            on_select: move |id: String| selected_agent_id.set(Some(id)),
            on_new_agent: move |_| show_new_dialog.set(true),
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
