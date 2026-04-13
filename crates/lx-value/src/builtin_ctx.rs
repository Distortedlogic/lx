use std::path::PathBuf;

use crate::EventStream;

pub trait BuiltinCtx: Send + Sync {
  fn event_stream(&self) -> &EventStream;
  fn source_dir(&self) -> Option<PathBuf>;
  fn network_denied(&self) -> bool;
  fn test_threshold(&self) -> Option<f64>;
  fn test_runs(&self) -> Option<u32>;
  fn as_any(&self) -> &dyn std::any::Any;
}
