use std::pin::Pin;
use std::sync::Arc;

use crate::ast::{AstArena, ExprId};
use crate::env::Env;
use crate::error::LxError;
use crate::sym::Sym;
use crate::value::LxVal;

#[derive(Debug, Clone)]
pub struct LxFunc {
  pub params: Vec<Sym>,
  pub defaults: Vec<Option<LxVal>>,
  pub guard: Option<ExprId>,
  pub body: ExprId,
  pub arena: Arc<AstArena>,
  pub closure: Arc<Env>,
  pub arity: usize,
  pub applied: Vec<LxVal>,
  pub source_text: Arc<str>,
  pub source_name: Arc<str>,
}

pub type SyncBuiltinFn = fn(&[LxVal], miette::SourceSpan, &Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError>;

pub type AsyncBuiltinFn =
  fn(Vec<LxVal>, miette::SourceSpan, Arc<crate::runtime::RuntimeCtx>) -> Pin<Box<dyn std::future::Future<Output = Result<LxVal, LxError>>>>;

pub type DynAsyncBuiltinFn = Arc<
  dyn Fn(Vec<LxVal>, miette::SourceSpan, Arc<crate::runtime::RuntimeCtx>) -> Pin<Box<dyn std::future::Future<Output = Result<LxVal, LxError>>>> + Send + Sync,
>;

#[derive(Clone)]
pub enum BuiltinKind {
  Sync(SyncBuiltinFn),
  Async(AsyncBuiltinFn),
  DynAsync(DynAsyncBuiltinFn),
}

#[derive(Clone)]
pub struct BuiltinFunc {
  pub name: &'static str,
  pub arity: usize,
  pub kind: BuiltinKind,
  pub applied: Vec<LxVal>,
}

pub fn mk_dyn_async(name: &'static str, arity: usize, func: DynAsyncBuiltinFn) -> LxVal {
  LxVal::BuiltinFunc(BuiltinFunc { name, arity, kind: BuiltinKind::DynAsync(func), applied: Vec::new() })
}
