use std::sync::Arc;
use std::time::Instant;

use lx::backends::{LogBackend, LogLevel};

use crate::event::{EventBus, RuntimeEvent};
use crate::langfuse::LangfuseClient;

pub struct LangfuseLogBackend {
    pub bus: Arc<EventBus>,
    pub langfuse: Arc<LangfuseClient>,
    pub agent_id: String,
}

impl LogBackend for LangfuseLogBackend {
    fn log(&self, level: LogLevel, msg: &str) {
        let level_str = match level {
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Err => "err",
            LogLevel::Debug => "debug",
        };

        self.bus.send(RuntimeEvent::Log {
            agent_id: self.agent_id.clone(),
            level: level_str.to_string(),
            msg: msg.to_string(),
            ts: Instant::now(),
        });

        self.langfuse.log_event(&self.agent_id, level_str, msg);
    }
}
