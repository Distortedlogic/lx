use std::sync::Arc;
use std::time::Instant;

use lx::backends::YieldBackend;
use lx::error::LxError;
use lx::span::Span;
use lx::stdlib::json_conv::{json_to_lx, lx_to_json};
use lx::value::Value;
use tokio::sync::oneshot;

use crate::event::{EventBus, RuntimeEvent, next_prompt_id};

pub struct DxYieldBackend {
    bus: Arc<EventBus>,
    agent_id: String,
    response_tx: Arc<std::sync::Mutex<Option<oneshot::Sender<serde_json::Value>>>>,
}

impl DxYieldBackend {
    pub fn new(bus: Arc<EventBus>, agent_id: String) -> Self {
        Self {
            bus,
            agent_id,
            response_tx: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn response_sender(
        &self,
    ) -> Arc<std::sync::Mutex<Option<oneshot::Sender<serde_json::Value>>>> {
        self.response_tx.clone()
    }
}

impl YieldBackend for DxYieldBackend {
    fn yield_value(&self, value: Value, span: Span) -> Result<Value, LxError> {
        let json_val = lx_to_json(&value, span)
            .map_err(|e| LxError::runtime(format!("yield: {e}"), span))?;

        let prompt_id = next_prompt_id();

        self.bus.send(RuntimeEvent::Emit {
            agent_id: self.agent_id.clone(),
            value: format!("[yield] {json_val}"),
            ts: Instant::now(),
        });

        let (tx, rx) = oneshot::channel();
        {
            let mut guard = self
                .response_tx
                .lock()
                .map_err(|e| LxError::runtime(format!("yield lock: {e}"), span))?;
            *guard = Some(tx);
        }

        self.bus.send(RuntimeEvent::UserPrompt {
            agent_id: self.agent_id.clone(),
            prompt_id,
            kind: crate::event::UserPromptKind::Ask {
                message: format!("yield: {json_val}"),
                default: None,
            },
            ts: Instant::now(),
        });

        let response = rx
            .blocking_recv()
            .map_err(|_| LxError::runtime("yield: orchestrator cancelled", span))?;

        self.bus.send(RuntimeEvent::UserResponse {
            agent_id: self.agent_id.clone(),
            prompt_id,
            response: response.clone(),
            ts: Instant::now(),
        });

        Ok(json_to_lx(response))
    }
}
