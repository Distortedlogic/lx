use std::sync::Arc;
use std::time::Instant;

use lx::backends::EmitBackend;
use lx::error::LxError;
use lx::span::Span;
use lx::value::LxVal;

use crate::event::{EventBus, RuntimeEvent};

pub struct DxEmitBackend {
  pub bus: Arc<EventBus>,
  pub agent_id: String,
}

impl EmitBackend for DxEmitBackend {
  fn emit(&self, value: &LxVal, _span: Span) -> Result<(), LxError> {
    self.bus.send(RuntimeEvent::Emit { agent_id: self.agent_id.clone(), value: format!("{value}"), ts: Instant::now() });
    Ok(())
  }
}
