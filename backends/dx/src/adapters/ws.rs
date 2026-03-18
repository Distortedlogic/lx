use std::sync::Arc;

use futures::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::event::{EventBus, RuntimeEvent};

pub struct WsStream {
    bus: Arc<EventBus>,
}

impl WsStream {
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }

    pub async fn stream_to<S, E>(&self, sink: &mut S, agent_filter: Option<String>)
    where
        S: futures::Sink<Message, Error = E> + Unpin,
        E: std::fmt::Display,
    {
        let mut rx = self.bus.subscribe();
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if let Some(ref filter) = agent_filter {
                        if event.agent_id().map_or(true, |id| id != filter) {
                            continue;
                        }
                    }
                    let json = serialize_event(&event);
                    let msg = Message::Text(json);
                    if let Err(e) = sink.send(msg).await {
                        eprintln!("ws stream error: {e}");
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("ws stream: lagged {n} events");
                }
            }
        }
    }
}

fn serialize_event(event: &RuntimeEvent) -> String {
    let type_name = match event {
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
    };
    let agent = event.agent_id().unwrap_or("system");
    serde_json::json!({
        "type": type_name,
        "agent_id": agent,
    })
    .to_string()
}
