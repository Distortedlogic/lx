#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityEvent {
  pub timestamp: String,
  pub kind: String,
  pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunStatus {
  pub status: String,
  pub source_path: Option<String>,
  pub elapsed_ms: Option<u64>,
  pub cost: Option<f64>,
  pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PendingPrompt {
  pub prompt_id: u64,
  pub kind: String,
  pub message: String,
  pub options: Option<Vec<String>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PromptResponse {
  pub prompt_id: u64,
  pub response: serde_json::Value,
}
