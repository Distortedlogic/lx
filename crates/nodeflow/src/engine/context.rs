use std::collections::HashMap;

use super::types::NodeExecutionData;

#[derive(Clone, Debug, Default)]
pub struct ExecutionContext {
  node_outputs: HashMap<String, HashMap<String, NodeExecutionData>>,
}

impl ExecutionContext {
  pub fn port_items(&self, node_id: &str, port_id: &str) -> Option<&NodeExecutionData> {
    self.node_outputs.get(node_id).and_then(|ports| ports.get(port_id))
  }

  pub fn set_node_outputs(&mut self, node_id: &str, outputs: HashMap<String, NodeExecutionData>) {
    self.node_outputs.insert(node_id.to_string(), outputs);
  }

  pub fn node_outputs(&self, node_id: &str) -> Option<&HashMap<String, NodeExecutionData>> {
    self.node_outputs.get(node_id)
  }

  pub fn snapshot(&self) -> HashMap<String, HashMap<String, NodeExecutionData>> {
    self.node_outputs.clone()
  }
}
