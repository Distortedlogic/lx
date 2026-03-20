use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::{LogLevel, RuntimeCtx};
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::mk;

fn make_log_builtin(name: &'static str, level: LogLevel) -> Value {
    fn log_fn(
        args: &[Value],
        span: Span,
        ctx: &Arc<RuntimeCtx>,
        level: LogLevel,
        name: &str,
    ) -> Result<Value, LxError> {
        let s = args[0].as_str().ok_or_else(|| {
            LxError::type_err(
                format!("log.{name} expects Str, got {}", args[0].type_name()),
                span,
            )
        })?;
        ctx.log.log(level, s);
        Ok(Value::Unit)
    }
    match level {
        LogLevel::Info => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Info, "info")),
        LogLevel::Warn => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Warn, "warn")),
        LogLevel::Err => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Err, "err")),
        LogLevel::Debug => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Debug, "debug")),
    }
}

fn bi_not(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Bool(b) => Ok(Value::Bool(!b)),
        other => Err(LxError::type_err(
            format!("not expects Bool, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_len(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let n = match &args[0] {
        Value::Str(s) => s.chars().count(),
        Value::List(l) => l.len(),
        Value::Record(r) => r.len(),
        Value::Map(m) => m.len(),
        Value::Tuple(t) => t.len(),
        Value::Store { id } => crate::stdlib::store_len(*id),
        other => {
            return Err(LxError::type_err(
                format!("len expects collection, got {}", other.type_name()),
                span,
            ));
        }
    };
    Ok(Value::Int(BigInt::from(n)))
}

fn bi_empty(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let empty = match &args[0] {
        Value::Str(s) => s.is_empty(),
        Value::List(l) => l.is_empty(),
        Value::Record(r) => r.is_empty(),
        Value::Map(m) => m.is_empty(),
        Value::Tuple(t) => t.is_empty(),
        Value::Store { id } => crate::stdlib::store_len(*id) == 0,
        other => {
            return Err(LxError::type_err(
                format!("empty? expects collection, got {}", other.type_name()),
                span,
            ));
        }
    };
    Ok(Value::Bool(empty))
}

fn bi_to_str(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(Value::Str(Arc::from(format!("{}", args[0]).as_str())))
}

fn bi_identity(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(args[0].clone())
}

fn bi_dbg(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    eprintln!("[dbg] {}", args[0]);
    Ok(args[0].clone())
}

fn bi_ok_q(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(Value::Bool(matches!(&args[0], Value::Ok(_))))
}

fn bi_err_q(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(Value::Bool(matches!(&args[0], Value::Err(_))))
}

fn bi_some_q(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(Value::Bool(matches!(&args[0], Value::Some(_))))
}

fn bi_even(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Bool(n % BigInt::from(2) == BigInt::from(0))),
        other => Err(LxError::type_err(
            format!("even? expects Int, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_odd(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Bool(n % BigInt::from(2) != BigInt::from(0))),
        other => Err(LxError::type_err(
            format!("odd? expects Int, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_type_of(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(Value::Str(Arc::from(args[0].type_name())))
}

fn bi_print(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    println!("{}", args[0]);
    Ok(Value::Unit)
}

pub fn register(env: &mut Env) {
    env.bind("true".into(), Value::Bool(true));
    env.bind("false".into(), Value::Bool(false));
    env.bind("None".into(), Value::None);
    env.bind(
        "Ok".into(),
        mk("Ok", 1, |a, _, _ctx| Ok(Value::Ok(Box::new(a[0].clone())))),
    );
    env.bind(
        "Err".into(),
        mk("Err", 1, |a, _, _ctx| {
            Ok(Value::Err(Box::new(a[0].clone())))
        }),
    );
    env.bind(
        "Some".into(),
        mk("Some", 1, |a, _, _ctx| {
            Ok(Value::Some(Box::new(a[0].clone())))
        }),
    );
    env.bind("not".into(), mk("not", 1, bi_not));
    env.bind("len".into(), mk("len", 1, bi_len));
    env.bind("empty?".into(), mk("empty?", 1, bi_empty));
    env.bind("to_str".into(), mk("to_str", 1, bi_to_str));
    env.bind("identity".into(), mk("identity", 1, bi_identity));
    env.bind("dbg".into(), mk("dbg", 1, bi_dbg));
    env.bind("ok?".into(), mk("ok?", 1, bi_ok_q));
    env.bind("err?".into(), mk("err?", 1, bi_err_q));
    env.bind("some?".into(), mk("some?", 1, bi_some_q));
    env.bind("even?".into(), mk("even?", 1, bi_even));
    env.bind("odd?".into(), mk("odd?", 1, bi_odd));
    env.bind("type_of".into(), mk("type_of", 1, bi_type_of));
    env.bind("print".into(), mk("print", 1, bi_print));
    super::convert::register(env);
    super::str::register(env);
    super::coll::register(env);
    let mut log_fields = IndexMap::new();
    log_fields.insert("info".into(), make_log_builtin("log.info", LogLevel::Info));
    log_fields.insert("warn".into(), make_log_builtin("log.warn", LogLevel::Warn));
    log_fields.insert("err".into(), make_log_builtin("log.err", LogLevel::Err));
    log_fields.insert(
        "debug".into(),
        make_log_builtin("log.debug", LogLevel::Debug),
    );
    env.bind("log".into(), Value::Record(Arc::new(log_fields)));
    super::hof::register(env);
    env.bind("Store".into(), crate::stdlib::build_constructor());
    env.bind("method_of".into(), mk("method_of", 2, bi_method_of));
    env.bind("methods_of".into(), mk("methods_of", 1, bi_methods_of));
    let mut ctx_fields = IndexMap::new();
    ctx_fields.insert(
        "current".into(),
        mk("context.current", 1, bi_global_context_current),
    );
    ctx_fields.insert("get".into(), mk("context.get", 1, bi_global_context_get));
    env.bind("context".into(), Value::Record(Arc::new(ctx_fields)));
}

fn bi_method_of(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = match &args[1] {
        Value::Str(s) => s.as_ref(),
        _ => return Ok(Value::None),
    };
    match &args[0] {
        Value::Object { methods, .. } | Value::Class { methods, .. } => match methods.get(name) {
            Some(method) => Ok(inject_self_for_method(method, &args[0])),
            None => Ok(Value::None),
        },
        _ => Ok(Value::None),
    }
}

fn inject_self_for_method(method: &Value, self_val: &Value) -> Value {
    if let Value::Func(lf) = method {
        let mut env = lf.closure.child();
        env.bind("self".to_string(), self_val.clone());
        let mut lf = lf.clone();
        lf.closure = env.into_arc();
        Value::Func(lf)
    } else {
        method.clone()
    }
}

fn bi_global_context_current(
    _args: &[Value],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    crate::interpreter::ambient::global_context_current()
}

fn bi_global_context_get(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    crate::interpreter::ambient::global_context_get(&args[0], span)
}

fn bi_methods_of(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let names = match &args[0] {
        Value::Object { methods, .. } | Value::Class { methods, .. } => methods
            .keys()
            .map(|k| Value::Str(Arc::from(k.as_str())))
            .collect(),
        _ => vec![],
    };
    Ok(Value::List(Arc::new(names)))
}
