use std::sync::Arc;

use crate::BuiltinCtx;
use crate::error::LxError;
use crate::interpreter::ambient::{global_context_current, global_context_get};
use crate::value::LxVal;
use miette::SourceSpan;

pub(super) fn bi_source_dir(_args: &[LxVal], _span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let dir = ctx.source_dir();
  match dir {
    Some(p) => Ok(LxVal::str(p.to_string_lossy())),
    None => Ok(LxVal::str(".")),
  }
}

pub(super) fn bi_agent_implements(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let target_trait = match &args[1] {
    LxVal::Str(s) => s.to_string(),
    LxVal::Trait(t) => t.name.as_str().to_string(),
    LxVal::Class(c) => c.name.as_str().to_string(),
    _ => String::new(),
  };
  let has = match &args[0] {
    LxVal::Object(o) => o.traits.iter().any(|t| t.as_str() == target_trait.as_str()),
    LxVal::Class(c) => c.traits.iter().any(|t| t.as_str() == target_trait.as_str()),
    LxVal::Record(r) => r
      .get(&crate::sym::intern("traits"))
      .or_else(|| r.get(&crate::sym::intern("__traits")))
      .and_then(|v| v.as_list())
      .map(|l| l.iter().any(|v| v.as_str() == Some(target_trait.as_str())))
      .unwrap_or(false),
    _ => false,
  };
  Ok(LxVal::Bool(has))
}

pub(super) fn bi_json_parse(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let s = args[0].require_str("json.parse", span)?;
  match serde_json::from_str::<serde_json::Value>(s) {
    Ok(jv) => Ok(LxVal::ok(LxVal::from(jv))),
    Err(e) => Ok(LxVal::err_str(e.to_string())),
  }
}

pub(super) fn bi_json_encode(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let s = serde_json::to_string(&args[0]).map_err(|e| LxError::runtime(e.to_string(), span))?;
  Ok(LxVal::str(s))
}

pub(super) fn bi_json_encode_pretty(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let s = serde_json::to_string_pretty(&args[0]).map_err(|e| LxError::runtime(e.to_string(), span))?;
  Ok(LxVal::str(s))
}

pub(super) fn bi_method_of(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let LxVal::Str(s) = &args[1] else {
    return Ok(LxVal::None);
  };
  let name = s.as_ref();
  match &args[0] {
    LxVal::Object(o) => match o.methods.get(&crate::sym::intern(name)) {
      Some(method) => Ok(method.bind_self(&args[0])),
      None => Ok(LxVal::None),
    },
    LxVal::Class(c) => match c.methods.get(&crate::sym::intern(name)) {
      Some(method) => Ok(method.bind_self(&args[0])),
      None => Ok(LxVal::None),
    },
    _ => Ok(LxVal::None),
  }
}

pub(super) fn bi_global_context_current(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  global_context_current()
}

pub(super) fn bi_global_context_get(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  global_context_get(&args[0], span)
}

pub(super) fn bi_resolve_handler(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let agent = &args[0];
  if let LxVal::Record(r) = agent
    && let Some(h) = r.get(&crate::sym::intern("handler"))
  {
    return Ok(h.clone());
  }
  Ok(LxVal::None)
}

pub(super) fn bi_try(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let f = &args[0];
  let arg = args[1].clone();
  match crate::builtins::call_value_sync(f, arg, span, ctx) {
    Ok(v) => Ok(v),
    Err(LxError::Propagate { value, .. }) => Ok(LxVal::Err(value)),
    Err(e) => Err(e),
  }
}

pub(super) fn bi_methods_of(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let names = match &args[0] {
    LxVal::Object(o) => o.methods.keys().map(LxVal::str).collect(),
    LxVal::Class(c) => c.methods.keys().map(LxVal::str).collect(),
    _ => vec![],
  };
  Ok(LxVal::list(names))
}
