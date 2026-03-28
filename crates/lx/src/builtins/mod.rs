pub(crate) mod agent;
mod call;
pub(crate) mod coll;
mod coll_transform;
mod convert;
mod hof;
mod hof_extra;
mod hof_parallel;
pub(crate) mod llm;
mod register;
mod register_helpers;
mod shell;
mod str;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::env::Env;
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::{AsyncBuiltinFn, BuiltinFunc, BuiltinKind, LxVal, SyncBuiltinFn};
use miette::SourceSpan;

pub(crate) type BoxFut = Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>>;

macro_rules! register_builtins {
  ($env:expr, { $( $name:literal / $arity:literal => $func:expr ),* $(,)? }) => {{
    $( $env.bind_str($name, $crate::builtins::mk($name, $arity, $func)); )*
  }};
  ($env:expr, async { $( $name:literal / $arity:literal => $func:expr ),* $(,)? }) => {{
    $( $env.bind_str($name, $crate::builtins::mk_async($name, $arity, $func)); )*
  }};
}
pub(crate) use register_builtins;

pub fn mk(name: &'static str, arity: usize, func: SyncBuiltinFn) -> LxVal {
  LxVal::BuiltinFunc(BuiltinFunc { name, arity, kind: BuiltinKind::Sync(func), applied: Vec::new() })
}

pub fn mk_async(name: &'static str, arity: usize, func: AsyncBuiltinFn) -> LxVal {
  LxVal::BuiltinFunc(BuiltinFunc { name, arity, kind: BuiltinKind::Async(func), applied: Vec::new() })
}

pub fn register(env: &Env) {
  register::register(env);
}

pub(crate) async fn call_value(f: &LxVal, arg: LxVal, span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  call::call_value(f, arg, span, ctx).await
}

pub(crate) fn call_value_sync(f: &LxVal, arg: LxVal, span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(call::call_value(f, arg, span, ctx)))
}
