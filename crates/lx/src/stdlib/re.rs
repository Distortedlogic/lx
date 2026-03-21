use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::LxVal;

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("match".into(), mk("re.match", 2, bi_match));
  m.insert("find_all".into(), mk("re.find_all", 2, bi_find_all));
  m.insert("replace".into(), mk("re.replace", 3, bi_replace));
  m.insert("replace_all".into(), mk("re.replace_all", 3, bi_replace_all));
  m.insert("split".into(), mk("re.split", 2, bi_split));
  m.insert("is_match".into(), mk("re.is_match", 2, bi_is_match));
  m
}

enum RePattern<'a> {
  Compiled(&'a regex::Regex),
  Raw(&'a str),
}

fn get_pattern(v: &LxVal, span: Span) -> Result<RePattern<'_>, LxError> {
  match v {
    LxVal::Regex(r) => Ok(RePattern::Compiled(r)),
    LxVal::Str(s) => Ok(RePattern::Raw(s.as_ref())),
    other => Err(LxError::type_err(format!("re: expected Regex or Str pattern, got {}", other.type_name()), span)),
  }
}

fn to_regex<'a>(pat: &'a RePattern<'a>, span: Span) -> Result<std::borrow::Cow<'a, regex::Regex>, LxError> {
  match pat {
    RePattern::Compiled(r) => Ok(std::borrow::Cow::Borrowed(r)),
    RePattern::Raw(s) => {
      let re = regex::Regex::new(s).map_err(|e| LxError::runtime(format!("re: invalid pattern: {e}"), span))?;
      Ok(std::borrow::Cow::Owned(re))
    },
  }
}

fn bi_match(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let pat = get_pattern(&args[0], span)?;
  let input = args[1].as_str().ok_or_else(|| LxError::type_err("re.match expects Str input", span))?;
  let re = to_regex(&pat, span)?;
  match re.find(input) {
    Some(m) => Ok(LxVal::Some(Box::new(record! {
        "text" => LxVal::str(m.as_str()),
        "start" => LxVal::Int(m.start().into()),
        "end" => LxVal::Int(m.end().into()),
    }))),
    None => Ok(LxVal::None),
  }
}

fn bi_find_all(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let pat = get_pattern(&args[0], span)?;
  let input = args[1].as_str().ok_or_else(|| LxError::type_err("re.find_all expects Str input", span))?;
  let re = to_regex(&pat, span)?;
  let matches: Vec<LxVal> = re.find_iter(input).map(|m| LxVal::str(m.as_str())).collect();
  Ok(LxVal::list(matches))
}

fn bi_replace(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let pat = get_pattern(&args[0], span)?;
  let replacement = args[1].as_str().ok_or_else(|| LxError::type_err("re.replace expects Str replacement", span))?;
  let input = args[2].as_str().ok_or_else(|| LxError::type_err("re.replace expects Str input", span))?;
  let re = to_regex(&pat, span)?;
  let result = re.replace(input, replacement);
  Ok(LxVal::str(result.as_ref()))
}

fn bi_replace_all(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let pat = get_pattern(&args[0], span)?;
  let replacement = args[1].as_str().ok_or_else(|| LxError::type_err("re.replace_all expects Str replacement", span))?;
  let input = args[2].as_str().ok_or_else(|| LxError::type_err("re.replace_all expects Str input", span))?;
  let re = to_regex(&pat, span)?;
  let result = re.replace_all(input, replacement);
  Ok(LxVal::str(result.as_ref()))
}

fn bi_split(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let pat = get_pattern(&args[0], span)?;
  let input = args[1].as_str().ok_or_else(|| LxError::type_err("re.split expects Str input", span))?;
  let re = to_regex(&pat, span)?;
  let parts: Vec<LxVal> = re.split(input).map(LxVal::str).collect();
  Ok(LxVal::list(parts))
}

fn bi_is_match(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let pat = get_pattern(&args[0], span)?;
  let input = args[1].as_str().ok_or_else(|| LxError::type_err("re.is_match expects Str input", span))?;
  let re = to_regex(&pat, span)?;
  Ok(LxVal::Bool(re.is_match(input)))
}
