mod defaults;
mod restricted;
mod user;

pub use defaults::*;
pub use restricted::*;
pub use user::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexMap;
use smart_default::SmartDefault;

use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

#[derive(SmartDefault)]
pub struct RuntimeCtx {
  #[default(Arc::new(StdoutEmitBackend))]
  pub emit: Arc<dyn EmitBackend>,
  #[default(Arc::new(ReqwestHttpBackend))]
  pub http: Arc<dyn HttpBackend>,
  #[default(Arc::new(StdinStdoutYieldBackend))]
  pub yield_: Arc<dyn YieldBackend>,
  #[default(Arc::new(StderrLogBackend))]
  pub log: Arc<dyn LogBackend>,
  #[default(Arc::new(NoopUserBackend))]
  pub user: Arc<dyn UserBackend>,
  pub source_dir: parking_lot::Mutex<Option<PathBuf>>,
  pub workspace_members: HashMap<String, PathBuf>,
  pub dep_dirs: HashMap<String, PathBuf>,
  #[default(Arc::new(tokio::runtime::Runtime::new().expect("failed to create tokio runtime")))]
  pub tokio_runtime: Arc<tokio::runtime::Runtime>,
  pub test_threshold: Option<f64>,
  pub test_runs: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct HttpOpts {
  pub headers: Option<IndexMap<String, String>>,
  pub query: Option<IndexMap<String, String>>,
  pub body: Option<serde_json::Value>,
}

pub trait EmitBackend: Send + Sync {
  fn emit(&self, value: &LxVal, span: SourceSpan) -> Result<(), LxError>;
}

pub trait HttpBackend: Send + Sync {
  fn request(&self, method: &str, url: &str, opts: &HttpOpts, span: SourceSpan) -> Result<LxVal, LxError>;
}

pub trait YieldBackend: Send + Sync {
  fn yield_value(&self, value: LxVal, span: SourceSpan) -> Result<LxVal, LxError>;
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
  fn check_signal(&self) -> Option<LxVal>;
}
