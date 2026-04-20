use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use super::context::ExecutionContext;
use super::executor::execute_flow;
use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{merged_inputs, properties_lookup};
use super::types::{FlowExecutionReport, NodeExecutionError, NodeItem, NodeRunOutcome};
use crate::credentials::CredentialStore;
use crate::pages::flows::storage::FlowPersistence;

const MAX_SUB_DEPTH: usize = 4;

#[derive(Clone)]
pub struct SubWorkflowRunner {
  persistence: FlowPersistence,
  registry: Arc<NodeRunnerRegistry>,
  credentials: CredentialStore,
  depth: usize,
}

impl SubWorkflowRunner {
  pub fn new(persistence: FlowPersistence, registry: Arc<NodeRunnerRegistry>, credentials: CredentialStore) -> Self {
    Self { persistence, registry, credentials, depth: 0 }
  }

  fn deeper(&self) -> Self {
    Self { persistence: self.persistence.clone(), registry: self.registry.clone(), credentials: self.credentials.clone(), depth: self.depth + 1 }
  }
}

#[async_trait]
impl NodeRunner for SubWorkflowRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    if self.depth >= MAX_SUB_DEPTH {
      return Err(NodeExecutionError::Runtime(format!("sub-workflow depth limit ({MAX_SUB_DEPTH}) reached")));
    }

    let flow_id = properties_lookup(&ctx.node.properties, "flow_id").as_str().map(ToOwned::to_owned).unwrap_or_default();
    if flow_id.is_empty() {
      return Err(NodeExecutionError::Runtime("control_sub_workflow: `flow_id` is required".to_string()));
    }

    let document = self.persistence.load_or_seed(&flow_id).map_err(|error| NodeExecutionError::Runtime(format!("load sub-flow `{flow_id}`: {error}")))?;
    let templates = crate::pages::flows::registry::sample_workflow_registry().templates();

    let inputs = merged_inputs(&ctx.inputs);
    let mut child_registry = (*self.registry).clone();
    child_registry.register("control_sub_workflow", Arc::new(self.deeper()));

    let (sub_report, sub_context) = execute_flow(&document, &templates, &child_registry, &self.credentials).await;

    let mut outputs = HashMap::new();
    outputs.insert(
      "out".to_string(),
      vec![NodeItem::from_json(json!({
        "sub_flow_id": flow_id,
        "inputs": inputs.iter().map(|item| item.json.clone()).collect::<Vec<_>>(),
        "aborted": sub_report.aborted,
        "error": sub_report.error,
        "success_count": sub_report.success_count(),
        "failed_count": sub_report.failed_count(),
        "skipped_count": sub_report.skipped_count(),
        "terminal_outputs": terminal_outputs(&sub_context),
      }))],
    );

    Ok(NodeRunOutcome { outputs, logs: vec![format!("Sub-flow `{flow_id}` ran ({} ok / {} err)", sub_report.success_count(), sub_report.failed_count())] })
  }
}

fn terminal_outputs(context: &ExecutionContext) -> serde_json::Value {
  let snapshot = context.snapshot();
  serde_json::to_value(&snapshot).unwrap_or(serde_json::Value::Null)
}

pub fn register_sub_workflow_runner(registry: &mut NodeRunnerRegistry, persistence: FlowPersistence, credentials: CredentialStore) {
  let base_registry = Arc::new(registry.clone());
  registry.register("control_sub_workflow", Arc::new(SubWorkflowRunner::new(persistence, base_registry, credentials)));
}

pub fn _compile_time_report_assertion(_report: &FlowExecutionReport) {}
