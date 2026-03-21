use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::LxVal;

pub fn build() -> IndexMap<String, LxVal> {
    let mut m = IndexMap::new();
    m.insert("system".into(), mk("introspect.system", 1, bi_system));
    m.insert("agents".into(), mk("introspect.agents", 1, bi_agents));
    m.insert("agent".into(), mk("introspect.agent", 1, bi_agent));
    m.insert("messages".into(), mk("introspect.messages", 1, bi_messages));
    m.insert(
        "bottleneck".into(),
        mk("introspect.bottleneck", 1, bi_bottleneck),
    );
    m
}

fn bi_system(_args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Ok(Box::new(record! {
        "agents" => LxVal::List(Arc::new(Vec::new())),
        "messages_in_flight" => LxVal::Int(BigInt::from(0)),
        "topics" => LxVal::List(Arc::new(Vec::new())),
        "supervisors" => LxVal::List(Arc::new(Vec::new())),
    })))
}

fn bi_agents(_args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Ok(Box::new(LxVal::List(Arc::new(Vec::new())))))
}

fn bi_agent(_args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Err(Box::new(LxVal::Str(Arc::from(
        "agent introspection unavailable (agent runtime removed)",
    )))))
}

fn bi_messages(_args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Ok(Box::new(LxVal::List(Arc::new(Vec::new())))))
}

fn bi_bottleneck(_args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::None)
}
