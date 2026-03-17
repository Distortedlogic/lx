use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use lx::error::LxError;
use lx::interpreter::Interpreter;
use lx::lexer;
use lx::parser;
use lx::value::Value;

use crate::backends::build_runtime_ctx;
use crate::event::{EventBus, RuntimeEvent};
use crate::langfuse::LangfuseClient;

pub struct ProgramRunner {
    pub bus: Arc<EventBus>,
    pub langfuse: Arc<LangfuseClient>,
}

impl ProgramRunner {
    pub fn new(bus: Arc<EventBus>, langfuse: Arc<LangfuseClient>) -> Self {
        Self { bus, langfuse }
    }

    pub async fn run(&self, source_path: &str) -> Result<Value, RunError> {
        let source = tokio::fs::read_to_string(source_path)
            .await
            .map_err(|e| RunError::Io(format!("read {source_path}: {e}")))?;

        let tokens = lexer::lex(&source).map_err(RunError::Lx)?;
        let program = parser::parse(tokens).map_err(RunError::Lx)?;

        let trace = self.langfuse.create_trace(
            source_path,
            serde_json::json!({ "source_path": source_path }),
        );

        let agent_id = "main".to_string();
        let ctx = build_runtime_ctx(
            self.bus.clone(),
            self.langfuse.clone(),
            agent_id,
        );

        self.bus.send(RuntimeEvent::ProgramStarted {
            source_path: source_path.to_string(),
            ts: Instant::now(),
        });

        let start = Instant::now();
        let source_dir = Path::new(source_path)
            .parent()
            .map(|p| p.to_path_buf());

        let source_clone = source.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut interp = Interpreter::new(&source_clone, source_dir, ctx);
            interp.exec(&program)
        })
        .await
        .map_err(|e| RunError::Io(format!("task join: {e}")))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        self.bus.send(RuntimeEvent::ProgramFinished {
            result: result
                .as_ref()
                .map(|v| format!("{v}"))
                .map_err(|e| format!("{e}")),
            duration_ms,
            ts: Instant::now(),
        });

        trace.end(result.is_ok());

        result.map_err(RunError::Lx)
    }
}

pub enum RunError {
    Io(String),
    Lx(LxError),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "IO error: {msg}"),
            Self::Lx(e) => write!(f, "{e}"),
        }
    }
}
