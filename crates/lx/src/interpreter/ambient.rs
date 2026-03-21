use std::sync::Arc;

use async_recursion::async_recursion;
use indexmap::IndexMap;

use crate::ast::{SExpr, SStmt};
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::Interpreter;

const AMBIENT_KEY: &str = "__ambient_context";

pub fn get_ambient(interp: &Interpreter) -> IndexMap<String, LxVal> {
  match interp.env.get(AMBIENT_KEY) {
    Some(LxVal::Record(r)) => r.as_ref().clone(),
    _ => IndexMap::new(),
  }
}

fn build_context_record(fields: &IndexMap<String, LxVal>) -> LxVal {
  let mut rec = fields.clone();
  let snapshot = LxVal::record(fields.clone());
  rec.insert("current".into(), mk("context.current", 1, bi_context_current));
  rec.insert("get".into(), mk("context.get", 1, bi_context_get));
  rec.insert("__snapshot".into(), snapshot);
  LxVal::record(rec)
}

fn bi_context_current(_args: &[LxVal], _span: Span, _ctx: &Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError> {
  let fields = AMBIENT_SNAPSHOT.with(|s| s.borrow().clone());
  Ok(LxVal::record(fields))
}

fn bi_context_get(args: &[LxVal], span: Span, _ctx: &Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError> {
  global_context_get(&args[0], span)
}

pub fn global_context_current() -> Result<LxVal, LxError> {
  let fields = get_ambient_snapshot();
  Ok(LxVal::record(fields))
}

pub fn global_context_get(key_val: &LxVal, span: Span) -> Result<LxVal, LxError> {
  let key = key_val.require_str("context.get", span)?;
  let fields = get_ambient_snapshot();
  match fields.get(key) {
    Some(v) => Ok(LxVal::Some(Box::new(v.clone()))),
    None => Ok(LxVal::None),
  }
}

thread_local! {
    static AMBIENT_SNAPSHOT: std::cell::RefCell<IndexMap<String, LxVal>> =
        std::cell::RefCell::new(IndexMap::new());
}

fn set_ambient_snapshot(fields: &IndexMap<String, LxVal>) {
  AMBIENT_SNAPSHOT.with(|s| {
    *s.borrow_mut() = fields.clone();
  });
}

fn get_ambient_snapshot() -> IndexMap<String, LxVal> {
  AMBIENT_SNAPSHOT.with(|s| s.borrow().clone())
}

impl Interpreter {
  #[async_recursion(?Send)]
  pub(super) async fn eval_with_context(&mut self, fields: &[(String, SExpr)], body: &[SStmt], _span: Span) -> Result<LxVal, LxError> {
    let mut new_fields = get_ambient(self);
    for (name, expr) in fields {
      let val = self.eval(expr).await?;
      new_fields.insert(name.clone(), val);
    }
    let saved_env = Arc::clone(&self.env);
    let saved_snapshot = get_ambient_snapshot();
    set_ambient_snapshot(&new_fields);
    let context_record = build_context_record(&new_fields);
    let mut child = self.env.child();
    child.bind(AMBIENT_KEY.into(), LxVal::record(new_fields));
    child.bind("context".into(), context_record);
    self.env = child.into_arc();
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
