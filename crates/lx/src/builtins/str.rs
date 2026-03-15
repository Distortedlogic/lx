use std::sync::Arc;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::mk;

fn bi_trim(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  match &args[0] {
    Value::Str(s) => Ok(Value::Str(Arc::from(s.trim()))),
    other => Err(LxError::type_err(format!("trim expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_trim_start(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  match &args[0] {
    Value::Str(s) => Ok(Value::Str(Arc::from(s.trim_start()))),
    other => Err(LxError::type_err(format!("trim_start expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_trim_end(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  match &args[0] {
    Value::Str(s) => Ok(Value::Str(Arc::from(s.trim_end()))),
    other => Err(LxError::type_err(format!("trim_end expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_upper(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  match &args[0] {
    Value::Str(s) => Ok(Value::Str(Arc::from(s.to_uppercase().as_str()))),
    other => Err(LxError::type_err(format!("upper expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_lower(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  match &args[0] {
    Value::Str(s) => Ok(Value::Str(Arc::from(s.to_lowercase().as_str()))),
    other => Err(LxError::type_err(format!("lower expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_lines(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  match &args[0] {
    Value::Str(s) => {
      let items: Vec<Value> = s.lines().map(|l| Value::Str(Arc::from(l))).collect();
      Ok(Value::List(Arc::new(items)))
    },
    other => Err(LxError::type_err(format!("lines expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_chars(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  match &args[0] {
    Value::Str(s) => {
      let items: Vec<Value> = s.chars().map(|c| Value::Str(Arc::from(c.to_string().as_str()))).collect();
      Ok(Value::List(Arc::new(items)))
    },
    other => Err(LxError::type_err(format!("chars expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_byte_len(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  match &args[0] {
    Value::Str(s) => Ok(Value::Int(BigInt::from(s.len()))),
    other => Err(LxError::type_err(format!("byte_len expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_split(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  let sep = args[0].as_str().ok_or_else(|| LxError::type_err("split: first arg must be Str", span))?;
  let s = args[1].as_str().ok_or_else(|| LxError::type_err("split: second arg must be Str", span))?;
  let items: Vec<Value> = s.split(sep).map(|p| Value::Str(Arc::from(p))).collect();
  Ok(Value::List(Arc::new(items)))
}

fn bi_join(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  let sep = args[0].as_str().ok_or_else(|| LxError::type_err("join: first arg must be Str", span))?;
  let list = args[1].as_list().ok_or_else(|| LxError::type_err("join: second arg must be List", span))?;
  let parts: Result<Vec<&str>, LxError> = list.iter().map(|v| v.as_str().ok_or_else(|| LxError::type_err("join: list elements must be Str", span))).collect();
  Ok(Value::Str(Arc::from(parts?.join(sep).as_str())))
}

fn bi_replace(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  let old = args[0].as_str().ok_or_else(|| LxError::type_err("replace: first arg must be Str", span))?;
  let new = args[1].as_str().ok_or_else(|| LxError::type_err("replace: second arg must be Str", span))?;
  let s = args[2].as_str().ok_or_else(|| LxError::type_err("replace: third arg must be Str", span))?;
  Ok(Value::Str(Arc::from(s.replacen(old, new, 1).as_str())))
}

fn bi_replace_all(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  let old = args[0].as_str().ok_or_else(|| LxError::type_err("replace_all: first arg must be Str", span))?;
  let new = args[1].as_str().ok_or_else(|| LxError::type_err("replace_all: second arg must be Str", span))?;
  let s = args[2].as_str().ok_or_else(|| LxError::type_err("replace_all: third arg must be Str", span))?;
  Ok(Value::Str(Arc::from(s.replace(old, new).as_str())))
}

fn bi_repeat(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  let n = args[0].as_int().ok_or_else(|| LxError::type_err("repeat: first arg must be Int", span))?;
  let s = args[1].as_str().ok_or_else(|| LxError::type_err("repeat: second arg must be Str", span))?;
  let count = n.to_usize().ok_or_else(|| LxError::runtime("repeat: count out of range", span))?;
  Ok(Value::Str(Arc::from(s.repeat(count).as_str())))
}

fn bi_starts(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  let prefix = args[0].as_str().ok_or_else(|| LxError::type_err("starts?: first arg must be Str", span))?;
  let s = args[1].as_str().ok_or_else(|| LxError::type_err("starts?: second arg must be Str", span))?;
  Ok(Value::Bool(s.starts_with(prefix)))
}

fn bi_ends(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  let suffix = args[0].as_str().ok_or_else(|| LxError::type_err("ends?: first arg must be Str", span))?;
  let s = args[1].as_str().ok_or_else(|| LxError::type_err("ends?: second arg must be Str", span))?;
  Ok(Value::Bool(s.ends_with(suffix)))
}

fn bi_pad_left(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  let width = args[0]
    .as_int()
    .ok_or_else(|| LxError::type_err("pad_left: first arg must be Int", span))?
    .to_usize()
    .ok_or_else(|| LxError::runtime("pad_left: width out of range", span))?;
  let s = args[1].as_str().ok_or_else(|| LxError::type_err("pad_left: second arg must be Str", span))?;
  let char_count = s.chars().count();
  if char_count >= width {
    Ok(Value::Str(Arc::from(s)))
  } else {
    let padding = " ".repeat(width - char_count);
    Ok(Value::Str(Arc::from(format!("{padding}{s}").as_str())))
  }
}

fn bi_pad_right(args: &[Value], span: Span, _ctx: &RuntimeCtx) -> Result<Value, LxError> {
  let width = args[0]
    .as_int()
    .ok_or_else(|| LxError::type_err("pad_right: first arg must be Int", span))?
    .to_usize()
    .ok_or_else(|| LxError::runtime("pad_right: width out of range", span))?;
  let s = args[1].as_str().ok_or_else(|| LxError::type_err("pad_right: second arg must be Str", span))?;
  let char_count = s.chars().count();
  if char_count >= width {
    Ok(Value::Str(Arc::from(s)))
  } else {
    let padding = " ".repeat(width - char_count);
    Ok(Value::Str(Arc::from(format!("{s}{padding}").as_str())))
  }
}

pub(super) fn register(env: &mut Env) {
  env.bind("trim".into(), mk("trim", 1, bi_trim));
  env.bind("trim_start".into(), mk("trim_start", 1, bi_trim_start));
  env.bind("trim_end".into(), mk("trim_end", 1, bi_trim_end));
  env.bind("upper".into(), mk("upper", 1, bi_upper));
  env.bind("lower".into(), mk("lower", 1, bi_lower));
  env.bind("lines".into(), mk("lines", 1, bi_lines));
  env.bind("chars".into(), mk("chars", 1, bi_chars));
  env.bind("byte_len".into(), mk("byte_len", 1, bi_byte_len));
  env.bind("split".into(), mk("split", 2, bi_split));
  env.bind("join".into(), mk("join", 2, bi_join));
  env.bind("replace".into(), mk("replace", 3, bi_replace));
  env.bind("replace_all".into(), mk("replace_all", 3, bi_replace_all));
  env.bind("repeat".into(), mk("repeat", 2, bi_repeat));
  env.bind("starts?".into(), mk("starts?", 2, bi_starts));
  env.bind("ends?".into(), mk("ends?", 2, bi_ends));
  env.bind("pad_left".into(), mk("pad_left", 2, bi_pad_left));
  env.bind("pad_right".into(), mk("pad_right", 2, bi_pad_right));
}
