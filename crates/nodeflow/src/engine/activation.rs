use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActivationState {
  pub flows: HashMap<String, bool>,
}

pub fn load_activation_state() -> ActivationState {
  let path = activation_path();
  if !path.exists() {
    return ActivationState::default();
  }
  fs::read_to_string(&path).ok().and_then(|payload| serde_json::from_str(&payload).ok()).unwrap_or_default()
}

pub fn save_activation_state(state: &ActivationState) -> Result<()> {
  let path = activation_path();
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).with_context(|| format!("failed to create `{}`", parent.display()))?;
  }
  let payload = serde_json::to_string_pretty(state).context("failed to serialize activation state")?;
  fs::write(&path, payload).with_context(|| format!("failed to write `{}`", path.display()))
}

pub fn set_flow_active(flow_id: &str, active: bool) -> Result<()> {
  let mut state = load_activation_state();
  if active {
    state.flows.insert(flow_id.to_string(), true);
  } else {
    state.flows.remove(flow_id);
  }
  save_activation_state(&state)
}

fn activation_path() -> PathBuf {
  let base = dirs::data_local_dir().or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share"))).unwrap_or_else(|| PathBuf::from("/tmp"));
  base.join("nodeflow").join("activations.json")
}
