use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::{BuiltinFunc, BuiltinKind, Value};

pub fn mk_dispatch() -> Value {
    mk("agent.dispatch", 1, bi_dispatch)
}

pub fn mk_dispatch_multi() -> Value {
    mk("agent.dispatch_multi", 1, bi_dispatch_multi)
}

fn bi_dispatch(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let rules = args[0].as_list().ok_or_else(|| {
        LxError::type_err("agent.dispatch: first arg must be a List of rules", span)
    })?;
    let handler = Value::BuiltinFunc(BuiltinFunc {
        name: "agent.dispatch.handler",
        arity: 2,
        kind: BuiltinKind::Sync(bi_dispatch_handler),
        applied: vec![Value::List(Arc::clone(rules))],
    });
    Ok(record! {
        "handler" => handler,
    })
}

fn bi_dispatch_multi(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let rules = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            "agent.dispatch_multi: first arg must be a List of rules",
            span,
        )
    })?;
    let handler = Value::BuiltinFunc(BuiltinFunc {
        name: "agent.dispatch_multi.handler",
        arity: 2,
        kind: BuiltinKind::Sync(bi_dispatch_multi_handler),
        applied: vec![Value::List(Arc::clone(rules))],
    });
    Ok(record! {
        "handler" => handler,
    })
}

fn bi_dispatch_handler(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let rules = args[0]
        .as_list()
        .ok_or_else(|| LxError::runtime("agent.dispatch.handler: invalid rules", span))?;
    let msg = &args[1];
    for rule in rules.as_ref() {
        let Value::Record(r) = rule else { continue };
        let Some(match_val) = r.get("match") else {
            continue;
        };
        if matches_dispatch(match_val, msg, span, ctx)? {
            let transformed = apply_transform(r, msg, span, ctx)?;
            return send_to_target(r, &transformed, span, ctx);
        }
    }
    Ok(Value::Err(Box::new(record! {
        "type" => Value::Str(Arc::from("no_route")),
        "message" => msg.clone(),
    })))
}

fn bi_dispatch_multi_handler(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let rules = args[0]
        .as_list()
        .ok_or_else(|| LxError::runtime("agent.dispatch_multi.handler: invalid rules", span))?;
    let msg = &args[1];
    let mut results = Vec::new();
    for rule in rules.as_ref() {
        let Value::Record(r) = rule else { continue };
        let Some(match_val) = r.get("match") else {
            continue;
        };
        if matches_dispatch(match_val, msg, span, ctx)? {
            let transformed = apply_transform(r, msg, span, ctx)?;
            let result = send_to_target(r, &transformed, span, ctx)?;
            results.push(record! {
                "result" => result,
            });
        }
    }
    Ok(Value::List(Arc::new(results)))
}

fn matches_dispatch(
    pattern: &Value,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<bool, LxError> {
    match pattern {
        Value::Str(s) if s.as_ref() == "default" => Ok(true),
        Value::Func(_) | Value::BuiltinFunc(_) => {
            let result = call_value_sync(pattern, msg.clone(), span, ctx)?;
            Ok(result.as_bool().unwrap_or(false))
        }
        Value::Record(pat) => match msg {
            Value::Record(msg_rec) => {
                for (key, expected) in pat.iter() {
                    match msg_rec.get(key) {
                        Some(actual) if actual == expected => {}
                        _ => return Ok(false),
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        },
        _ => Ok(pattern == msg),
    }
}

fn apply_transform(
    rule: &IndexMap<String, Value>,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    match rule.get("transform") {
        Some(f @ (Value::Func(_) | Value::BuiltinFunc(_))) => {
            call_value_sync(f, msg.clone(), span, ctx)
        }
        _ => Ok(msg.clone()),
    }
}

fn send_to_target(
    rule: &IndexMap<String, Value>,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let target = rule
        .get("to")
        .ok_or_else(|| LxError::runtime("agent.dispatch: rule missing 'to' field", span))?;
    match target {
        Value::Func(_) | Value::BuiltinFunc(_) => call_value_sync(target, msg.clone(), span, ctx),
        Value::Record(r) => {
            if let Some(handler) = r.get("handler") {
                return call_value_sync(handler, msg.clone(), span, ctx);
            }
            if let Some(pid_val) = r.get("__pid") {
                let pid: u32 = pid_val
                    .as_int()
                    .and_then(|n| n.try_into().ok())
                    .ok_or_else(|| LxError::type_err("agent.dispatch: invalid __pid", span))?;
                return super::agent::ask_subprocess(pid, msg, span);
            }
            Err(LxError::runtime(
                "agent.dispatch: 'to' target has no handler or __pid",
                span,
            ))
        }
        _ => Err(LxError::type_err(
            "agent.dispatch: 'to' must be an agent or Fn",
            span,
        )),
    }
}
