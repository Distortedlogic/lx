use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvEntry {
  pub key: String,
  pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingsData {
  pub env_vars: Vec<EnvEntry>,
  pub task_priority: f64,
  pub auto_scale: bool,
  pub redundant_verify: bool,
  pub compute_quota: u8,
  pub memory_quota: u8,
  pub storage_quota: u8,
}

impl Default for SettingsData {
  fn default() -> Self {
    Self {
      env_vars: vec![
        EnvEntry { key: "API_ENDPOINT_ROOT".into(), value: "https://core.monolith.io/v2".into() },
        EnvEntry { key: "MAX_CONCURRENCY".into(), value: "512".into() },
        EnvEntry { key: "RETRY_POLICY".into(), value: "EXPONENTIAL_BACKOFF".into() },
      ],
      task_priority: 0.84,
      auto_scale: true,
      redundant_verify: false,
      compute_quota: 85,
      memory_quota: 32,
      storage_quota: 95,
    }
  }
}

#[derive(Clone, Copy)]
pub struct SettingsState {
  pub data: Signal<SettingsData>,
  pub saved: Signal<SettingsData>,
}

impl SettingsState {
  pub fn provide() -> Self {
    let saved = dioxus_storage::use_persistent("lx_settings", SettingsData::default);
    let data = use_signal(|| saved.read().clone());
    let ctx = Self { data, saved };
    use_context_provider(|| ctx);
    ctx
  }

  pub fn discard(&self) {
    let mut data = self.data;
    data.set((self.saved)());
  }

  pub fn execute(&self) {
    let mut saved = self.saved;
    saved.set((self.data)());
  }
}
