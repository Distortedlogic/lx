mod defaults;
mod embed;
mod pane;
mod restricted;
mod user;

pub use defaults::*;
pub use embed::*;
pub use pane::*;
pub use restricted::*;
pub use user::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub enum AgentEvent {
    Spawned { id: String, name: String },
    Killed { id: String },
}

pub struct RuntimeCtx {
    pub ai: Arc<dyn AiBackend>,
    pub emit: Arc<dyn EmitBackend>,
    pub http: Arc<dyn HttpBackend>,
    pub shell: Arc<dyn ShellBackend>,
    pub yield_: Arc<dyn YieldBackend>,
    pub log: Arc<dyn LogBackend>,
    pub user: Arc<dyn UserBackend>,
    pub pane: Arc<dyn PaneBackend>,
    pub embed: Arc<dyn EmbedBackend>,
    pub on_agent_event: Option<Arc<dyn Fn(AgentEvent) + Send + Sync>>,
    pub source_dir: parking_lot::Mutex<Option<PathBuf>>,
    pub workspace_members: HashMap<String, PathBuf>,
    pub dep_dirs: HashMap<String, PathBuf>,
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,
    pub test_threshold: Option<f64>,
    pub test_runs: Option<u32>,
}

impl Default for RuntimeCtx {
    fn default() -> Self {
        Self {
            ai: Arc::new(ClaudeCodeAiBackend),
            emit: Arc::new(StdoutEmitBackend),
            http: Arc::new(ReqwestHttpBackend),
            shell: Arc::new(ProcessShellBackend),
            yield_: Arc::new(StdinStdoutYieldBackend),
            log: Arc::new(StderrLogBackend),
            user: Arc::new(NoopUserBackend),
            pane: Arc::new(YieldPaneBackend),
            embed: Arc::new(VoyageEmbedBackend),
            on_agent_event: None,
            source_dir: parking_lot::Mutex::new(None),
            workspace_members: HashMap::new(),
            dep_dirs: HashMap::new(),
            tokio_runtime: Arc::new(
                tokio::runtime::Runtime::new().expect("failed to create tokio runtime"),
            ),
            test_threshold: None,
            test_runs: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AiOpts {
    pub system: Option<String>,
    pub model: Option<String>,
    pub max_turns: Option<i64>,
    pub resume: Option<String>,
    pub tools: Option<Vec<String>>,
    pub append_system: Option<String>,
    pub disable_tools: bool,
    pub json_schema: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HttpOpts {
    pub headers: Option<IndexMap<String, String>>,
    pub query: Option<IndexMap<String, String>>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct EmbedOpts {
    pub model: Option<String>,
    pub dimensions: Option<usize>,
}

pub trait AiBackend: Send + Sync {
    fn prompt(&self, text: &str, opts: &AiOpts, span: Span) -> Result<Value, LxError>;
}

pub trait EmitBackend: Send + Sync {
    fn emit(&self, value: &Value, span: Span) -> Result<(), LxError>;
}

pub trait HttpBackend: Send + Sync {
    fn request(
        &self,
        method: &str,
        url: &str,
        opts: &HttpOpts,
        span: Span,
    ) -> Result<Value, LxError>;
}

pub trait ShellBackend: Send + Sync {
    fn exec(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
    fn exec_capture(&self, cmd: &str, span: Span) -> Result<Value, LxError>;
}

pub trait YieldBackend: Send + Sync {
    fn yield_value(&self, value: Value, span: Span) -> Result<Value, LxError>;
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Err,
    Debug,
}

pub trait LogBackend: Send + Sync {
    fn log(&self, level: LogLevel, msg: &str);
}

pub trait UserBackend: Send + Sync {
    fn confirm(&self, message: &str) -> Result<bool, String>;
    fn choose(&self, message: &str, options: &[String]) -> Result<usize, String>;
    fn ask(&self, message: &str, default: Option<&str>) -> Result<String, String>;
    fn progress(&self, current: usize, total: usize, message: &str);
    fn progress_pct(&self, pct: f64, message: &str);
    fn status(&self, level: &str, message: &str);
    fn table(&self, headers: &[String], rows: &[Vec<String>]);
    fn check_signal(&self) -> Option<Value>;
}

pub trait PaneBackend: Send + Sync {
    fn open(&self, kind: &str, config: &Value, span: Span) -> Result<Value, LxError>;
    fn update(&self, pane_id: &str, content: &Value, span: Span) -> Result<(), LxError>;
    fn close(&self, pane_id: &str, span: Span) -> Result<(), LxError>;
    fn list(&self, span: Span) -> Result<Value, LxError>;
}

pub trait EmbedBackend: Send + Sync {
    fn embed(&self, texts: &[String], opts: &EmbedOpts, span: Span) -> Result<Value, LxError>;
}
