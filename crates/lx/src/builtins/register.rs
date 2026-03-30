use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::BuiltinCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

use super::mk;
use super::register_helpers::{
  bi_agent_implements, bi_global_context_current, bi_global_context_get, bi_json_encode, bi_json_encode_pretty, bi_json_parse, bi_method_of, bi_methods_of,
  bi_resolve_handler, bi_source_dir, bi_try,
};

fn log_at(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>, level: &str) -> Result<LxVal, LxError> {
  let s = args[0].require_str(&format!("log.{level}"), span)?;
  eprintln!("[{}] {}", level.to_uppercase(), s);
  let mut fields = indexmap::IndexMap::new();
  fields.insert(crate::sym::intern("level"), LxVal::str(level));
  fields.insert(crate::sym::intern("msg"), LxVal::str(s));
  ctx.event_stream().xadd("runtime/log", "main", None, fields);
  Ok(LxVal::Unit)
}

fn bi_log_info(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  log_at(args, span, ctx, "info")
}
fn bi_log_warn(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  log_at(args, span, ctx, "warn")
}
fn bi_log_err(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  log_at(args, span, ctx, "err")
}
fn bi_log_debug(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  log_at(args, span, ctx, "debug")
}

fn bi_not(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Bool(b) => Ok(LxVal::Bool(!b)),
    other => Err(LxError::type_err(format!("not expects Bool, got {}", other.type_name()), span, None)),
  }
}

fn bi_len(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
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

fn bi_empty(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
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

fn bi_to_str(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::str(args[0].to_string()))
}

fn bi_identity(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  Ok(args[0].clone())
}

fn bi_dbg(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  eprintln!("[dbg] {}", args[0]);
  Ok(args[0].clone())
}

fn bi_ok_q(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Bool(matches!(&args[0], LxVal::Ok(_))))
}

fn bi_err_q(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Bool(matches!(&args[0], LxVal::Err(_))))
}

fn bi_some_q(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  Ok(LxVal::Bool(matches!(&args[0], LxVal::Some(_))))
}

fn bi_even(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Int(n) => Ok(LxVal::Bool(n % BigInt::from(2) == BigInt::from(0))),
    other => Err(LxError::type_err(format!("even? expects Int, got {}", other.type_name()), span, None)),
  }
}

fn bi_odd(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Int(n) => Ok(LxVal::Bool(n % BigInt::from(2) != BigInt::from(0))),
    other => Err(LxError::type_err(format!("odd? expects Int, got {}", other.type_name()), span, None)),
  }
}

fn bi_type_of(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let name = match &args[0] {
    LxVal::BuiltinFunc(_) => "Func",
    LxVal::MultiFunc(_) => "Func",
    other => other.type_name(),
  };
  Ok(LxVal::typ(name))
}

fn bi_print(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  println!("{}", args[0]);
  Ok(LxVal::Unit)
}

pub fn register(env: &Env) {
  env.bind_str("true", LxVal::Bool(true));
  env.bind_str("false", LxVal::Bool(false));
  env.bind_str("None", LxVal::None);

  env.bind_str("Str", LxVal::typ("Str"));
  env.bind_str("Int", LxVal::typ("Int"));
  env.bind_str("Float", LxVal::typ("Float"));
  env.bind_str("Bool", LxVal::typ("Bool"));
  env.bind_str("List", LxVal::typ("List"));
  env.bind_str("Record", LxVal::typ("Record"));
  env.bind_str("Map", LxVal::typ("Map"));
  env.bind_str("Tuple", LxVal::typ("Tuple"));
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
  log_fields.insert(crate::sym::intern("info"), mk("log.info", 1, bi_log_info));
  log_fields.insert(crate::sym::intern("warn"), mk("log.warn", 1, bi_log_warn));
  log_fields.insert(crate::sym::intern("err"), mk("log.err", 1, bi_log_err));
  log_fields.insert(crate::sym::intern("debug"), mk("log.debug", 1, bi_log_debug));
  env.bind_str("log", LxVal::record(log_fields));
  super::hof::register(env);
  super::shell::register(env);
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
  agent_fields.insert(
    crate::sym::intern("spawn"),
    super::mk_async("agent.spawn", 1, |args, span, ctx| Box::pin(crate::builtins::agent::bi_agent_spawn(args, span, ctx))),
  );
  agent_fields.insert(
    crate::sym::intern("kill"),
    mk("agent.kill", 1, |args, span, _ctx| {
      let name = args[0].require_str("agent.kill", span)?;
      crate::runtime::channel_registry::channel_unsubscribe_all(name);
      match crate::runtime::agent_registry::remove_agent(name) {
        Some(_) => Ok(LxVal::ok_unit()),
        None => Ok(LxVal::err_str(format!("agent '{name}' not running"))),
      }
    }),
  );
  agent_fields.insert(
    crate::sym::intern("exists"),
    mk("agent.exists", 1, |args, span, _ctx| {
      let name = args[0].require_str("agent.exists", span)?;
      Ok(LxVal::Bool(crate::runtime::agent_registry::agent_exists(name)))
    }),
  );
  agent_fields.insert(
    crate::sym::intern("list"),
    mk("agent.list", 0, |_args, _span, _ctx| {
      let names = crate::runtime::agent_registry::agent_names();
      Ok(LxVal::list(names.into_iter().map(LxVal::str).collect()))
    }),
  );
  agent_fields.insert(crate::sym::intern("implements"), mk("agent.implements", 2, bi_agent_implements));
  env.bind_str("agent", LxVal::record(agent_fields));

  let mut llm_fields = IndexMap::new();
  llm_fields.insert(crate::sym::intern("prompt"), mk("llm.prompt", 1, super::llm::bi_prompt));
  llm_fields.insert(crate::sym::intern("prompt_with"), mk("llm.prompt_with", 1, super::llm::bi_prompt_with));
  llm_fields.insert(crate::sym::intern("prompt_structured"), mk("llm.prompt_structured", 2, super::llm::bi_prompt_structured));
  env.bind_str("llm", LxVal::record(llm_fields));

  let mut ctx_fields = IndexMap::new();
  ctx_fields.insert(crate::sym::intern("current"), mk("context.current", 1, bi_global_context_current));
  ctx_fields.insert(crate::sym::intern("get"), mk("context.get", 1, bi_global_context_get));
  env.bind_str("context", LxVal::record(ctx_fields));
  env.bind_str("source_dir", mk("source_dir", 0, bi_source_dir));
}
