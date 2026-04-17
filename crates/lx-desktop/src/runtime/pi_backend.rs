use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdout, Command};
use tokio::sync::{Mutex, mpsc};

use super::backend::BackendDispatch;
use super::commands::DesktopRuntimeCommand;
use super::pi_event_mapper::handle_stdout_value;
use super::registry::DesktopRuntimeRegistry;
use super::types::{DesktopAgentRuntime, DesktopAgentStatus, DesktopRuntimeEvent, DesktopRuntimeEventKind, text_payload};

#[derive(Clone)]
pub struct PiProcessHandle {
  pub command_tx: mpsc::UnboundedSender<serde_json::Value>,
}

pub async fn spawn_pi_agent(
  processes: Arc<Mutex<HashMap<String, PiProcessHandle>>>,
  registry: DesktopRuntimeRegistry,
  agent: DesktopAgentRuntime,
  prompt: String,
) {
  let mut command = Command::new("pi");
  command.arg("--mode").arg("rpc").arg("--no-session");
  if let Some(cwd) = agent.cwd.as_ref().map(PathBuf::from) {
    command.current_dir(cwd);
  }
  command.stdin(std::process::Stdio::piped());
  command.stdout(std::process::Stdio::piped());
  command.stderr(std::process::Stdio::piped());

  let Ok(mut child) = command.spawn() else {
    registry.update_agent(&agent.id, |entry| entry.status = DesktopAgentStatus::Error);
    registry.append_event(DesktopRuntimeEvent::new(agent.id.clone(), DesktopRuntimeEventKind::BackendError, text_payload("system", "Failed to spawn pi")));
    return;
  };

  let Some(mut stdin) = child.stdin.take() else {
    registry.append_event(DesktopRuntimeEvent::new(
      agent.id.clone(),
      DesktopRuntimeEventKind::BackendError,
      text_payload("system", "Pi stdin was unavailable"),
    ));
    return;
  };
  let Some(stdout) = child.stdout.take() else {
    registry.append_event(DesktopRuntimeEvent::new(
      agent.id.clone(),
      DesktopRuntimeEventKind::BackendError,
      text_payload("system", "Pi stdout was unavailable"),
    ));
    return;
  };
  let Some(stderr) = child.stderr.take() else {
    registry.append_event(DesktopRuntimeEvent::new(
      agent.id.clone(),
      DesktopRuntimeEventKind::BackendError,
      text_payload("system", "Pi stderr was unavailable"),
    ));
    return;
  };

  let (command_tx, mut command_rx) = mpsc::unbounded_channel::<serde_json::Value>();
  processes.lock().await.insert(agent.id.clone(), PiProcessHandle { command_tx: command_tx.clone() });

  let writer_agent_id = agent.id.clone();
  let writer_registry = registry.clone();
  tokio::task::spawn_local(async move {
    while let Some(value) = command_rx.recv().await {
      match serde_json::to_string(&value) {
        Ok(line) => {
          if stdin.write_all(format!("{line}\n").as_bytes()).await.is_err() || stdin.flush().await.is_err() {
            writer_registry.append_event(DesktopRuntimeEvent::new(
              writer_agent_id.clone(),
              DesktopRuntimeEventKind::BackendError,
              text_payload("system", "Failed to write command to pi"),
            ));
            break;
          }
        },
        Err(error) => writer_registry.append_event(DesktopRuntimeEvent::new(
          writer_agent_id.clone(),
          DesktopRuntimeEventKind::BackendError,
          text_payload("system", format!("Failed to encode command: {error}")),
        )),
      }
    }
  });

  tokio::task::spawn_local(read_stdout(stdout, registry.clone(), agent.id.clone()));
  tokio::task::spawn_local(read_stderr(stderr, registry.clone(), agent.id.clone()));
  let _ = command_tx.send(serde_json::json!({ "type": "get_state" }));
  let _ = command_tx.send(serde_json::json!({ "type": "prompt", "message": prompt }));

  match child.wait().await {
    Ok(_status) => {
      processes.lock().await.remove(&agent.id);
      if let Some(current) = registry.find_agent(&agent.id)
        && current.status != DesktopAgentStatus::Error
        && current.status != DesktopAgentStatus::Aborted
      {
        registry.update_agent(&agent.id, |entry| entry.status = DesktopAgentStatus::Completed);
      }
    },
    Err(error) => {
      registry.append_event(DesktopRuntimeEvent::new(
        agent.id.clone(),
        DesktopRuntimeEventKind::BackendError,
        text_payload("system", format!("Pi process wait failed: {error}")),
      ));
      processes.lock().await.remove(&agent.id);
    },
  }
}

