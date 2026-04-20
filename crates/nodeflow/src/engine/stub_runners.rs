use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use lx_graph_editor::catalog::PortDirection;

use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::types::{NodeExecutionData, NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_passthrough_runners(registry: &mut NodeRunnerRegistry) {
  let passthrough: Arc<dyn NodeRunner> = Arc::new(PassthroughRunner);
  for template_id in ["topic_input", "curated_sources", "web_fetch", "extract_signals", "dedupe_rank", "summarize_briefs", "feed_output"] {
    registry.register(template_id, passthrough.clone());
  }
}

pub struct PassthroughRunner;

#[async_trait]
impl NodeRunner for PassthroughRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let merged: NodeExecutionData = ctx.inputs.values().flat_map(|items| items.iter().cloned()).collect();
    let payload = if merged.is_empty() { vec![NodeItem::from_json(ctx.node.properties.clone())] } else { merged };

    let mut outputs: HashMap<String, NodeExecutionData> = HashMap::new();
    for port in ctx.template.ports.iter().filter(|port| port.direction == PortDirection::Output) {
      outputs.insert(port.id.clone(), payload.clone());
    }

    Ok(NodeRunOutcome { outputs, logs: vec![format!("Passthrough ran `{}` ({} items)", ctx.node.template_id, payload.len())] })
  }
}
