use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeItem {
  pub json: Value,
  #[serde(default, skip_serializing_if = "HashMap::is_empty")]
  pub binary: HashMap<String, BinaryData>,
}

impl NodeItem {
  pub fn from_json(json: Value) -> Self {
    Self { json, binary: HashMap::new() }
  }

  pub fn with_binary(mut self, key: impl Into<String>, data: BinaryData) -> Self {
    self.binary.insert(key.into(), data);
    self
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BinaryData {
  pub mime_type: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub file_name: Option<String>,
  pub data_base64: String,
}

pub type NodeExecutionData = Vec<NodeItem>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodeRunOutcome {
  pub outputs: HashMap<String, NodeExecutionData>,
  pub logs: Vec<String>,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum NodeExecutionError {
  #[error("missing required input `{0}`")]
  MissingInput(String),
  #[error("no runner registered for template `{0}`")]
  RunnerNotFound(String),
  #[error("template `{0}` not found in document templates")]
  TemplateNotFound(String),
  #[error("{0}")]
  Runtime(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeExecutionStatus {
  Success,
  Failed,
  Skipped,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeExecutionRecord {
  pub node_id: String,
  pub template_id: String,
  pub status: NodeExecutionStatus,
  pub logs: Vec<String>,
  pub error: Option<String>,
  pub started_at: String,
  pub finished_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FlowExecutionReport {
  pub records: Vec<NodeExecutionRecord>,
  pub aborted: bool,
  pub error: Option<String>,
  pub started_at: String,
  pub finished_at: Option<String>,
}

impl FlowExecutionReport {
  pub fn success_count(&self) -> usize {
    self.records.iter().filter(|record| record.status == NodeExecutionStatus::Success).count()
  }

  pub fn failed_count(&self) -> usize {
    self.records.iter().filter(|record| record.status == NodeExecutionStatus::Failed).count()
  }

  pub fn skipped_count(&self) -> usize {
    self.records.iter().filter(|record| record.status == NodeExecutionStatus::Skipped).count()
  }
}

pub(crate) fn now_ts() -> String {
  std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|duration| duration.as_millis().to_string()).unwrap_or_default()
}
