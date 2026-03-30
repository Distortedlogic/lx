use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use lx_value::{EventStream, LxError, LxVal};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd")]
pub enum ControlCommand {
  #[serde(rename = "pause")]
  Pause { agent: Option<String> },
  #[serde(rename = "resume")]
  Resume { agent: Option<String> },
  #[serde(rename = "cancel")]
  Cancel,
  #[serde(rename = "inspect")]
  Inspect,
  #[serde(rename = "inject")]
  Inject { value: serde_json::Value },
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlResponse {
  pub ok: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub state: Option<InspectState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectState {
  pub paused: bool,
  pub agents: Vec<AgentInspect>,
  pub stream_position: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentInspect {
  pub name: String,
  pub paused: bool,
}

impl ControlResponse {
  pub fn ok() -> Self {
    Self { ok: true, error: None, state: None }
  }

  pub fn err(msg: impl Into<String>) -> Self {
    Self { ok: false, error: Some(msg.into()), state: None }
  }

  pub fn with_state(state: InspectState) -> Self {
    Self { ok: true, error: None, state: Some(state) }
  }
}

pub struct ControlChannelState {
  pub global_pause: Arc<AtomicBool>,
  pub cancel_flag: Arc<AtomicBool>,
  pub inject_tx: Option<tokio::sync::mpsc::Sender<LxVal>>,
  pub event_stream: Arc<EventStream>,
}

pub fn handle_command(cmd: ControlCommand, state: &ControlChannelState) -> ControlResponse {
  match cmd {
    ControlCommand::Pause { agent } => handle_pause(agent, state),
    ControlCommand::Resume { agent } => handle_resume(agent, state),
    ControlCommand::Cancel => handle_cancel(state),
    ControlCommand::Inspect => handle_inspect(state),
    ControlCommand::Inject { value } => handle_inject(value, state),
  }
}

fn handle_pause(agent: Option<String>, state: &ControlChannelState) -> ControlResponse {
  match agent {
    None => {
      state.global_pause.store(true, Ordering::SeqCst);
      ControlResponse::ok()
    },
    Some(name) => match crate::runtime::agent_registry::get_agent_pause_flag(&name) {
      Some(flag) => {
        flag.store(true, Ordering::SeqCst);
        ControlResponse::ok()
      },
      None => ControlResponse::err(format!("agent '{name}' not found")),
    },
  }
}

fn handle_resume(agent: Option<String>, state: &ControlChannelState) -> ControlResponse {
  match agent {
    None => {
      state.global_pause.store(false, Ordering::SeqCst);
      ControlResponse::ok()
    },
    Some(name) => match crate::runtime::agent_registry::get_agent_pause_flag(&name) {
      Some(flag) => {
        flag.store(false, Ordering::SeqCst);
        ControlResponse::ok()
      },
      None => ControlResponse::err(format!("agent '{name}' not found")),
    },
  }
}

fn handle_cancel(state: &ControlChannelState) -> ControlResponse {
  state.cancel_flag.store(true, Ordering::SeqCst);
  state.global_pause.store(false, Ordering::SeqCst);
  ControlResponse::ok()
}

fn handle_inspect(state: &ControlChannelState) -> ControlResponse {
  let paused = state.global_pause.load(Ordering::SeqCst);
  let agent_names = crate::runtime::agent_registry::agent_names();
  let agents: Vec<AgentInspect> = agent_names
    .into_iter()
    .map(|name| {
      let agent_paused = crate::runtime::agent_registry::get_agent_pause_flag(&name).map(|f| f.load(Ordering::Relaxed)).unwrap_or(false);
      AgentInspect { name, paused: agent_paused }
    })
    .collect();
  let last_entries = state.event_stream.xrange("-", "+", None);
  let stream_position = last_entries.last().map(|e| e.id.clone()).unwrap_or_else(|| String::from("0-0"));
  ControlResponse::with_state(InspectState { paused, agents, stream_position })
}

fn handle_inject(value: serde_json::Value, state: &ControlChannelState) -> ControlResponse {
  let lx_val = LxVal::from(value);
  match &state.inject_tx {
    Some(tx) => match tx.try_send(lx_val) {
      Ok(()) => ControlResponse::ok(),
      Err(_) => ControlResponse::err("no pending yield to inject into"),
    },
    None => ControlResponse::err("inject not available (no yield backend configured)"),
  }
}

pub struct ControlYieldBackend {
  pub inject_rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<LxVal>>>,
}

impl crate::runtime::YieldBackend for ControlYieldBackend {
  fn yield_value(&self, value: LxVal, span: miette::SourceSpan) -> Result<LxVal, LxError> {
    println!("{}", serde_json::to_string(&value).unwrap_or_else(|_| value.to_string()));
    let rx = Arc::clone(&self.inject_rx);
    tokio::task::block_in_place(|| {
      tokio::runtime::Handle::current().block_on(async {
        let mut guard = rx.lock().await;
        guard.recv().await.ok_or_else(|| LxError::runtime("yield: inject channel closed", span))
      })
    })
  }
}
