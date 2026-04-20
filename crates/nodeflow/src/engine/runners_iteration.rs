use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{merged_inputs, properties_lookup};
use super::types::{NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_iteration_runners(registry: &mut NodeRunnerRegistry) {
  registry.register("control_split_out", Arc::new(SplitOutRunner));
}

pub struct SplitOutRunner;

#[async_trait]
impl NodeRunner for SplitOutRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let field = properties_lookup(&ctx.node.properties, "field").as_str().map(ToOwned::to_owned).unwrap_or_else(|| "items".to_string());
    let inputs = merged_inputs(&ctx.inputs);

    let mut out_items: Vec<NodeItem> = Vec::new();
    for item in inputs {
      let target = if field.is_empty() { &item.json } else { item.json.get(&field).unwrap_or(&Value::Null) };
      match target {
        Value::Array(elements) => {
          for element in elements {
            out_items.push(NodeItem::from_json(element.clone()));
          }
        },
        Value::Null => {},
        other => out_items.push(NodeItem::from_json(other.clone())),
      }
    }

    let mut outputs = HashMap::new();
    let count = out_items.len();
    outputs.insert("out".to_string(), out_items);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("SplitOut emitted {count} items from `{field}`")] })
  }
}
