use dioxus::prelude::*;

use crate::pages::agents::run_types::HeartbeatRun;

use super::types::{
  DesktopAgentRuntime, DesktopAgentStatus, DesktopFlowRun, DesktopFlowRunSummary, DesktopRuntimeEvent, DesktopRuntimeEventKind, DesktopToolActivity,
  DesktopToolStatus, payload_text,
};

#[derive(Clone)]
pub struct DesktopRuntimeRegistry {
  pub agents: Signal<Vec<DesktopAgentRuntime>>,
  pub events: Signal<Vec<DesktopRuntimeEvent>>,
  pub tools: Signal<Vec<DesktopToolActivity>>,
  pub flows: Signal<Vec<DesktopFlowRun>>,
}

impl Default for DesktopRuntimeRegistry {
  fn default() -> Self {
    Self::new()
  }
}

impl DesktopRuntimeRegistry {
  pub fn new() -> Self {
    Self { agents: Signal::new(Vec::new()), events: Signal::new(Vec::new()), tools: Signal::new(Vec::new()), flows: Signal::new(Vec::new()) }
  }

  pub fn register_agent(&self, agent: DesktopAgentRuntime) {
    let mut agents = self.agents;
    agents.write().push(agent);
  }

  pub fn register_flow_run(&self, flow: DesktopFlowRun) {
    let mut flows = self.flows;
    flows.write().push(flow);
  }

  pub fn update_agent<F>(&self, agent_id: &str, update: F)
  where
    F: FnOnce(&mut DesktopAgentRuntime),
  {
    let mut agents = self.agents;
    if let Some(agent) = agents.write().iter_mut().find(|agent| agent.id == agent_id) {
      update(agent);
    }
  }

  pub fn append_event(&self, event: DesktopRuntimeEvent) {
    self.update_agent(&event.agent_id, |agent| {
      agent.last_event_at = event.ts.clone();
      match event.kind {
        DesktopRuntimeEventKind::AgentSpawn => agent.status = DesktopAgentStatus::Running,
        DesktopRuntimeEventKind::AgentStop => {
          if agent.status != DesktopAgentStatus::Error && agent.status != DesktopAgentStatus::Aborted {
            agent.status = DesktopAgentStatus::Completed;
          }
        },
        DesktopRuntimeEventKind::BackendError => agent.status = DesktopAgentStatus::Error,
        _ => {},
      }
    });
    let mut events = self.events;
    events.write().push(event);
  }

  pub fn upsert_tool(&self, activity: DesktopToolActivity) {
    let mut tools_signal = self.tools;
    let mut tools = tools_signal.write();
    if let Some(existing) = tools.iter_mut().find(|tool| tool.call_id == activity.call_id) {
      *existing = activity;
      return;
    }
    tools.push(activity);
  }

  pub fn all_agents(&self) -> Vec<DesktopAgentRuntime> {
    let mut agents = self.agents.read().clone();
    agents.sort_by(|left, right| right.last_event_at.cmp(&left.last_event_at));
    agents
  }

  pub fn find_agent(&self, agent_id: &str) -> Option<DesktopAgentRuntime> {
    self.agents.read().iter().find(|agent| agent.id == agent_id).cloned()
  }

  pub fn events_for_agent(&self, agent_id: &str) -> Vec<DesktopRuntimeEvent> {
    self.events.read().iter().filter(|event| event.agent_id == agent_id).cloned().collect()
  }

  pub fn tools_for_agent(&self, agent_id: &str) -> Vec<DesktopToolActivity> {
    let mut tools: Vec<_> = self.tools.read().iter().filter(|tool| tool.agent_id == agent_id).cloned().collect();
    tools.sort_by(|left, right| right.call_id.cmp(&left.call_id));
    tools
  }

  pub fn runs_for_agent(&self, agent_id: &str) -> Vec<HeartbeatRun> {
    self
      .find_agent(agent_id)
      .map(|agent| {
        vec![HeartbeatRun {
          id: agent.flow_run_id.clone().unwrap_or_else(|| agent.id.clone()),
          agent_id: agent.id.clone(),
          company_id: String::new(),
          status: status_label(&agent.status).to_string(),
          invocation_source: if agent.flow_id.is_some() { "automation".to_string() } else { "on_demand".to_string() },
          trigger_detail: agent.flow_run_id.clone().or(agent.flow_id.clone()),
          started_at: Some(agent.created_at.clone()),
          finished_at: None,
          created_at: agent.created_at,
          error: self.last_error_for_agent(agent_id),
          error_code: None,
          usage_json: None,
          result_json: None,
          context_snapshot: None,
        }]
      })
      .unwrap_or_default()
  }

