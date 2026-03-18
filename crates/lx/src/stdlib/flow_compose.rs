use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub(crate) fn bi_pipe(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let flows = match &args[0] {
        Value::List(l) => l.as_ref().clone(),
        _ => {
            return Err(LxError::type_err(
                format!(
                    "flow.pipe: expected List of flows, got {} `{}`",
                    args[0].type_name(),
                    args[0].short_display()
                ),
                span,
            ));
        }
    };
    Ok(record! {
        "__flow" => Value::Str(Arc::from("pipe")),
        "flows" => Value::List(Arc::new(flows)),
    })
}

pub(crate) fn bi_par(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let flows = match &args[0] {
        Value::List(l) => l.as_ref().clone(),
        _ => {
            return Err(LxError::type_err(
                format!(
                    "flow.parallel: expected List of flows, got {} `{}`",
                    args[0].type_name(),
                    args[0].short_display()
                ),
                span,
            ));
        }
    };
    Ok(record! {
        "__flow" => Value::Str(Arc::from("par")),
        "flows" => Value::List(Arc::new(flows)),
    })
}

pub(crate) fn bi_branch(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let router = &args[0];
    match router {
        Value::Func(_) | Value::BuiltinFunc(_) => {}
        _ => {
            return Err(LxError::type_err(
                format!(
                    "flow.branch: expected function, got {} `{}`",
                    router.type_name(),
                    router.short_display()
                ),
                span,
            ));
        }
    }
    Ok(record! {
        "__flow" => Value::Str(Arc::from("branch")),
        "router" => router.clone(),
    })
}

pub(crate) fn bi_with_retry(
    args: &[Value],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let opts = &args[0];
    let flow = &args[1];
    Ok(record! {
        "__flow" => Value::Str(Arc::from("retry")),
        "inner" => flow.clone(),
        "opts" => opts.clone(),
    })
}

pub(crate) fn bi_with_timeout(
    args: &[Value],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let seconds = &args[0];
    let flow = &args[1];
    Ok(record! {
        "__flow" => Value::Str(Arc::from("timeout")),
        "inner" => flow.clone(),
        "seconds" => seconds.clone(),
    })
}

pub(crate) fn bi_with_fallback(
    args: &[Value],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let fallback = &args[0];
    let flow = &args[1];
    Ok(record! {
        "__flow" => Value::Str(Arc::from("fallback")),
        "inner" => flow.clone(),
        "fallback" => fallback.clone(),
    })
}
