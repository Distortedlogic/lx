use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("get".into(), mk("env.get", 1, bi_get));
  m.insert("vars".into(), mk("env.vars", 1, bi_vars));
  m.insert("args".into(), mk("env.args", 1, bi_args));
  m.insert("cwd".into(), mk("env.cwd", 1, bi_cwd));
  m.insert("home".into(), mk("env.home", 1, bi_home));
  m
}

fn bi_get(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let key = args[0].require_str("env.get", span)?;
  match std::env::var(key) {
    Ok(val) => Ok(LxVal::some(LxVal::str(val))),
    Err(_) => Ok(LxVal::None),
  }
}

fn bi_vars(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  let mut fields = IndexMap::new();
  for (k, v) in std::env::vars() {
    fields.insert(k, LxVal::str(v));
  }
  Ok(LxVal::record(fields))
}

fn bi_args(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  let items: Vec<LxVal> = std::env::args().map(LxVal::str).collect();
  Ok(LxVal::list(items))
}

fn bi_cwd(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  match std::env::current_dir() {
    Ok(p) => Ok(LxVal::str(p.to_string_lossy())),
    Err(e) => Err(LxError::runtime(format!("env.cwd: {e}"), span)),
  }
}

fn bi_home(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  match std::env::var("HOME") {
    Ok(h) => Ok(LxVal::some(LxVal::str(h))),
    Err(_) => Ok(LxVal::None),
  }
}
