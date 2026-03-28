pub mod agent_registry;
pub mod channel_registry;
mod defaults;

pub use defaults::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use smart_default::SmartDefault;

use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

#[derive(SmartDefault)]
pub struct RuntimeCtx {
  #[default(Arc::new(StdinStdoutYieldBackend))]
  pub yield_: Arc<dyn YieldBackend>,
  pub source_dir: parking_lot::Mutex<Option<PathBuf>>,
  pub workspace_members: HashMap<String, PathBuf>,
  pub dep_dirs: HashMap<String, PathBuf>,
  #[default(Arc::new(tokio::runtime::Runtime::new().expect("failed to create tokio runtime")))]
  pub tokio_runtime: Arc<tokio::runtime::Runtime>,
  pub test_threshold: Option<f64>,
  pub test_runs: Option<u32>,
  #[default(Arc::new(crate::event_stream::EventStream::new(None)))]
  pub event_stream: Arc<crate::event_stream::EventStream>,
  #[default(false)]
  pub network_denied: bool,
}

pub trait YieldBackend: Send + Sync {
  fn yield_value(&self, value: LxVal, span: SourceSpan) -> Result<LxVal, LxError>;
}
