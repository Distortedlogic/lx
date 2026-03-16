mod call;
pub(crate) mod coll;
mod coll_transform;
mod convert;
mod hof;
mod hof_extra;
mod register;
mod str;

use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{BuiltinFn, BuiltinFunc, Value};

pub fn mk(name: &'static str, arity: usize, func: BuiltinFn) -> Value {
    Value::BuiltinFunc(BuiltinFunc {
        name,
        arity,
        func,
        applied: Vec::new(),
    })
}

pub fn register(env: &mut Env) {
    register::register(env);
}

pub(crate) fn call_value(
    f: &Value,
    arg: Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    call::call_value(f, arg, span, ctx)
}
