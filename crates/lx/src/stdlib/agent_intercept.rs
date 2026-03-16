use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{BuiltinFunc, Value};

pub fn mk_intercept() -> Value {
    mk("agent.intercept", 2, bi_intercept)
}

fn bi_intercept(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = &args[0];
    let middleware = &args[1];
    let Value::Record(original) = agent else {
        return Err(LxError::type_err(
            "agent.intercept: first arg must be an agent Record",
            span,
        ));
    };
    let next_fn = make_next_fn(agent);
    let handler = make_intercepted_handler(middleware, &next_fn);
    let mut new_agent = original.as_ref().clone();
    new_agent.shift_remove("__pid");
    new_agent.insert("handler".into(), handler);
    Ok(Value::Record(Arc::new(new_agent)))
}

fn make_next_fn(agent: &Value) -> Value {
    Value::BuiltinFunc(BuiltinFunc {
        name: "agent.intercept.next",
        arity: 2,
        func: bi_next,
        applied: vec![agent.clone()],
    })
}

fn make_intercepted_handler(middleware: &Value, next_fn: &Value) -> Value {
    Value::BuiltinFunc(BuiltinFunc {
        name: "agent.intercept.handler",
        arity: 3,
        func: bi_intercepted_handler,
        applied: vec![middleware.clone(), next_fn.clone()],
    })
}

fn bi_intercepted_handler(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let middleware = &args[0];
    let next_fn = &args[1];
    let msg = &args[2];
    let partial = call_value(middleware, msg.clone(), span, ctx)?;
    call_value(&partial, next_fn.clone(), span, ctx)
}

fn bi_next(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = &args[0];
    let msg = &args[1];
    let Value::Record(r) = agent else {
        return Err(LxError::type_err(
            "agent.intercept.next: captured agent is not a Record",
            span,
        ));
    };
    if let Some(pid_val) = r.get("__pid") {
        let pid: u32 = pid_val
            .as_int()
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("agent.intercept.next: invalid __pid", span))?;
        return super::agent::ask_subprocess(pid, msg, span);
    }
    let handler = r.get("handler").ok_or_else(|| {
        LxError::runtime(
            "agent.intercept.next: agent has no 'handler' or '__pid'",
            span,
        )
    })?;
    call_value(handler, msg.clone(), span, ctx)
}
