use std::sync::Arc;

use crate::ast::SExpr;
use crate::env::Env;
use crate::error::LxError;
use crate::sym::Sym;
use crate::value::LxVal;

#[derive(Debug, Clone)]
pub struct LxFunc {
  pub params: Vec<Sym>,
  pub defaults: Vec<Option<LxVal>>,
  pub body: Arc<SExpr>,
  pub closure: Arc<Env>,
  pub arity: usize,
  pub applied: Vec<LxVal>,
  pub source_text: Arc<str>,
  pub source_name: Arc<str>,
}

pub type SyncBuiltinFn = fn(&[LxVal], miette::SourceSpan, &Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError>;

pub type AsyncBuiltinFn =
  fn(Vec<LxVal>, miette::SourceSpan, Arc<crate::runtime::RuntimeCtx>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LxVal, LxError>>>>;

#[derive(Clone, Copy)]
pub enum BuiltinKind {
  Sync(SyncBuiltinFn),
  Async(AsyncBuiltinFn),
}

#[derive(Clone)]
pub struct BuiltinFunc {
  pub name: &'static str,
  pub arity: usize,
  pub kind: BuiltinKind,
  pub applied: Vec<LxVal>,
}
