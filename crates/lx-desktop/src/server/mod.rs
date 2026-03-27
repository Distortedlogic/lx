mod activity_api;
mod run_api;
mod settings_api;
mod ws_events;

use std::collections::VecDeque;
use std::sync::LazyLock;

use tokio::sync::{RwLock, broadcast};

use crate::contexts::activity_log::ActivityEvent;
use crate::pages::settings::state::SettingsData;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct RunStatus {
  pub status: String,
  pub source_path: Option<String>,
  pub elapsed_ms: Option<u64>,
  pub cost: Option<f64>,
  pub error: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PendingPrompt {
  pub prompt_id: u64,
  pub kind: String,
  pub message: String,
  pub options: Option<Vec<String>>,
}

pub struct ServerState {
  pub settings: RwLock<SettingsData>,
  pub settings_path: String,
  pub activity: RwLock<VecDeque<ActivityEvent>>,
  pub run_status: RwLock<RunStatus>,
  pub prompts: RwLock<Vec<PendingPrompt>>,
  pub event_tx: broadcast::Sender<ActivityEvent>,
}

pub static STATE: LazyLock<ServerState> = LazyLock::new(|| {
  let settings_path = dirs_or_default("lx", "settings.json");
  let settings = load_settings(&settings_path);
  let (event_tx, _) = broadcast::channel(1024);
  ServerState {
    settings: RwLock::new(settings),
    settings_path,
    activity: RwLock::new(VecDeque::new()),
    run_status: RwLock::new(RunStatus::default()),
    prompts: RwLock::new(Vec::new()),
    event_tx,
  }
});

fn dirs_or_default(app: &str, file: &str) -> String {
  if let Some(config_dir) = dirs::config_dir() {
    let dir = config_dir.join(app);
    let _ = std::fs::create_dir_all(&dir);
    dir.join(file).display().to_string()
  } else {
    format!(".{app}_{file}")
  }
}

fn load_settings(path: &str) -> SettingsData {
  std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}
