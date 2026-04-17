use std::collections::HashSet;

use crate::runtime::DesktopRuntimeRegistry;
use crate::runtime::types::{DesktopAgentRuntime, DesktopAgentStatus, DesktopFlowRunSummary};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowRunGroup {
  pub run_id: String,
  pub flow_id: String,
  pub title: String,
  pub root_agent_id: String,
  pub root_agent_name: String,
  pub root_agent_status: DesktopAgentStatus,
  pub created_at: String,
  pub last_event_at: String,
  pub agents: Vec<DesktopAgentRuntime>,
}

pub fn flow_run_groups(registry: &DesktopRuntimeRegistry, flow_id: &str) -> Vec<FlowRunGroup> {
  let mut groups: Vec<_> = registry.flow_runs_for_flow(flow_id).into_iter().map(|summary| group_from_summary(registry, summary)).collect();
  let mut seen_run_ids: HashSet<String> = groups.iter().map(|group| group.run_id.clone()).collect();

  for agent in registry.agents_for_flow(flow_id) {
    match agent.flow_run_id.clone() {
      Some(run_id) => {
        if !seen_run_ids.insert(run_id.clone()) {
          continue;
        }
        let agents = registry.agents_for_flow_run(&run_id);
        groups.push(group_from_agent(run_id, flow_id.to_string(), agent, agents));
      },
      None => groups.push(group_from_agent(agent.id.clone(), flow_id.to_string(), agent.clone(), vec![agent])),
    }
  }

  groups.sort_by(|left, right| right.last_event_at.cmp(&left.last_event_at));
  groups
}

fn group_from_summary(registry: &DesktopRuntimeRegistry, summary: DesktopFlowRunSummary) -> FlowRunGroup {
  let agents = {
    let agents = registry.agents_for_flow_run(&summary.id);
    if agents.is_empty() { registry.find_agent(&summary.root_agent_id).into_iter().collect() } else { agents }
  };
  FlowRunGroup {
    run_id: summary.id,
    flow_id: summary.flow_id,
    title: summary.title,
    root_agent_id: summary.root_agent_id,
    root_agent_name: summary.root_agent_name,
    root_agent_status: summary.root_agent_status,
    created_at: summary.created_at,
    last_event_at: summary.last_event_at,
    agents,
  }
}

fn group_from_agent(run_id: String, flow_id: String, root_agent: DesktopAgentRuntime, agents: Vec<DesktopAgentRuntime>) -> FlowRunGroup {
  let agents = if agents.is_empty() { vec![root_agent.clone()] } else { agents };
  FlowRunGroup {
    run_id,
    flow_id,
    title: root_agent.name.clone(),
    root_agent_id: root_agent.id.clone(),
    root_agent_name: root_agent.name,
    root_agent_status: root_agent.status,
    created_at: root_agent.created_at.clone(),
    last_event_at: root_agent.last_event_at.clone(),
    agents,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::runtime::types::{DesktopAgentLaunchSpec, DesktopAgentRuntime, DesktopFlowRun, now_ts};

  #[test]
  fn groups_agents_by_flow_run_id() {
    let registry = DesktopRuntimeRegistry::new();

    let mut root = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("Root", "root", "prompt"));
    root.id = "agent-root".to_string();
    root.flow_id = Some("flow-a".to_string());
    root.flow_run_id = Some("run-1".to_string());
    root.last_event_at = "20".to_string();
    registry.register_agent(root.clone());
    registry.register_flow_run(DesktopFlowRun {
      id: "run-1".to_string(),
      flow_id: "flow-a".to_string(),
      root_agent_id: Some(root.id.clone()),
      title: "Run one".to_string(),
      created_at: now_ts(),
    });

    let mut child = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("Child", "child", "prompt"));
    child.id = "agent-child".to_string();
    child.flow_id = Some("flow-a".to_string());
    child.flow_run_id = Some("run-1".to_string());
    child.parent_id = Some(root.id.clone());
    registry.register_agent(child.clone());

    let mut other = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("Other", "other", "prompt"));
    other.id = "agent-other".to_string();
    other.flow_id = Some("flow-a".to_string());
    other.flow_run_id = Some("run-2".to_string());
    other.last_event_at = "10".to_string();
    registry.register_agent(other.clone());
    registry.register_flow_run(DesktopFlowRun {
      id: "run-2".to_string(),
      flow_id: "flow-a".to_string(),
      root_agent_id: Some(other.id.clone()),
      title: "Run two".to_string(),
      created_at: now_ts(),
    });

    let groups = flow_run_groups(&registry, "flow-a");
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].run_id, "run-1");
    assert_eq!(groups[0].agents.len(), 2);
    assert_eq!(groups[1].run_id, "run-2");
    assert_eq!(groups[1].agents.len(), 1);
  }
}