pub async fn dispatch_pi_command(
  processes: &Arc<Mutex<HashMap<String, PiProcessHandle>>>,
  agent_id: &str,
  command: DesktopRuntimeCommand,
) -> Result<BackendDispatch, String> {
  let Some(rpc_command) = command_to_pi_rpc(&command) else {
    return Ok(BackendDispatch::Unsupported(unsupported_message(&command)));
  };
  send_command(processes, agent_id, rpc_command).await?;
  Ok(BackendDispatch::Sent)
}

pub async fn send_command(processes: &Arc<Mutex<HashMap<String, PiProcessHandle>>>, agent_id: &str, command: serde_json::Value) -> Result<(), String> {
  let processes = processes.lock().await;
  let handle = processes.get(agent_id).ok_or_else(|| format!("No Pi process for agent {agent_id}"))?;
  handle.command_tx.send(command).map_err(|_| format!("Pi command channel is closed for {agent_id}"))
}

fn command_to_pi_rpc(command: &DesktopRuntimeCommand) -> Option<serde_json::Value> {
  Some(match command {
    DesktopRuntimeCommand::Prompt { message } => serde_json::json!({ "type": "prompt", "message": message }),
    DesktopRuntimeCommand::Steer { message } => serde_json::json!({ "type": "steer", "message": message }),
    DesktopRuntimeCommand::FollowUp { message } => serde_json::json!({ "type": "follow_up", "message": message }),
    DesktopRuntimeCommand::Abort => serde_json::json!({ "type": "abort" }),
    DesktopRuntimeCommand::RefreshState => serde_json::json!({ "type": "get_state" }),
    DesktopRuntimeCommand::Pause | DesktopRuntimeCommand::Resume => return None,
  })
}

fn unsupported_message(command: &DesktopRuntimeCommand) -> &'static str {
  match command {
    DesktopRuntimeCommand::Pause => "Pause is not supported by the Pi backend",
    DesktopRuntimeCommand::Resume => "Resume is not supported by the Pi backend",
    _ => "This command is not supported by the Pi backend",
  }
}

async fn read_stdout(stdout: ChildStdout, registry: DesktopRuntimeRegistry, agent_id: String) {
  let mut lines = BufReader::new(stdout).lines();
  while let Ok(Some(line)) = lines.next_line().await {
    let trimmed = line.trim();
    if trimmed.is_empty() {
      continue;
    }
    match serde_json::from_str::<serde_json::Value>(trimmed) {
      Ok(value) => handle_stdout_value(&registry, &agent_id, &value),
      Err(error) => registry.append_event(DesktopRuntimeEvent::new(
        agent_id.clone(),
        DesktopRuntimeEventKind::BackendError,
        text_payload("system", format!("Failed to parse pi stdout JSON: {error}")),
      )),
    }
  }
}

async fn read_stderr(stderr: tokio::process::ChildStderr, registry: DesktopRuntimeRegistry, agent_id: String) {
  let mut lines = BufReader::new(stderr).lines();
  while let Ok(Some(line)) = lines.next_line().await {
    let trimmed = line.trim();
    if trimmed.is_empty() {
      continue;
    }
    registry.append_event(DesktopRuntimeEvent::new(agent_id.clone(), DesktopRuntimeEventKind::BackendError, text_payload("system", trimmed)));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pause_and_resume_are_marked_unsupported_for_pi() {
    assert!(command_to_pi_rpc(&DesktopRuntimeCommand::Pause).is_none());
    assert!(command_to_pi_rpc(&DesktopRuntimeCommand::Resume).is_none());
    assert_eq!(unsupported_message(&DesktopRuntimeCommand::Pause), "Pause is not supported by the Pi backend");
  }
}
