use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;

use crate::event::EventBus;

pub struct HttpReporter {
  bus: Arc<EventBus>,
  client: Client,
  base_url: String,
}

impl HttpReporter {
  pub fn new(bus: Arc<EventBus>, base_url: String) -> Self {
    Self { bus, client: Client::new(), base_url }
  }

  pub fn start(self) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
      let mut rx = self.bus.subscribe();
      let mut buffer = Vec::new();
      let flush_interval = Duration::from_millis(500);
      let mut ticker = tokio::time::interval(flush_interval);
      ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

      loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let type_name = event_type_name(&event);
                        let agent = event.agent_id().unwrap_or("system").to_string();
                        buffer.push(serde_json::json!({
                            "type": type_name,
                            "agent_id": agent,
                        }));
                        if buffer.len() >= 100 {
                            self.flush(&mut buffer).await;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        if !buffer.is_empty() {
                            self.flush(&mut buffer).await;
                        }
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("http reporter: lagged {n} events");
                    }
                }
            }
            _ = ticker.tick() => {
                if !buffer.is_empty() {
                    self.flush(&mut buffer).await;
                }
            }
        }
      }
    })
  }

  async fn flush(&self, buffer: &mut Vec<serde_json::Value>) {
    let events: Vec<serde_json::Value> = std::mem::take(buffer);
    let url = format!("{}/api/events", self.base_url);
    let body = serde_json::json!({ "events": events });
    if let Err(e) = self.client.post(&url).json(&body).send().await {
      eprintln!("http reporter: POST failed: {e}");
    }
  }
}

fn event_type_name(event: &crate::event::RuntimeEvent) -> &'static str {
  use crate::event::RuntimeEvent;
  match event {
    RuntimeEvent::AgentSpawned { .. } => "agent_spawned",
    RuntimeEvent::AgentKilled { .. } => "agent_killed",
    RuntimeEvent::AiCallStart { .. } => "ai_call_start",
    RuntimeEvent::AiCallComplete { .. } => "ai_call_complete",
    RuntimeEvent::AiCallError { .. } => "ai_call_error",
    RuntimeEvent::MessageSend { .. } => "message_send",
    RuntimeEvent::MessageAsk { .. } => "message_ask",
    RuntimeEvent::MessageResponse { .. } => "message_response",
    RuntimeEvent::Emit { .. } => "emit",
    RuntimeEvent::Log { .. } => "log",
    RuntimeEvent::ShellExec { .. } => "shell_exec",
    RuntimeEvent::ShellResult { .. } => "shell_result",
    RuntimeEvent::UserPrompt { .. } => "user_prompt",
    RuntimeEvent::UserResponse { .. } => "user_response",
    RuntimeEvent::TraceSpanRecorded { .. } => "trace_span",
    RuntimeEvent::Progress { .. } => "progress",
    RuntimeEvent::Error { .. } => "error",
    RuntimeEvent::ProgramStarted { .. } => "program_started",
    RuntimeEvent::ProgramFinished { .. } => "program_finished",
  }
}
