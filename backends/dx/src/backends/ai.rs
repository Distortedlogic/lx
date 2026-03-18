use std::sync::Arc;
use std::time::Instant;

use lx::backends::{AiBackend, AiOpts};
use lx::error::LxError;
use lx::span::Span;
use lx::value::Value;

use crate::event::{EventBus, RuntimeEvent, next_call_id};
use crate::langfuse::LangfuseClient;

pub struct DxAiBackend {
    pub inner: Box<dyn AiBackend>,
    pub bus: Arc<EventBus>,
    pub langfuse: Arc<LangfuseClient>,
    pub agent_id: String,
}

impl AiBackend for DxAiBackend {
    fn prompt(&self, text: &str, opts: &AiOpts, span: Span) -> Result<Value, LxError> {
        let call_id = next_call_id();
        let model_name = opts.model.clone().unwrap_or_else(|| "claude".to_string());

        self.bus.send(RuntimeEvent::AiCallStart {
            agent_id: self.agent_id.clone(),
            call_id,
            prompt: text.to_string(),
            model: Some(model_name.clone()),
            system: opts.system.clone(),
            ts: Instant::now(),
        });

        let trace = self.langfuse.create_trace(
            &format!("ai.prompt:{}", self.agent_id),
            serde_json::json!({}),
        );
        let generation = self
            .langfuse
            .create_generation(&trace, "ai.prompt", &model_name, text);

        let start = Instant::now();
        let result = self.inner.prompt(text, opts, span);
        let elapsed = start.elapsed();
        let duration_ms = elapsed.as_millis() as u64;

        match &result {
            Ok(val) => {
                let response_text = format!("{val}");
                let cost = val.float_field("cost");
                let actual_model = val.str_field("model").unwrap_or(&model_name);

                generation.end_success(&response_text, duration_ms, actual_model);

                self.bus.send(RuntimeEvent::AiCallComplete {
                    agent_id: self.agent_id.clone(),
                    call_id,
                    response: response_text,
                    cost_usd: cost,
                    duration_ms,
                    model: actual_model.to_string(),
                    langfuse_trace_id: Some(trace.id.clone()),
                    ts: Instant::now(),
                });
            }
            Err(e) => {
                let error_text = format!("{e}");
                generation.end_error(&error_text);

                self.bus.send(RuntimeEvent::AiCallError {
                    agent_id: self.agent_id.clone(),
                    call_id,
                    error: error_text,
                    ts: Instant::now(),
                });
            }
        }

        result
    }
}
