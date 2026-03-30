use std::env;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::std_module;
use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<lx_span::sym::Sym, LxVal> {
  std_module! {
    "get"  => "env.get",  1, bi_get;
    "vars" => "env.vars", 1, bi_vars;
    "args" => "env.args", 1, bi_args;
    "cwd"  => "env.cwd",  1, bi_cwd;
    "home" => "env.home", 1, bi_home
  }
}

fn bi_get(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let key = args[0].require_str("env.get", span)?;
  match env::var(key) {
    Ok(val) => Ok(LxVal::some(LxVal::str(val))),
    Err(_) => Ok(LxVal::None),
  }
}

fn bi_vars(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  let mut fields = IndexMap::new();
  for (k, v) in env::vars() {
    fields.insert(lx_span::sym::intern(&k), LxVal::str(v));
  }
  Ok(LxVal::record(fields))
}

fn bi_args(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  let items: Vec<LxVal> = env::args().map(LxVal::str).collect();
  Ok(LxVal::list(items))
}

fn bi_cwd(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  match env::current_dir() {
    Ok(p) => Ok(LxVal::str(p.to_string_lossy())),
    Err(e) => Err(LxError::runtime(format!("env.cwd: {e}"), span)),
  }
}

fn bi_home(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  match env::var("HOME") {
    Ok(h) => Ok(LxVal::some(LxVal::str(h))),
    Err(_) => Ok(LxVal::None),
  }
}
