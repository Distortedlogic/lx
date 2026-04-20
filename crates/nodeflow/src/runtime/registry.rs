use std::sync::{Arc, RwLock};

use tokio::sync::watch;

use super::types::{
  DesktopAgentRuntime, DesktopAgentStatus, DesktopFlowRun, DesktopFlowRunSummary, DesktopRuntimeEvent, DesktopRuntimeEventKind, DesktopToolActivity,
  DesktopToolStatus,
};

#[derive(Clone)]
pub struct DesktopRuntimeRegistry {
  state: Arc<RwLock<DesktopRuntimeState>>,
  revisions: watch::Sender<u64>,
}

#[derive(Default)]
struct DesktopRuntimeState {
  agents: Vec<DesktopAgentRuntime>,
  events: Vec<DesktopRuntimeEvent>,
  tools: Vec<DesktopToolActivity>,
  flows: Vec<DesktopFlowRun>,
}

impl Default for DesktopRuntimeRegistry {
  fn default() -> Self {
    Self::new()
  }
}

impl DesktopRuntimeRegistry {
  pub fn new() -> Self {
    let (revisions, _) = watch::channel(0);
    Self { state: Arc::new(RwLock::new(DesktopRuntimeState::default())), revisions }
  }

  pub fn register_agent(&self, agent: DesktopAgentRuntime) {
    self.write_state().agents.push(agent);
    self.notify_change();
  }

  pub fn register_flow_run(&self, flow: DesktopFlowRun) {
    self.write_state().flows.push(flow);
    self.notify_change();
  }

  pub fn update_flow_run<F>(&self, flow_run_id: &str, update: F)
  where
    F: FnOnce(&mut DesktopFlowRun),
  {
    if let Some(flow_run) = self.write_state().flows.iter_mut().find(|flow| flow.id == flow_run_id) {
      update(flow_run);
    }
    self.notify_change();
  }

  pub fn update_agent<F>(&self, agent_id: &str, update: F)
  where
    F: FnOnce(&mut DesktopAgentRuntime),
  {
    if let Some(agent) = self.write_state().agents.iter_mut().find(|agent| agent.id == agent_id) {
      update(agent);
    }
    self.notify_change();
  }

  pub fn append_event(&self, event: DesktopRuntimeEvent) {
    let mut state = self.write_state();
    if let Some(agent) = state.agents.iter_mut().find(|agent| agent.id == event.agent_id) {
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
    }
    state.events.push(event);
    drop(state);
    self.notify_change();
  }

  pub fn upsert_tool(&self, activity: DesktopToolActivity) {
    let mut state = self.write_state();
    if let Some(existing) = state.tools.iter_mut().find(|tool| tool.call_id == activity.call_id) {
      *existing = activity;
      drop(state);
      self.notify_change();
      return;
    }
    state.tools.push(activity);
    drop(state);
    self.notify_change();
  }

  pub fn all_agents(&self) -> Vec<DesktopAgentRuntime> {
    let mut agents = self.read_state().agents.clone();
    agents.sort_by(|left, right| right.last_event_at.cmp(&left.last_event_at));
    agents
  }

  pub fn find_agent(&self, agent_id: &str) -> Option<DesktopAgentRuntime> {
    self.read_state().agents.iter().find(|agent| agent.id == agent_id).cloned()
  }

  pub fn events_for_agent(&self, agent_id: &str) -> Vec<DesktopRuntimeEvent> {
    self.read_state().events.iter().filter(|event| event.agent_id == agent_id).cloned().collect()
  }

  pub fn tools_for_agent(&self, agent_id: &str) -> Vec<DesktopToolActivity> {
    let mut tools: Vec<_> = self.read_state().tools.iter().filter(|tool| tool.agent_id == agent_id).cloned().collect();
    tools.sort_by(|left, right| right.call_id.cmp(&left.call_id));
    tools
  }

  pub fn flow_runs_for_flow(&self, flow_id: &str) -> Vec<DesktopFlowRunSummary> {
    let mut runs: Vec<_> = self
      .read_state()
      .flows
      .iter()
      .filter(|run| run.flow_id == flow_id)
      .filter_map(|run| {
        let root_agent =
          run.root_agent_id.as_deref().and_then(|agent_id| self.find_agent(agent_id)).or_else(|| self.agents_for_flow_run(&run.id).into_iter().next());
        let root_agent = root_agent?;
        Some(DesktopFlowRunSummary {
          id: run.id.clone(),
          flow_id: run.flow_id.clone(),
          title: run.title.clone(),
          root_agent_id: root_agent.id.clone(),
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

  pub fn agent_for_flow_run_node(&self, flow_run_id: &str, flow_node_id: &str) -> Option<DesktopAgentRuntime> {
    self.agents_for_flow_run(flow_run_id).into_iter().find(|agent| agent.flow_node_id.as_deref() == Some(flow_node_id))
  }

  pub fn revision(&self) -> u64 {
    *self.revisions.borrow()
  }

  pub fn subscribe(&self) -> watch::Receiver<u64> {
    self.revisions.subscribe()
  }

  fn notify_change(&self) {
    let next = self.revision().saturating_add(1);
    let _ = self.revisions.send(next);
  }

  fn read_state(&self) -> std::sync::RwLockReadGuard<'_, DesktopRuntimeState> {
    self.state.read().expect("desktop runtime registry read lock poisoned")
  }

  fn write_state(&self) -> std::sync::RwLockWriteGuard<'_, DesktopRuntimeState> {
    self.state.write().expect("desktop runtime registry write lock poisoned")
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
      root_agent_id: Some(alpha.id.clone()),
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
      root_agent_id: Some(beta.id.clone()),
      title: "Run two".to_string(),
      created_at: now_ts(),
    });

    let summaries = registry.flow_runs_for_flow("flow-a");
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].id, "run-2");
    assert_eq!(summaries[1].id, "run-1");
    assert_eq!(registry.agents_for_flow_run("run-1").len(), 1);
  }

  #[test]
  fn flow_run_nodes_are_addressable_by_flow_node_id() {
    let registry = DesktopRuntimeRegistry::new();
    let mut agent = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("Alpha", "a", "p"));
    agent.id = "agent-alpha".to_string();
    agent.flow_id = Some("flow-a".to_string());
    agent.flow_run_id = Some("run-1".to_string());
    agent.flow_node_id = Some("node-a".to_string());
    registry.register_agent(agent.clone());

    assert_eq!(registry.agent_for_flow_run_node("run-1", "node-a").map(|entry| entry.id), Some("agent-alpha".to_string()));
  }
}