  pub fn flow_runs_for_flow(&self, flow_id: &str) -> Vec<DesktopFlowRunSummary> {
    let mut runs: Vec<_> = self
      .flows
      .read()
      .iter()
      .filter(|run| run.flow_id == flow_id)
      .filter_map(|run| {
        let root_agent = self.find_agent(&run.root_agent_id)?;
        Some(DesktopFlowRunSummary {
          id: run.id.clone(),
          flow_id: run.flow_id.clone(),
          title: run.title.clone(),
          root_agent_id: run.root_agent_id.clone(),
          root_agent_name: root_agent.name.clone(),
          root_agent_status: root_agent.status,
          created_at: run.created_at.clone(),
          last_event_at: root_agent.last_event_at,
        })
      })
      .collect();
    runs.sort_by(|left, right| right.last_event_at.cmp(&left.last_event_at));
    runs
  }

  pub fn agents_for_flow(&self, flow_id: &str) -> Vec<DesktopAgentRuntime> {
    self.all_agents().into_iter().filter(|agent| agent.flow_id.as_deref() == Some(flow_id)).collect()
  }

  pub fn agents_for_flow_run(&self, flow_run_id: &str) -> Vec<DesktopAgentRuntime> {
    self.all_agents().into_iter().filter(|agent| agent.flow_run_id.as_deref() == Some(flow_run_id)).collect()
  }

  fn last_error_for_agent(&self, agent_id: &str) -> Option<String> {
    self
      .events_for_agent(agent_id)
      .into_iter()
      .rev()
      .find(|event| matches!(event.kind, DesktopRuntimeEventKind::ToolError | DesktopRuntimeEventKind::BackendError))
      .and_then(|event| payload_text(&event.payload))
  }
}

pub fn status_label(status: &DesktopAgentStatus) -> &'static str {
  match status {
    DesktopAgentStatus::Idle => "idle",
    DesktopAgentStatus::Starting => "queued",
    DesktopAgentStatus::Running => "running",
    DesktopAgentStatus::Paused => "paused",
    DesktopAgentStatus::Completed => "succeeded",
    DesktopAgentStatus::Error => "error",
    DesktopAgentStatus::Aborted => "cancelled",
  }
}

pub fn tool_status_label(status: &DesktopToolStatus) -> &'static str {
  match status {
    DesktopToolStatus::Running => "running",
    DesktopToolStatus::Completed => "completed",
    DesktopToolStatus::Error => "error",
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::runtime::types::{DesktopAgentLaunchSpec, DesktopAgentRuntime, now_ts};

  #[test]
  fn flow_runs_are_grouped_by_flow_and_sorted_by_latest_event() {
    let registry = DesktopRuntimeRegistry::new();

    let mut alpha = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("Alpha", "a", "p"));
    alpha.id = "agent-alpha".to_string();
    alpha.flow_id = Some("flow-a".to_string());
    alpha.flow_run_id = Some("run-1".to_string());
    alpha.last_event_at = "10".to_string();
    registry.register_agent(alpha.clone());
    registry.register_flow_run(DesktopFlowRun {
      id: "run-1".to_string(),
      flow_id: "flow-a".to_string(),
      root_agent_id: alpha.id.clone(),
      title: "Run one".to_string(),
      created_at: now_ts(),
    });

    let mut beta = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("Beta", "b", "p"));
    beta.id = "agent-beta".to_string();
    beta.flow_id = Some("flow-a".to_string());
    beta.flow_run_id = Some("run-2".to_string());
    beta.last_event_at = "20".to_string();
    registry.register_agent(beta.clone());
    registry.register_flow_run(DesktopFlowRun {
      id: "run-2".to_string(),
      flow_id: "flow-a".to_string(),
      root_agent_id: beta.id.clone(),
      title: "Run two".to_string(),
      created_at: now_ts(),
    });

    let summaries = registry.flow_runs_for_flow("flow-a");
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].id, "run-2");
    assert_eq!(summaries[1].id, "run-1");
    assert_eq!(registry.agents_for_flow_run("run-1").len(), 1);
  }
}
