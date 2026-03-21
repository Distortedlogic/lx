use std::sync::Arc;

use crate::env::Env;
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::LxVal;

use super::mk;

#[path = "str_extra.rs"]
mod str_extra;

fn str_transform(args: &[LxVal], span: Span, name: &str, f: fn(&str) -> String) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => Ok(LxVal::str(f(s))),
    other => Err(LxError::type_err(format!("{name} expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_trim(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "trim", |s| s.trim().to_string())
}

fn bi_trim_start(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "trim_start", |s| s.trim_start().to_string())
}

fn bi_trim_end(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "trim_end", |s| s.trim_end().to_string())
}

fn bi_upper(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "upper", |s| s.to_uppercase())
}

fn bi_lower(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "lower", |s| s.to_lowercase())
}

fn bi_lines(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => {
      let items: Vec<LxVal> = s.lines().map(LxVal::str).collect();
      Ok(LxVal::list(items))
    },
    other => Err(LxError::type_err(format!("lines expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_chars(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => {
      let items: Vec<LxVal> = s.chars().map(|c| LxVal::str(c.to_string())).collect();
      Ok(LxVal::list(items))
    },
    other => Err(LxError::type_err(format!("chars expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_byte_len(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => Ok(LxVal::int(s.len())),
    other => Err(LxError::type_err(format!("byte_len expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_split(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let sep = args[0].require_str("split", span)?;
  let s = args[1].require_str("split", span)?;
  let items: Vec<LxVal> = s.split(sep).map(LxVal::str).collect();
  Ok(LxVal::list(items))
}

fn bi_join(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let sep = args[0].require_str("join", span)?;
  let list = args[1].require_list("join", span)?;
  let parts: Result<Vec<&str>, LxError> = list.iter().map(|v| v.require_str("join", span)).collect();
  Ok(LxVal::str(parts?.join(sep)))
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
  env.bind("replace".into(), mk("replace", 3, str_extra::bi_replace));
  env.bind("replace_all".into(), mk("replace_all", 3, str_extra::bi_replace_all));
  env.bind("repeat".into(), mk("repeat", 2, str_extra::bi_repeat));
  env.bind("starts?".into(), mk("starts?", 2, str_extra::bi_starts));
  env.bind("ends?".into(), mk("ends?", 2, str_extra::bi_ends));
  env.bind("pad_left".into(), mk("pad_left", 2, str_extra::bi_pad_left));
  env.bind("pad_right".into(), mk("pad_right", 2, str_extra::bi_pad_right));
}
