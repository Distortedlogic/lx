use std::future::Future;
use std::pin::Pin;

use crate::LxVal;
use crate::error::LxError;
use crate::event_stream::EventStream;

pub trait ToolModuleHandle: std::fmt::Debug + Send + Sync {
  fn call_tool<'a>(
    &'a self,
    method: &'a str,
    args: LxVal,
    event_stream: &'a EventStream,
    agent_name: &'a str,
  ) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>> + 'a>>;

  fn shutdown(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>;

  fn command(&self) -> &str;
  fn alias(&self) -> &str;
}
