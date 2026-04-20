use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use dioxus::prelude::*;
use tokio::sync::Mutex;

use super::backend::{BackendDispatch, dispatch_backend_command, spawn_backend_agent};
use super::commands::{DesktopRuntimeCommand, command_message};
use super::pi_backend::PiProcessHandle;
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
      spawn_backend_agent(processes, registry, agent, prompt).await;
    });
    agent_id
  }

  pub fn create_flow_run(&self, flow_id: String, title: String) -> String {
    let run_id = new_id("flow-run");
    self.registry.register_flow_run(DesktopFlowRun { id: run_id.clone(), flow_id, root_agent_id: None, title, created_at: now_ts() });
    run_id
  }

  pub fn launch_pi_agent_for_flow_run(&self, flow_id: String, flow_run_id: &str, spec: &DesktopAgentLaunchSpec) -> String {
    let mut spec = spec.clone();
    spec.flow_id = Some(flow_id);
    spec.flow_run_id = Some(flow_run_id.to_string());
    let agent_id = self.launch_pi_agent(&spec);
    if spec.parent_id.is_none() {
      self.registry.update_flow_run(flow_run_id, |run| {
        if run.root_agent_id.is_none() {
          run.root_agent_id = Some(agent_id.clone());
        }
      });
    }
    agent_id
  }

  pub fn launch_flow_pi_agent(&self, flow_id: String, name: &str, prompt: String, cwd: Option<PathBuf>) -> String {
    let run_id = self.create_flow_run(flow_id.clone(), name.to_string());
    let mut spec = DesktopAgentLaunchSpec::new(name.to_string(), format!("Flow run for {flow_id}"), prompt);
    spec.cwd = cwd;
    self.launch_pi_agent_for_flow_run(flow_id, &run_id, &spec)
  }

  pub fn prompt(&self, agent_id: String, message: String) {
    self.dispatch(agent_id, DesktopRuntimeCommand::Prompt { message });
  }

  pub fn steer(&self, agent_id: String, message: String) {
    self.dispatch(agent_id, DesktopRuntimeCommand::Steer { message });
  }

  pub fn follow_up(&self, agent_id: String, message: String) {
    self.dispatch(agent_id, DesktopRuntimeCommand::FollowUp { message });
  }

  pub fn abort(&self, agent_id: String) {
    self.registry.update_agent(&agent_id, |agent| agent.status = DesktopAgentStatus::Aborted);
    self.dispatch(agent_id, DesktopRuntimeCommand::Abort);
  }

  pub fn pause(&self, agent_id: String) {
    self.dispatch(agent_id, DesktopRuntimeCommand::Pause);
  }

  pub fn resume(&self, agent_id: String) {
    self.dispatch(agent_id, DesktopRuntimeCommand::Resume);
  }

  fn dispatch(&self, agent_id: String, command: DesktopRuntimeCommand) {
    self.record_local_command(&agent_id, &command);
    let Some(agent) = self.registry.find_agent(&agent_id) else {
      self.registry.append_event(DesktopRuntimeEvent::new(agent_id, DesktopRuntimeEventKind::BackendError, text_payload("system", "Runtime agent not found")));
      return;
    };
    let processes = Arc::clone(&self.processes);
    let registry = self.registry.clone();
    spawn(async move {
      match dispatch_backend_command(&processes, &agent, command.clone()).await {
        Ok(BackendDispatch::Sent) => {},
        Ok(BackendDispatch::Unsupported(reason)) => {
          registry.append_event(DesktopRuntimeEvent::new(agent.id.clone(), DesktopRuntimeEventKind::ControlState, text_payload("system", reason)))
        },
        Err(error) => registry.append_event(DesktopRuntimeEvent::new(
          agent.id.clone(),
          DesktopRuntimeEventKind::BackendError,
          text_payload("system", format!("{} failed: {error}", command.label())),
        )),
      }
    });
  }

  fn record_local_command(&self, agent_id: &str, command: &DesktopRuntimeCommand) {
    match command {
      DesktopRuntimeCommand::Prompt { .. } => {
        if let Some(message) = command_message(command) {
          self.registry.append_event(DesktopRuntimeEvent::new(agent_id.to_string(), DesktopRuntimeEventKind::MessageComplete, text_payload("user", message)));
        }
      },
      DesktopRuntimeCommand::Steer { .. } | DesktopRuntimeCommand::FollowUp { .. } => {
        if let Some(message) = command_message(command) {
          self.registry.append_event(DesktopRuntimeEvent::new(
            agent_id.to_string(),
            DesktopRuntimeEventKind::ControlState,
            text_payload("system", format!("Queued {}: {message}", command.label())),
          ));
        }
      },
      DesktopRuntimeCommand::Abort => self.registry.append_event(DesktopRuntimeEvent::new(
        agent_id.to_string(),
        DesktopRuntimeEventKind::ControlState,
        text_payload("system", "Abort requested"),
      )),
      DesktopRuntimeCommand::Pause | DesktopRuntimeCommand::Resume | DesktopRuntimeCommand::RefreshState => {},
    }
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
  let controller = use_context::<DesktopRuntimeController>();
  let registry = controller.registry.clone();
  let mut revision = use_signal(|| registry.revision());

  use_hook(move || {
    let mut updates = registry.subscribe();
    spawn(async move {
      while updates.changed().await.is_ok() {
        revision.set(*updates.borrow_and_update());
      }
    });
  });

  let _ = revision();
  controller
}
