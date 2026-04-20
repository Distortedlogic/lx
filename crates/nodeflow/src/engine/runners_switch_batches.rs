use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use super::expression::resolve_string;
use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{make_expr_ctx, merged_inputs, properties_lookup};
use super::types::{NodeExecutionData, NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_switch_and_batches(registry: &mut NodeRunnerRegistry) {
  registry.register("control_switch", Arc::new(SwitchRunner));
  registry.register("control_split_in_batches", Arc::new(SplitInBatchesRunner));
}

pub struct SwitchRunner;

#[async_trait]
impl NodeRunner for SwitchRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let items = merged_inputs(&ctx.inputs);
    let props = ctx.node.properties.clone();
    let mut outputs: HashMap<String, NodeExecutionData> = HashMap::new();
    for port in ["case_1", "case_2", "case_3", "default"] {
      outputs.insert(port.to_string(), Vec::new());
    }

    for item in items {
      let expr_ctx = make_expr_ctx(ctx.exec, Some(&item), &ctx.node.id);
      let mut placed = false;
      for (rule_field, port) in [("rule_1", "case_1"), ("rule_2", "case_2"), ("rule_3", "case_3")] {
        let rule = resolve_string(&properties_lookup(&props, rule_field), &expr_ctx, ctx.credentials).unwrap_or_default();
        if rule.trim().is_empty() {
          continue;
        }
        if is_truthy(&rule) {
          outputs.entry(port.to_string()).or_default().push(item.clone());
          placed = true;
          break;
        }
      }
      if !placed {
        outputs.entry("default".to_string()).or_default().push(item);
      }
    }

    let summary = ["case_1", "case_2", "case_3", "default"]
      .iter()
      .map(|port| format!("{port}={}", outputs.get(*port).map(|items| items.len()).unwrap_or(0)))
      .collect::<Vec<_>>()
      .join(" ");
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Switch routed {summary}")] })
  }
}

pub struct SplitInBatchesRunner;

#[async_trait]
impl NodeRunner for SplitInBatchesRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let items = merged_inputs(&ctx.inputs);
    let batch_size =
      properties_lookup(&ctx.node.properties, "batch_size").as_u64().and_then(|value| usize::try_from(value).ok()).filter(|size| *size > 0).unwrap_or(10);

    let mut out_items: Vec<NodeItem> = Vec::new();
    for (batch_index, chunk) in items.chunks(batch_size).enumerate() {
      let payload: Vec<Value> = chunk.iter().map(|item| item.json.clone()).collect();
      out_items.push(NodeItem::from_json(json!({
        "batch_index": batch_index,
        "batch_size": chunk.len(),
        "items": payload,
      })));
    }

    let mut outputs = HashMap::new();
    let count = out_items.len();
    outputs.insert("batches".to_string(), out_items);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("SplitInBatches produced {count} batches of up to {batch_size}")] })
  }
}

fn is_truthy(value: &str) -> bool {
  let trimmed = value.trim().to_lowercase();
  !matches!(trimmed.as_str(), "" | "false" | "0" | "no" | "null" | "undefined")
}
