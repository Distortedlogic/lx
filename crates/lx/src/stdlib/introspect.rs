use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::std_module;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<crate::sym::Sym, LxVal> {
  std_module! {
    "system"     => "introspect.system",     1, bi_system;
    "agents"     => "introspect.agents",     1, bi_agents;
    "agent"      => "introspect.agent",      1, bi_agent;
    "messages"   => "introspect.messages",   1, bi_messages;
    "bottleneck" => "introspect.bottleneck", 1, bi_bottleneck
  }
}

fn bi_system(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::ok(record! {
      "agents" => LxVal::list(Vec::new()),
      "messages_in_flight" => LxVal::int(0),
      "topics" => LxVal::list(Vec::new()),
      "supervisors" => LxVal::list(Vec::new()),
  }))
}

fn bi_agents(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::ok(LxVal::list(Vec::new())))
}

fn bi_agent(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::err_str("agent introspection unavailable (agent runtime removed)"))
}

fn bi_messages(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::ok(LxVal::list(Vec::new())))
}

fn bi_bottleneck(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::None)
}
