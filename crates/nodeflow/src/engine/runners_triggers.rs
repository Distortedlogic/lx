use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{merged_inputs, properties_lookup};
use super::types::{NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_trigger_runners(registry: &mut NodeRunnerRegistry) {
  registry.register("trigger_manual", Arc::new(ManualTriggerRunner));
  registry.register("trigger_cron", Arc::new(CronTriggerRunner));
  registry.register("trigger_webhook", Arc::new(WebhookTriggerRunner));
}

pub struct ManualTriggerRunner;

#[async_trait]
impl NodeRunner for ManualTriggerRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let merged = merged_inputs(&ctx.inputs);
    let payload_raw = properties_lookup(&ctx.node.properties, "payload");
    let payload = parse_payload(&payload_raw);

    let items =
      if merged.is_empty() { vec![NodeItem::from_json(payload.unwrap_or_else(|| json!({ "triggered_at": super::types::now_ts() })))] } else { merged };

    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), items);
    Ok(NodeRunOutcome { outputs, logs: vec!["Manual trigger fired".to_string()] })
  }
}

pub struct CronTriggerRunner;

#[async_trait]
impl NodeRunner for CronTriggerRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let cron_expression = properties_lookup(&ctx.node.properties, "cron_expression").as_str().unwrap_or("").to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let item = NodeItem::from_json(json!({ "triggered_at": now, "cron_expression": cron_expression }));
    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), vec![item]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Cron tick at {now}")] })
  }
}

pub struct WebhookTriggerRunner;

#[async_trait]
impl NodeRunner for WebhookTriggerRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let merged = merged_inputs(&ctx.inputs);
    let path = properties_lookup(&ctx.node.properties, "path").as_str().unwrap_or("").to_string();
    let method = properties_lookup(&ctx.node.properties, "method").as_str().unwrap_or("ANY").to_string();
    let items = if merged.is_empty() {
      vec![NodeItem::from_json(json!({
        "path": path,
        "method": method,
        "note": "invoked via manual run; webhook listener supplies real request on production runs",
      }))]
    } else {
      merged
    };
    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), items);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Webhook trigger `{path}` ({method})")] })
  }
}

fn parse_payload(raw: &Value) -> Option<Value> {
  match raw {
    Value::Null => None,
    Value::String(text) => serde_json::from_str(text).ok().or_else(|| Some(Value::String(text.clone()))),
    other => Some(other.clone()),
  }
}
