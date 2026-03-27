#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityEvent {
  pub timestamp: String,
  pub kind: String,
  pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
  #[default]
  Idle,
  Running,
  Completed,
  Failed,
  Waiting,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunStatus {
  pub status: RunState,
  pub source_path: Option<String>,
  pub elapsed_ms: Option<u64>,
  pub cost: Option<f64>,
  pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptKind {
  Confirm,
  Choose,
  Ask,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PendingPrompt {
  pub prompt_id: u64,
  pub kind: PromptKind,
  pub message: String,
  pub options: Option<Vec<String>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PromptResponse {
  pub prompt_id: u64,
  pub response: serde_json::Value,
}
