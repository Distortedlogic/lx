use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("system".into(), mk("introspect.system", 1, bi_system));
  m.insert("agents".into(), mk("introspect.agents", 1, bi_agents));
  m.insert("agent".into(), mk("introspect.agent", 1, bi_agent));
  m.insert("messages".into(), mk("introspect.messages", 1, bi_messages));
  m.insert("bottleneck".into(), mk("introspect.bottleneck", 1, bi_bottleneck));
  m
}

fn bi_system(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Ok(Box::new(record! {
      "agents" => LxVal::list(Vec::new()),
      "messages_in_flight" => LxVal::int(0),
      "topics" => LxVal::list(Vec::new()),
      "supervisors" => LxVal::list(Vec::new()),
  })))
}

fn bi_agents(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Ok(Box::new(LxVal::list(Vec::new()))))
}

fn bi_agent(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Err(Box::new(LxVal::str("agent introspection unavailable (agent runtime removed)"))))
}

fn bi_messages(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Ok(Box::new(LxVal::list(Vec::new()))))
}

fn bi_bottleneck(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::None)
}
