mod call;
pub(crate) mod coll;
mod coll_transform;
mod convert;
mod hof;
mod hof_extra;
mod hof_parallel;
mod register;
mod str;

use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{AsyncBuiltinFn, BuiltinFunc, BuiltinKind, SyncBuiltinFn, LxVal};

pub fn mk(name: &'static str, arity: usize, func: SyncBuiltinFn) -> LxVal {
    LxVal::BuiltinFunc(BuiltinFunc {
        name,
        arity,
        kind: BuiltinKind::Sync(func),
        applied: Vec::new(),
    })
}

pub fn mk_async(name: &'static str, arity: usize, func: AsyncBuiltinFn) -> LxVal {
    LxVal::BuiltinFunc(BuiltinFunc {
        name,
        arity,
        kind: BuiltinKind::Async(func),
        applied: Vec::new(),
    })
}

pub fn register(env: &mut Env) {
    register::register(env);
}

pub(crate) async fn call_value(
    f: &LxVal,
    arg: LxVal,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    call::call_value(f, arg, span, ctx).await
}

pub(crate) fn call_value_sync(
    f: &LxVal,
    arg: LxVal,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(call::call_value(f, arg, span, ctx))
    })
}
