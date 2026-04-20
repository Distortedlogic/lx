use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use lx_graph_editor::catalog::GraphNodeTemplate;
use lx_graph_editor::model::GraphNode;

use crate::credentials::CredentialStore;

use super::context::ExecutionContext;
use super::types::{NodeExecutionData, NodeExecutionError, NodeRunOutcome};

pub struct NodeRunContext<'a> {
  pub node: &'a GraphNode,
  pub template: &'a GraphNodeTemplate,
  pub inputs: HashMap<String, NodeExecutionData>,
  pub exec: &'a ExecutionContext,
  pub credentials: &'a CredentialStore,
}

#[async_trait]
pub trait NodeRunner: Send + Sync {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError>;
}

#[derive(Default, Clone)]
pub struct NodeRunnerRegistry {
  runners: HashMap<String, Arc<dyn NodeRunner>>,
}

impl NodeRunnerRegistry {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn register(&mut self, template_id: impl Into<String>, runner: Arc<dyn NodeRunner>) {
    self.runners.insert(template_id.into(), runner);
  }

  pub fn get(&self, template_id: &str) -> Option<Arc<dyn NodeRunner>> {
    self.runners.get(template_id).cloned()
  }

  pub fn contains(&self, template_id: &str) -> bool {
    self.runners.contains_key(template_id)
  }
}
