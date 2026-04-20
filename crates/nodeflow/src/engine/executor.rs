use std::collections::{HashMap, HashSet, VecDeque};

use lx_graph_editor::catalog::GraphNodeTemplate;
use lx_graph_editor::model::GraphDocument;

use crate::credentials::CredentialStore;

use super::context::ExecutionContext;
use super::runner::{NodeRunContext, NodeRunnerRegistry};
use super::types::{FlowExecutionReport, NodeExecutionData, NodeExecutionRecord, NodeExecutionStatus, now_ts};

pub async fn execute_single_node(
  document: &GraphDocument,
  node_id: &str,
  templates: &[GraphNodeTemplate],
  registry: &NodeRunnerRegistry,
  credentials: &CredentialStore,
) -> (FlowExecutionReport, ExecutionContext) {
  let mut isolated = GraphDocument {
    id: document.id.clone(),
    title: document.title.clone(),
    metadata: document.metadata.clone(),
    viewport: document.viewport,
    selection: document.selection.clone(),
    nodes: document.nodes.iter().filter(|node| node.id == node_id).cloned().collect(),
    edges: Vec::new(),
  };
  if let Some(node) = isolated.nodes.first_mut()
    && let Some(pin) = node.properties.as_object().and_then(|map| map.get("__pin__")).cloned()
  {
    node.properties["__pin_applied__"] = pin;
  }
  execute_flow(&isolated, templates, registry, credentials).await
}

pub async fn execute_flow(
  document: &GraphDocument,
  templates: &[GraphNodeTemplate],
  registry: &NodeRunnerRegistry,
  credentials: &CredentialStore,
) -> (FlowExecutionReport, ExecutionContext) {
  let mut report = FlowExecutionReport { started_at: now_ts(), ..FlowExecutionReport::default() };
  let mut context = ExecutionContext::default();

  let node_ids: HashSet<String> = document.nodes.iter().map(|node| node.id.clone()).collect();
  let mut indegree: HashMap<String, usize> = node_ids.iter().map(|id| (id.clone(), 0usize)).collect();
  let mut outgoing: HashMap<String, Vec<(String, String, String)>> = HashMap::new();
  let mut incoming: HashMap<String, Vec<(String, String, String)>> = HashMap::new();

  for edge in &document.edges {
    if !node_ids.contains(&edge.from.node_id) || !node_ids.contains(&edge.to.node_id) {
      continue;
    }
    outgoing.entry(edge.from.node_id.clone()).or_default().push((edge.from.port_id.clone(), edge.to.node_id.clone(), edge.to.port_id.clone()));
    incoming.entry(edge.to.node_id.clone()).or_default().push((edge.from.node_id.clone(), edge.from.port_id.clone(), edge.to.port_id.clone()));
    *indegree.entry(edge.to.node_id.clone()).or_default() += 1;
  }

  let template_by_id: HashMap<&str, &GraphNodeTemplate> = templates.iter().map(|template| (template.id.as_str(), template)).collect();

  let mut queue: VecDeque<String> = indegree.iter().filter(|(_, count)| **count == 0).map(|(id, _)| id.clone()).collect();
  let mut visited: HashSet<String> = HashSet::new();

  while let Some(node_id) = queue.pop_front() {
    if !visited.insert(node_id.clone()) {
      continue;
    }
    let Some(node) = document.node(&node_id) else {
      continue;
    };

    let started_at = now_ts();
    let template = template_by_id.get(node.template_id.as_str()).copied();

    let Some(template) = template else {
      record_skip(&mut report, &node_id, &node.template_id, started_at, format!("Template `{}` is not registered.", node.template_id));
      context.set_node_outputs(&node_id, HashMap::new());
      advance_downstream(&outgoing, &node_id, &mut indegree, &mut queue);
      continue;
    };

    let Some(runner) = registry.get(&node.template_id) else {
      let outputs = empty_outputs_for_template(template);
      context.set_node_outputs(&node_id, outputs);
      record_skip(&mut report, &node_id, &node.template_id, started_at, format!("No runner registered for `{}`.", node.template_id));
      advance_downstream(&outgoing, &node_id, &mut indegree, &mut queue);
      continue;
    };

    let policy = retry_policy(node);
    let mut attempt: usize = 0;
    let mut last_logs: Vec<String> = Vec::new();
    let outcome_result = loop {
      let inputs = collect_inputs(&incoming, &node_id, &context);
      match runner.run(NodeRunContext { node, template, inputs, exec: &context, credentials }).await {
        Ok(outcome) => break Ok(outcome),
        Err(error) if attempt < policy.max_retries => {
          last_logs.push(format!("attempt {} failed: {error}", attempt + 1));
          attempt += 1;
          if policy.delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(policy.delay_ms)).await;
          }
        },
        Err(error) => break Err(error),
      }
    };

    match outcome_result {
      Ok(outcome) => {
        context.set_node_outputs(&node_id, outcome.outputs);
        let mut combined_logs = last_logs;
        combined_logs.extend(outcome.logs);
        report.records.push(NodeExecutionRecord {
          node_id: node_id.clone(),
          template_id: node.template_id.clone(),
          status: NodeExecutionStatus::Success,
          logs: combined_logs,
          error: None,
          started_at,
          finished_at: Some(now_ts()),
        });
        advance_downstream(&outgoing, &node_id, &mut indegree, &mut queue);
      },
      Err(error) => {
        let error_message = error.to_string();
        let mut combined_logs = last_logs;
        combined_logs.push(format!("final attempt failed: {error_message}"));
        report.records.push(NodeExecutionRecord {
          node_id: node_id.clone(),
          template_id: node.template_id.clone(),
          status: NodeExecutionStatus::Failed,
          logs: combined_logs,
          error: Some(error_message.clone()),
          started_at,
          finished_at: Some(now_ts()),
        });
        if policy.continue_on_fail {
          context.set_node_outputs(&node_id, empty_outputs_for_template(template));
          advance_downstream(&outgoing, &node_id, &mut indegree, &mut queue);
        } else {
          report.aborted = true;
          report.error = Some(format!("Node `{node_id}` failed: {error_message}"));
          report.finished_at = Some(now_ts());
          return (report, context);
        }
      },
    }
  }

  report.finished_at = Some(now_ts());
  (report, context)
}

