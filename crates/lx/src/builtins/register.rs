use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::env::Env;
use crate::error::LxError;
use crate::runtime::{LogLevel, RuntimeCtx};
use crate::value::LxVal;
use miette::SourceSpan;

use super::mk;

fn make_log_builtin(name: &'static str, level: LogLevel) -> LxVal {
  fn log_fn(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>, level: LogLevel, name: &str) -> Result<LxVal, LxError> {
    let s = args[0].require_str(&format!("log.{name}"), span)?;
    ctx.log.log(level, s);
    Ok(LxVal::Unit)
  }
  match level {
    LogLevel::Info => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Info, "info")),
    LogLevel::Warn => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Warn, "warn")),
    LogLevel::Err => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Err, "err")),
    LogLevel::Debug => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Debug, "debug")),
  }
}

fn bi_not(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Bool(b) => Ok(LxVal::Bool(!b)),
    other => Err(LxError::type_err(format!("not expects Bool, got {}", other.type_name()), span, None)),
  }
}

fn bi_len(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let n = match &args[0] {
    LxVal::Str(s) => s.chars().count(),
    LxVal::List(l) => l.len(),
    LxVal::Record(r) => r.len(),
    LxVal::Map(m) => m.len(),
    LxVal::Tuple(t) => t.len(),
    LxVal::Store { id } => crate::stdlib::store_len(*id),
    other => {
      return Err(LxError::type_err(format!("len expects collection, got {}", other.type_name()), span, None));
    },
  };
  Ok(LxVal::int(n))
}

fn bi_empty(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let empty = match &args[0] {
    LxVal::Str(s) => s.is_empty(),
    LxVal::List(l) => l.is_empty(),
    LxVal::Record(r) => r.is_empty(),
    LxVal::Map(m) => m.is_empty(),
    LxVal::Tuple(t) => t.is_empty(),
    LxVal::Store { id } => crate::stdlib::store_len(*id) == 0,
    other => {
      return Err(LxError::type_err(format!("empty? expects collection, got {}", other.type_name()), span, None));
    },
  };
  Ok(LxVal::Bool(empty))
}

fn bi_to_str(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::str(args[0].to_string()))
}

fn bi_identity(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(args[0].clone())
}

fn bi_dbg(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  eprintln!("[dbg] {}", args[0]);
  Ok(args[0].clone())
}

fn bi_ok_q(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Bool(matches!(&args[0], LxVal::Ok(_))))
}

fn bi_err_q(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Bool(matches!(&args[0], LxVal::Err(_))))
}

fn bi_some_q(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Bool(matches!(&args[0], LxVal::Some(_))))
}

fn bi_even(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Int(n) => Ok(LxVal::Bool(n % BigInt::from(2) == BigInt::from(0))),
    other => Err(LxError::type_err(format!("even? expects Int, got {}", other.type_name()), span, None)),
  }
}

fn bi_odd(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Int(n) => Ok(LxVal::Bool(n % BigInt::from(2) != BigInt::from(0))),
    other => Err(LxError::type_err(format!("odd? expects Int, got {}", other.type_name()), span, None)),
  }
}

fn bi_type_of(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::str(args[0].type_name()))
}

fn bi_print(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  println!("{}", args[0]);
  Ok(LxVal::Unit)
}

