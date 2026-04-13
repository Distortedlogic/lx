use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::env::Env;
use crate::error::LxError;
use crate::value::LxVal;
use lx_ast::ast::{AstArena, ExprId};
use lx_span::sym::Sym;
use miette::SourceSpan;

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

pub type SyncBuiltinFn = fn(&[LxVal], SourceSpan, &dyn crate::BuiltinCtx) -> Result<LxVal, LxError>;

pub type AsyncBuiltinFn = fn(Vec<LxVal>, SourceSpan, Arc<dyn crate::BuiltinCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>>;

pub type DynAsyncBuiltinFn =
  Arc<dyn Fn(Vec<LxVal>, SourceSpan, Arc<dyn crate::BuiltinCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> + Send + Sync>;

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
