use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ToastTone {
  #[default]
  Info,
  Success,
  Warn,
  Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastAction {
  pub label: String,
  pub href: String,
}

#[derive(Clone, Debug)]
pub struct ToastInput {
  pub title: String,
  pub body: Option<String>,
  pub tone: ToastTone,
  pub ttl_ms: Option<u64>,
  pub action: Option<ToastAction>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastItem {
  pub id: String,
  pub title: String,
  pub body: Option<String>,
  pub tone: ToastTone,
  pub ttl_ms: u64,
  pub action: Option<ToastAction>,
  pub created_at: u64,
}

const MAX_TOASTS: usize = 5;

#[derive(Clone, Copy)]
pub struct ToastState {
  pub toasts: Signal<Vec<ToastItem>>,
}

impl ToastState {
  pub fn provide() -> Self {
    let state = Self { toasts: Signal::new(Vec::new()) };
    use_context_provider(|| state);
    state
  }

  pub fn push(&self, input: ToastInput) -> String {
    let tone = input.tone;
    let ttl_ms = input.ttl_ms.unwrap_or_else(|| default_ttl(tone));
    let id = format!("toast_{}_{}", timestamp_ms(), random_suffix());
    let item = ToastItem { id: id.clone(), title: input.title, body: input.body, tone, ttl_ms, action: input.action, created_at: timestamp_ms() };
    let mut toasts = self.toasts;
    let mut list = toasts.write();
    list.insert(0, item);
    list.truncate(MAX_TOASTS);
    id
  }

  pub fn dismiss(&self, id: &str) {
    let mut toasts = self.toasts;
    toasts.write().retain(|t| t.id != id);
  }

  pub fn clear(&self) {
    let mut toasts = self.toasts;
    toasts.write().clear();
  }
}

fn timestamp_ms() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn random_suffix() -> String {
  uuid::Uuid::new_v4().to_string()[..8].to_string()
}

fn default_ttl(tone: ToastTone) -> u64 {
  match tone {
    ToastTone::Info => 4000,
    ToastTone::Success => 3500,
    ToastTone::Warn => 8000,
    ToastTone::Error => 10000,
  }
}
