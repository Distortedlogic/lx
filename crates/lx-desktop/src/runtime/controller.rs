use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use dioxus::prelude::*;
use tokio::sync::Mutex;

use super::pi_backend::{PiProcessHandle, send_command, spawn_pi_agent};
use super::registry::DesktopRuntimeRegistry;
use super::types::{
  DesktopAgentLaunchSpec, DesktopAgentRuntime, DesktopAgentStatus, DesktopFlowRun, DesktopRuntimeEvent, DesktopRuntimeEventKind, new_id, now_ts, text_payload,
};

#[derive(Clone)]
pub struct DesktopRuntimeController {
  pub registry: DesktopRuntimeRegistry,
  processes: Arc<Mutex<HashMap<String, PiProcessHandle>>>,
}

impl Default for DesktopRuntimeController {
  fn default() -> Self {
    Self::new()
  }
}

impl DesktopRuntimeController {
  pub fn new() -> Self {
    Self { registry: DesktopRuntimeRegistry::new(), processes: Arc::new(Mutex::new(HashMap::new())) }
  }

  pub fn launch_pi_agent(&self, spec: &DesktopAgentLaunchSpec) -> String {
    let agent = DesktopAgentRuntime::new(spec);
    let agent_id = agent.id.clone();
    let prompt = spec.prompt.clone();
    let processes = Arc::clone(&self.processes);
    let registry = self.registry.clone();
    registry.register_agent(agent.clone());
    registry.append_event(DesktopRuntimeEvent::new(agent_id.clone(), DesktopRuntimeEventKind::MessageComplete, text_payload("user", prompt.clone())));
    spawn(async move {
      spawn_pi_agent(processes, registry, agent, prompt).await;
    });
    agent_id
  }

  pub fn launch_flow_pi_agent(&self, flow_id: String, name: &str, prompt: String, cwd: Option<PathBuf>) -> String {
    let mut spec = DesktopAgentLaunchSpec::new(name.to_string(), format!("Flow run for {flow_id}"), prompt);
    spec.flow_id = Some(flow_id.clone());
    spec.cwd = cwd;
    let agent_id = self.launch_pi_agent(&spec);
    self.registry.register_flow_run(DesktopFlowRun { id: new_id("flow-run"), flow_id, root_agent_id: agent_id.clone(), created_at: now_ts() });
    agent_id
  }

  pub fn prompt(&self, agent_id: String, message: String) {
    self.registry.append_event(DesktopRuntimeEvent::new(agent_id.clone(), DesktopRuntimeEventKind::MessageComplete, text_payload("user", message.clone())));
    let processes = Arc::clone(&self.processes);
    spawn(async move {
      let _ = send_command(&processes, &agent_id, serde_json::json!({ "type": "prompt", "message": message })).await;
    });
  }

  pub fn steer(&self, agent_id: String, message: String) {
    self.registry.append_event(DesktopRuntimeEvent::new(
      agent_id.clone(),
      DesktopRuntimeEventKind::ControlState,
      text_payload("system", format!("Queued steer: {message}")),
    ));
    let processes = Arc::clone(&self.processes);
    spawn(async move {
      let _ = send_command(&processes, &agent_id, serde_json::json!({ "type": "steer", "message": message })).await;
    });
  }

  pub fn follow_up(&self, agent_id: String, message: String) {
    self.registry.append_event(DesktopRuntimeEvent::new(
      agent_id.clone(),
      DesktopRuntimeEventKind::ControlState,
      text_payload("system", format!("Queued follow-up: {message}")),
    ));
    let processes = Arc::clone(&self.processes);
    spawn(async move {
      let _ = send_command(&processes, &agent_id, serde_json::json!({ "type": "follow_up", "message": message })).await;
    });
  }

  pub fn abort(&self, agent_id: String) {
    self.registry.update_agent(&agent_id, |agent| agent.status = DesktopAgentStatus::Aborted);
    self.registry.append_event(DesktopRuntimeEvent::new(agent_id.clone(), DesktopRuntimeEventKind::ControlState, text_payload("system", "Abort requested")));
    let processes = Arc::clone(&self.processes);
    spawn(async move {
      let _ = send_command(&processes, &agent_id, serde_json::json!({ "type": "abort" })).await;
    });
  }

  pub fn pause(&self, agent_id: String) {
    self.registry.append_event(DesktopRuntimeEvent::new(
      agent_id,
      DesktopRuntimeEventKind::ControlState,
      text_payload("system", "Pause is not supported by the Pi backend"),
    ));
  }

  pub fn resume(&self, agent_id: String) {
    self.registry.append_event(DesktopRuntimeEvent::new(
      agent_id,
      DesktopRuntimeEventKind::ControlState,
      text_payload("system", "Resume is not supported by the Pi backend"),
    ));
  }
}

#[component]
pub fn DesktopRuntimeProvider(children: Element) -> Element {
  let controller = use_hook(DesktopRuntimeController::new);
  use_context_provider(|| controller.clone());
  rsx! {
    {children}
  }
}

pub fn use_desktop_runtime() -> DesktopRuntimeController {
  use_context()
}