fn record_skip(report: &mut FlowExecutionReport, node_id: &str, template_id: &str, started_at: String, log: String) {
  report.records.push(NodeExecutionRecord {
    node_id: node_id.to_string(),
    template_id: template_id.to_string(),
    status: NodeExecutionStatus::Skipped,
    logs: vec![log],
    error: None,
    started_at,
    finished_at: Some(now_ts()),
  });
}

fn empty_outputs_for_template(template: &GraphNodeTemplate) -> HashMap<String, NodeExecutionData> {
  template
    .ports
    .iter()
    .filter(|port| port.direction == lx_graph_editor::catalog::PortDirection::Output)
    .map(|port| (port.id.clone(), NodeExecutionData::new()))
    .collect()
}

fn collect_inputs(incoming: &HashMap<String, Vec<(String, String, String)>>, node_id: &str, context: &ExecutionContext) -> HashMap<String, NodeExecutionData> {
  let mut inputs: HashMap<String, NodeExecutionData> = HashMap::new();
  let Some(edges) = incoming.get(node_id) else {
    return inputs;
  };
  for (from_node, from_port, to_port) in edges {
    if let Some(items) = context.port_items(from_node, from_port) {
      inputs.entry(to_port.clone()).or_default().extend(items.iter().cloned());
    }
  }
  inputs
}

#[derive(Clone, Debug)]
struct RetryPolicy {
  max_retries: usize,
  delay_ms: u64,
  continue_on_fail: bool,
}

fn retry_policy(node: &lx_graph_editor::model::GraphNode) -> RetryPolicy {
  let raw = node.properties.as_object().and_then(|map| map.get("__retry__")).cloned().unwrap_or_default();
  RetryPolicy {
    max_retries: raw.get("max_retries").and_then(serde_json::Value::as_u64).unwrap_or(0).min(10) as usize,
    delay_ms: raw.get("delay_ms").and_then(serde_json::Value::as_u64).unwrap_or(500),
    continue_on_fail: raw.get("continue_on_fail").and_then(serde_json::Value::as_bool).unwrap_or(false),
  }
}

fn advance_downstream(
  outgoing: &HashMap<String, Vec<(String, String, String)>>,
  node_id: &str,
  indegree: &mut HashMap<String, usize>,
  queue: &mut VecDeque<String>,
) {
  let Some(edges) = outgoing.get(node_id) else {
    return;
  };
  for (_, to_node, _) in edges {
    if let Some(count) = indegree.get_mut(to_node) {
      *count = count.saturating_sub(1);
      if *count == 0 {
        queue.push_back(to_node.clone());
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::credentials::CredentialStore;
  use crate::engine::default_registry;
  use crate::engine::types::NodeExecutionStatus;
  use crate::pages::flows::catalog::workflow_node_templates;
  use crate::pages::flows::sample::{DEFAULT_FLOW_ID, sample_document};

  #[tokio::test]
  async fn sample_flow_runs_end_to_end() {
    let document = sample_document(DEFAULT_FLOW_ID);
    let templates = workflow_node_templates();
    let registry = default_registry();
    let credentials = CredentialStore::in_memory();

    let (report, context) = execute_flow(&document, &templates, &registry, &credentials).await;

    assert!(!report.aborted, "sample flow should not abort: {:?}", report.error);
    assert_eq!(report.records.len(), document.nodes.len());
    for record in &report.records {
      assert_eq!(record.status, NodeExecutionStatus::Success, "node `{}` failed", record.node_id);
    }
    for node in &document.nodes {
      let outputs = context.node_outputs(&node.id).unwrap_or_else(|| panic!("no outputs for node `{}`", node.id));
      for port in templates.iter().find(|template| template.id == node.template_id).map(|template| template.ports.as_slice()).unwrap_or_default() {
        if port.direction == lx_graph_editor::catalog::PortDirection::Output {
          assert!(outputs.contains_key(&port.id), "node `{}` missing output port `{}`", node.id, port.id);
        }
      }
    }
  }

  #[tokio::test]
  async fn stops_on_first_failure() {
    use async_trait::async_trait;
    use std::sync::Arc;
    struct FailingRunner;
    #[async_trait]
    impl crate::engine::NodeRunner for FailingRunner {
      async fn run(&self, _ctx: NodeRunContext<'_>) -> Result<crate::engine::NodeRunOutcome, crate::engine::NodeExecutionError> {
        Err(crate::engine::NodeExecutionError::Runtime("boom".to_string()))
      }
    }

    let document = sample_document(DEFAULT_FLOW_ID);
    let templates = workflow_node_templates();
    let mut registry = default_registry();
    registry.register("web_fetch", Arc::new(FailingRunner));
    let credentials = CredentialStore::in_memory();

    let (report, _context) = execute_flow(&document, &templates, &registry, &credentials).await;

    assert!(report.aborted);
    assert!(report.error.as_deref().unwrap_or_default().contains("boom"));
    assert!(report.failed_count() >= 1);
  }
}
