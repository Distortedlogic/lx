use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;

pub use lx_api::types::ActivityEvent;

#[derive(Clone, Copy)]
pub struct ActivityLog {
  pub events: Signal<VecDeque<ActivityEvent>>,
}

impl ActivityLog {
  pub fn provide() -> Self {
    let ctx = Self { events: Signal::new(VecDeque::new()) };
    use_context_provider(|| ctx);
    ctx
  }

  pub fn push(&self, kind: &str, message: &str) {
    self.push_with_adapter(kind, message, None);
  }

  pub fn push_with_adapter(&self, kind: &str, message: &str, adapter: Option<String>) {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0).to_string();
    let event = ActivityEvent { timestamp, kind: kind.to_string(), message: message.to_string(), token_count: None, adapter };
    let mut events_sig = self.events;
    let mut events = events_sig.write();
    events.push_front(event);
    if events.len() > 500 {
      events.pop_back();
    }
  }
}
