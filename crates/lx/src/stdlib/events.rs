use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use indexmap::IndexMap;
use num_traits::ToPrimitive;

use crate::error::LxError;
use crate::event_stream::entry_to_lxval;
use crate::runtime::RuntimeCtx;
use crate::sym::{Sym, intern};
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<Sym, LxVal> {
  let mut m = IndexMap::new();
  m.insert(intern("xadd"), crate::builtins::mk("events.xadd", 1, bi_xadd));
  m.insert(intern("xrange"), crate::builtins::mk("events.xrange", 2, bi_xrange));
  m.insert(intern("xread"), crate::builtins::mk_async("events.xread", 2, bi_xread));
  m.insert(intern("xlen"), crate::builtins::mk("events.xlen", 1, bi_xlen));
  m.insert(intern("xtrim"), crate::builtins::mk("events.xtrim", 1, bi_xtrim));
  m
}

fn bi_xadd(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let rec = args[0].require_record("events.xadd", span)?;
  let kind = rec.get(&intern("kind")).and_then(|v| v.as_str()).ok_or_else(|| LxError::type_err("events.xadd: 'kind' field required as Str", span, None))?;
  let agent = rec.get(&intern("agent")).and_then(|v| v.as_str()).unwrap_or("main");
  let mut fields = IndexMap::new();
  for (k, v) in rec.iter() {
    let name = k.as_str();
    if name != "kind" && name != "agent" {
      fields.insert(*k, v.clone());
    }
  }
  let id = ctx.event_stream.xadd(kind, agent, None, fields);
  Ok(LxVal::str(id))
}

fn bi_xrange(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let start = args[0].require_str("events.xrange", span)?;
  let end = args[1].require_str("events.xrange", span)?;
  let entries = ctx.event_stream.xrange(start, end, None);
  let items: Vec<LxVal> = entries.iter().map(entry_to_lxval).collect();
  Ok(LxVal::list(items))
}

fn bi_xread(args: Vec<LxVal>, span: SourceSpan, ctx: Arc<RuntimeCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> {
  Box::pin(async move {
    let last_id = match &args[0] {
      LxVal::Str(s) => s.to_string(),
      other => {
        return Err(LxError::type_err(format!("events.xread: expected Str for last_id, got {}", other.type_name()), span, None));
      },
    };
    let timeout_ms = args[1].int_field("timeout_ms").and_then(|n| n.to_u64());
    match ctx.event_stream.xread(&last_id, timeout_ms).await {
      Some(entry) => Ok(entry_to_lxval(&entry)),
      None => Ok(LxVal::None),
    }
  })
}

fn bi_xlen(_args: &[LxVal], _span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::int(ctx.event_stream.xlen() as i64))
}

fn bi_xtrim(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let rec = args[0].require_record("events.xtrim", span)?;
  let maxlen = rec
    .get(&intern("maxlen"))
    .and_then(|v| v.as_int())
    .and_then(|n| n.to_usize())
    .ok_or_else(|| LxError::type_err("events.xtrim: 'maxlen' field required as Int", span, None))?;
  ctx.event_stream.xtrim(maxlen);
  Ok(LxVal::Unit)
}
