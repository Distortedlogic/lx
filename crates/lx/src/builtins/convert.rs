use std::sync::Arc;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::env::Env;
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

fn bi_collect(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Range { start, end, inclusive } => {
      let items: Vec<LxVal> = if *inclusive { (*start..=*end).map(LxVal::int).collect() } else { (*start..*end).map(LxVal::int).collect() };
      Ok(LxVal::list(items))
    },
    other => Ok(other.clone()),
  }
}

fn bi_step(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let step = args[0].require_int("step", span)?;
  let step = step.to_i64().ok_or_else(|| LxError::runtime("step: value too large", span))?;
  if step <= 0 {
    return Err(LxError::runtime("step: must be positive", span));
  }
  match &args[1] {
    LxVal::Range { start, end, inclusive } => {
      let mut items = Vec::new();
      let mut i = *start;
      let limit = if *inclusive { *end + 1 } else { *end };
      while i < limit {
        items.push(LxVal::int(i));
        i += step;
      }
      Ok(LxVal::list(items))
    },
    LxVal::List(l) => {
      let items: Vec<LxVal> = l.iter().step_by(step as usize).cloned().collect();
      Ok(LxVal::list(items))
    },
    other => Err(LxError::type_err(format!("step: expects Range/List, got {}", other.type_name()), span, None)),
  }
}

fn bi_require(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[1] {
    LxVal::Some(v) => Ok(LxVal::ok(*v.clone())),
    LxVal::None => Ok(LxVal::err(args[0].clone())),
    other => Ok(LxVal::ok(other.clone())),
  }
}

fn bi_parse_int(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => match s.parse::<BigInt>() {
      Ok(n) => Ok(LxVal::ok(LxVal::Int(n))),
      Err(e) => Ok(LxVal::err_str(e.to_string())),
    },
    other => Err(LxError::type_err(format!("parse_int expects Str, got {}", other.type_name()), span, None)),
  }
}

fn bi_parse_float(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => match s.parse::<f64>() {
      Ok(f) => Ok(LxVal::ok(LxVal::Float(f))),
      Err(e) => Ok(LxVal::err_str(e.to_string())),
    },
    other => Err(LxError::type_err(format!("parse_float expects Str, got {}", other.type_name()), span, None)),
  }
}

fn bi_to_int(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Int(_) => Ok(args[0].clone()),
    LxVal::Float(f) => Ok(LxVal::int(*f as i64)),
    LxVal::Str(s) => s.parse::<BigInt>().map(LxVal::Int).map_err(|e| LxError::runtime(format!("to_int: {e}"), span)),
    LxVal::Bool(b) => Ok(LxVal::Int(if *b { 1.into() } else { 0.into() })),
    other => Err(LxError::type_err(format!("to_int: cannot convert {}", other.type_name()), span, None)),
  }
}

fn bi_to_float(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Float(_) => Ok(args[0].clone()),
    LxVal::Int(n) => n.to_f64().map(LxVal::Float).ok_or_else(|| LxError::runtime("to_float: int too large", span)),
    LxVal::Str(s) => s.parse::<f64>().map(LxVal::Float).map_err(|e| LxError::runtime(format!("to_float: {e}"), span)),
    other => Err(LxError::type_err(format!("to_float: cannot convert {}", other.type_name()), span, None)),
  }
}

fn bi_sleep(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let secs = match &args[0] {
    LxVal::Int(n) => n.to_f64().ok_or_else(|| LxError::runtime("sleep: value too large", span))?,
    LxVal::Float(f) => *f,
    other => {
      return Err(LxError::type_err(format!("sleep expects number, got {}", other.type_name()), span, None));
    },
  };
  std::thread::sleep(std::time::Duration::from_secs_f64(secs));
  Ok(LxVal::Unit)
}

pub(super) fn register(env: &Env) {
  super::register_builtins!(env, {
    "collect"/1 => bi_collect, "step"/2 => bi_step, "require"/2 => bi_require,
    "parse_int"/1 => bi_parse_int, "parse_float"/1 => bi_parse_float,
    "to_int"/1 => bi_to_int, "to_float"/1 => bi_to_float, "sleep"/1 => bi_sleep,
  });
}
