use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use super::commands::DesktopRuntimeCommand;
use super::pi_backend::{PiProcessHandle, dispatch_pi_command, spawn_pi_agent};
use super::registry::DesktopRuntimeRegistry;
use super::types::{DesktopAgentRuntime, DesktopBackendKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendDispatch {
  Sent,
  Unsupported(&'static str),
}

pub async fn spawn_backend_agent(
  processes: Arc<Mutex<HashMap<String, PiProcessHandle>>>,
  registry: DesktopRuntimeRegistry,
  agent: DesktopAgentRuntime,
  initial_prompt: String,
) {
  match agent.backend_kind {
    DesktopBackendKind::Pi => spawn_pi_agent(processes, registry, agent, initial_prompt).await,
  }
}

pub async fn dispatch_backend_command(
  processes: &Arc<Mutex<HashMap<String, PiProcessHandle>>>,
  agent: &DesktopAgentRuntime,
  command: DesktopRuntimeCommand,
) -> Result<BackendDispatch, String> {
  match agent.backend_kind {
    DesktopBackendKind::Pi => dispatch_pi_command(processes, &agent.id, command).await,
  }
}
