use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot};

use crate::value::LxVal;

pub struct AgentMessage {
  pub payload: LxVal,
  pub reply: Option<oneshot::Sender<LxVal>>,
}

pub struct AgentHandle {
  pub name: String,
  pub mailbox: mpsc::Sender<AgentMessage>,
  pub task: tokio::task::JoinHandle<()>,
  pub pause_flag: Arc<std::sync::atomic::AtomicBool>,
}

static AGENT_REGISTRY: LazyLock<DashMap<String, AgentHandle>> = LazyLock::new(DashMap::new);

pub fn register_agent(name: String, handle: AgentHandle) -> Result<(), String> {
  if AGENT_REGISTRY.contains_key(&name) {
    return Err(format!("agent '{name}' already running"));
  }
  AGENT_REGISTRY.insert(name, handle);
  Ok(())
}

pub fn get_agent_mailbox(name: &str) -> Option<mpsc::Sender<AgentMessage>> {
  AGENT_REGISTRY.get(name).map(|e| e.mailbox.clone())
}

pub fn remove_agent(name: &str) -> Option<(String, AgentHandle)> {
  AGENT_REGISTRY.remove(name)
}

pub fn agent_exists(name: &str) -> bool {
  AGENT_REGISTRY.contains_key(name)
}

pub fn agent_names() -> Vec<String> {
  AGENT_REGISTRY.iter().map(|e| e.key().clone()).collect()
}

pub fn get_agent_entry(name: &str) -> Option<dashmap::mapref::one::Ref<'_, String, AgentHandle>> {
  AGENT_REGISTRY.get(name)
}

pub fn get_agent_pause_flag(name: &str) -> Option<Arc<std::sync::atomic::AtomicBool>> {
  AGENT_REGISTRY.get(name).map(|e| Arc::clone(&e.pause_flag))
}
