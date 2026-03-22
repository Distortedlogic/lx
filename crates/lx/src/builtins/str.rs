use std::sync::Arc;

use crate::env::Env;
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

#[path = "str_extra.rs"]
mod str_extra;

fn str_transform(args: &[LxVal], span: SourceSpan, name: &str, f: fn(&str) -> String) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => Ok(LxVal::str(f(s))),
    other => Err(LxError::type_err(format!("{name} expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_trim(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "trim", |s| s.trim().to_string())
}

fn bi_trim_start(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "trim_start", |s| s.trim_start().to_string())
}

fn bi_trim_end(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "trim_end", |s| s.trim_end().to_string())
}

fn bi_upper(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "upper", |s| s.to_uppercase())
}

fn bi_lower(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  str_transform(args, span, "lower", |s| s.to_lowercase())
}

fn bi_lines(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => {
      let items: Vec<LxVal> = s.lines().map(LxVal::str).collect();
      Ok(LxVal::list(items))
    },
    other => Err(LxError::type_err(format!("lines expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_chars(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => {
      let items: Vec<LxVal> = s.chars().map(|c| LxVal::str(c.to_string())).collect();
      Ok(LxVal::list(items))
    },
    other => Err(LxError::type_err(format!("chars expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_byte_len(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Str(s) => Ok(LxVal::int(s.len())),
    other => Err(LxError::type_err(format!("byte_len expects Str, got {}", other.type_name()), span)),
  }
}

fn bi_split(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let sep = args[0].require_str("split", span)?;
  let s = args[1].require_str("split", span)?;
  let items: Vec<LxVal> = s.split(sep).map(LxVal::str).collect();
  Ok(LxVal::list(items))
}

fn bi_join(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let sep = args[0].require_str("join", span)?;
  let list = args[1].require_list("join", span)?;
  let parts: Result<Vec<&str>, LxError> = list.iter().map(|v| v.require_str("join", span)).collect();
  Ok(LxVal::str(parts?.join(sep)))
}

pub(super) fn register(env: &Env) {
  super::register_builtins!(env, {
    "trim"/1 => bi_trim, "trim_start"/1 => bi_trim_start, "trim_end"/1 => bi_trim_end,
    "upper"/1 => bi_upper, "lower"/1 => bi_lower, "lines"/1 => bi_lines,
    "chars"/1 => bi_chars, "byte_len"/1 => bi_byte_len,
    "split"/2 => bi_split, "join"/2 => bi_join,
    "replace"/3 => str_extra::bi_replace, "replace_all"/3 => str_extra::bi_replace_all,
    "repeat"/2 => str_extra::bi_repeat, "starts?"/2 => str_extra::bi_starts,
    "ends?"/2 => str_extra::bi_ends, "pad_left"/2 => str_extra::bi_pad_left,
    "pad_right"/2 => str_extra::bi_pad_right,
  });
}
