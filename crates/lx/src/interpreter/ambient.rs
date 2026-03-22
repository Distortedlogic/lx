use std::sync::Arc;

use async_recursion::async_recursion;
use indexmap::IndexMap;

use crate::ast::{SExpr, SStmt};
use crate::builtins::mk;
use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

use super::Interpreter;

const AMBIENT_KEY: &str = "__ambient_context";

pub fn get_ambient(interp: &Interpreter) -> IndexMap<crate::sym::Sym, LxVal> {
  match interp.env.get(crate::sym::intern(AMBIENT_KEY)) {
    Some(LxVal::Record(r)) => r.as_ref().clone(),
    _ => IndexMap::new(),
  }
}

fn build_context_record(fields: &IndexMap<crate::sym::Sym, LxVal>) -> LxVal {
  let mut rec = fields.clone();
  let snapshot = LxVal::record(fields.clone());
  rec.insert(crate::sym::intern("current"), mk("context.current", 1, bi_context_current));
  rec.insert(crate::sym::intern("get"), mk("context.get", 1, bi_context_get));
  rec.insert(crate::sym::intern("__snapshot"), snapshot);
  LxVal::record(rec)
}

fn bi_context_current(_args: &[LxVal], _span: SourceSpan, _ctx: &Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError> {
  let fields = AMBIENT_SNAPSHOT.with(|s| s.borrow().clone());
  Ok(LxVal::record(fields))
}

fn bi_context_get(args: &[LxVal], span: SourceSpan, _ctx: &Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError> {
  global_context_get(&args[0], span)
}

pub fn global_context_current() -> Result<LxVal, LxError> {
  let fields = get_ambient_snapshot();
  Ok(LxVal::record(fields))
}

pub fn global_context_get(key_val: &LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
  let key = key_val.require_str("context.get", span)?;
  let fields = get_ambient_snapshot();
  match fields.get(&crate::sym::intern(key)) {
    Some(v) => Ok(LxVal::some(v.clone())),
    None => Ok(LxVal::None),
  }
}

thread_local! {
    static AMBIENT_SNAPSHOT: std::cell::RefCell<IndexMap<crate::sym::Sym, LxVal>> =
        std::cell::RefCell::new(IndexMap::new());
}

fn set_ambient_snapshot(fields: &IndexMap<crate::sym::Sym, LxVal>) {
  AMBIENT_SNAPSHOT.with(|s| {
    *s.borrow_mut() = fields.clone();
  });
}

fn get_ambient_snapshot() -> IndexMap<crate::sym::Sym, LxVal> {
  AMBIENT_SNAPSHOT.with(|s| s.borrow().clone())
}

impl Interpreter {
  #[async_recursion(?Send)]
  pub(super) async fn eval_with_context(&mut self, fields: &[(crate::sym::Sym, SExpr)], body: &[SStmt], _span: SourceSpan) -> Result<LxVal, LxError> {
    let mut new_fields = get_ambient(self);
    for (name, expr) in fields {
      let val = self.eval(expr).await?;
      new_fields.insert(*name, val);
    }
    let saved_env = Arc::clone(&self.env);
    let saved_snapshot = get_ambient_snapshot();
    set_ambient_snapshot(&new_fields);
    let context_record = build_context_record(&new_fields);
    let child = self.env.child();
    child.bind_str(AMBIENT_KEY, LxVal::record(new_fields));
    child.bind_str("context", context_record);
    self.env = Arc::new(child);
    let mut result = LxVal::Unit;
    for stmt in body {
      match self.eval_stmt(stmt).await {
        Ok(v) => result = v,
        Err(e) => {
          self.env = saved_env;
          set_ambient_snapshot(&saved_snapshot);
          return Err(e);
        },
      }
    }
    self.env = saved_env;
    set_ambient_snapshot(&saved_snapshot);
    Ok(result)
  }
}