pub fn register(env: &Env) {
  env.bind_str("true", LxVal::Bool(true));
  env.bind_str("false", LxVal::Bool(false));
  env.bind_str("None", LxVal::None);
  env.bind_str("Ok", mk("Ok", 1, |a, _, _ctx| Ok(LxVal::ok(a[0].clone()))));
  env.bind_str("Err", mk("Err", 1, |a, _, _ctx| Ok(LxVal::err(a[0].clone()))));
  env.bind_str("Some", mk("Some", 1, |a, _, _ctx| Ok(LxVal::some(a[0].clone()))));
  super::register_builtins!(env, {
    "not"/1 => bi_not, "len"/1 => bi_len, "empty?"/1 => bi_empty,
    "to_str"/1 => bi_to_str, "identity"/1 => bi_identity, "dbg"/1 => bi_dbg,
    "ok?"/1 => bi_ok_q, "err?"/1 => bi_err_q, "some?"/1 => bi_some_q,
    "even?"/1 => bi_even, "odd?"/1 => bi_odd, "type_of"/1 => bi_type_of,
    "print"/1 => bi_print,
  });
  super::convert::register(env);
  super::str::register(env);
  super::coll::register(env);
  let mut log_fields = IndexMap::new();
  log_fields.insert(crate::sym::intern("info"), make_log_builtin("log.info", LogLevel::Info));
  log_fields.insert(crate::sym::intern("warn"), make_log_builtin("log.warn", LogLevel::Warn));
  log_fields.insert(crate::sym::intern("err"), make_log_builtin("log.err", LogLevel::Err));
  log_fields.insert(crate::sym::intern("debug"), make_log_builtin("log.debug", LogLevel::Debug));
  env.bind_str("log", LxVal::record(log_fields));
  super::hof::register(env);
  env.bind_str("Store", crate::stdlib::build_constructor());
  super::register_builtins!(env, {
    "method_of"/2 => bi_method_of, "methods_of"/1 => bi_methods_of,
    "try"/2 => bi_try, "resolve_handler"/1 => bi_resolve_handler,
  });

  let mut json_fields = IndexMap::new();
  json_fields.insert(crate::sym::intern("parse"), mk("json.parse", 1, bi_json_parse));
  json_fields.insert(crate::sym::intern("encode"), mk("json.encode", 1, bi_json_encode));
  json_fields.insert(crate::sym::intern("encode_pretty"), mk("json.encode_pretty", 1, bi_json_encode_pretty));
  env.bind_str("json", LxVal::record(json_fields));

  let mut agent_fields = IndexMap::new();
  agent_fields.insert(crate::sym::intern("spawn"), mk("agent.spawn", 1, bi_agent_spawn_stub));
  agent_fields.insert(crate::sym::intern("kill"), mk("agent.kill", 1, bi_agent_kill_stub));
  agent_fields.insert(crate::sym::intern("implements"), mk("agent.implements", 2, bi_agent_implements));
  env.bind_str("agent", LxVal::record(agent_fields));

  let mut ctx_fields = IndexMap::new();
  ctx_fields.insert(crate::sym::intern("current"), mk("context.current", 1, bi_global_context_current));
  ctx_fields.insert(crate::sym::intern("get"), mk("context.get", 1, bi_global_context_get));
  env.bind_str("context", LxVal::record(ctx_fields));
}

fn bi_agent_spawn_stub(_args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Err(LxError::runtime("agent.spawn: subprocess agents not yet available in this build", span))
}

fn bi_agent_kill_stub(_args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Err(LxError::runtime("agent.kill: subprocess agents not yet available in this build", span))
}

fn bi_agent_implements(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
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

fn bi_json_parse(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let s = args[0].require_str("json.parse", span)?;
  match serde_json::from_str::<serde_json::Value>(s) {
    Ok(jv) => Ok(LxVal::ok(LxVal::from(jv))),
    Err(e) => Ok(LxVal::err_str(e.to_string())),
  }
}

fn bi_json_encode(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let s = serde_json::to_string(&args[0]).map_err(|e| LxError::runtime(e.to_string(), span))?;
  Ok(LxVal::str(s))
}

fn bi_json_encode_pretty(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let s = serde_json::to_string_pretty(&args[0]).map_err(|e| LxError::runtime(e.to_string(), span))?;
  Ok(LxVal::str(s))
}

fn bi_method_of(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let name = match &args[1] {
    LxVal::Str(s) => s.as_ref(),
    _ => return Ok(LxVal::None),
  };
  match &args[0] {
    LxVal::Object(o) => match o.methods.get(&crate::sym::intern(name)) {
      Some(method) => Ok(inject_self_for_method(method, &args[0])),
      None => Ok(LxVal::None),
    },
    LxVal::Class(c) => match c.methods.get(&crate::sym::intern(name)) {
      Some(method) => Ok(inject_self_for_method(method, &args[0])),
      None => Ok(LxVal::None),
    },
    _ => Ok(LxVal::None),
  }
}

fn inject_self_for_method(method: &LxVal, self_val: &LxVal) -> LxVal {
  if let LxVal::Func(lf) = method {
    let env = lf.closure.child();
    env.bind_str("self", self_val.clone());
    let mut lf = lf.clone();
    lf.closure = Arc::new(env);
    LxVal::Func(lf)
  } else {
    method.clone()
  }
}

fn bi_global_context_current(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  crate::interpreter::ambient::global_context_current()
}

fn bi_global_context_get(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  crate::interpreter::ambient::global_context_get(&args[0], span)
}

fn bi_resolve_handler(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let agent = &args[0];
  if let LxVal::Record(r) = agent
    && let Some(h) = r.get(&crate::sym::intern("handler"))
  {
    return Ok(h.clone());
  }
  Ok(LxVal::None)
}

fn bi_try(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let f = &args[0];
  let arg = args[1].clone();
  match crate::builtins::call_value_sync(f, arg, span, ctx) {
    Ok(v) => Ok(v),
    Err(LxError::Propagate { value, .. }) => Ok(LxVal::Err(value)),
    Err(e) => Err(e),
  }
}

fn bi_methods_of(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let names = match &args[0] {
    LxVal::Object(o) => o.methods.keys().map(LxVal::str).collect(),
    LxVal::Class(c) => c.methods.keys().map(LxVal::str).collect(),
    _ => vec![],
  };
  Ok(LxVal::list(names))
}
