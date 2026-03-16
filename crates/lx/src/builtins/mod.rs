mod call;
pub(crate) mod coll;
mod hof;
mod hof_extra;
mod str;

use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::{LogLevel, RuntimeCtx};
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

fn make_log_builtin(level: &'static str) -> Value {
    fn log_info(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
        let s = args[0]
            .as_str()
            .ok_or_else(|| LxError::type_err("log.info expects Str", span))?;
        ctx.log.log(LogLevel::Info, s);
        Ok(Value::Unit)
    }
    fn log_warn(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
        let s = args[0]
            .as_str()
            .ok_or_else(|| LxError::type_err("log.warn expects Str", span))?;
        ctx.log.log(LogLevel::Warn, s);
        Ok(Value::Unit)
    }
    fn log_err(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
        let s = args[0]
            .as_str()
            .ok_or_else(|| LxError::type_err("log.err expects Str", span))?;
        ctx.log.log(LogLevel::Err, s);
        Ok(Value::Unit)
    }
    fn log_debug(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
        let s = args[0]
            .as_str()
            .ok_or_else(|| LxError::type_err("log.debug expects Str", span))?;
        ctx.log.log(LogLevel::Debug, s);
        Ok(Value::Unit)
    }
    match level {
        "info" => mk("log.info", 1, log_info),
        "warn" => mk("log.warn", 1, log_warn),
        "err" => mk("log.err", 1, log_err),
        "debug" => mk("log.debug", 1, log_debug),
        _ => unreachable!(),
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

fn bi_collect(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            let items: Vec<Value> = if *inclusive {
                (*start..=*end)
                    .map(|i| Value::Int(BigInt::from(i)))
                    .collect()
            } else {
                (*start..*end)
                    .map(|i| Value::Int(BigInt::from(i)))
                    .collect()
            };
            Ok(Value::List(Arc::new(items)))
        }
        other => Ok(other.clone()),
    }
}

fn bi_step(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let step = args[0]
        .as_int()
        .ok_or_else(|| LxError::type_err("step: first arg must be Int", span))?;
    let step = step
        .to_i64()
        .ok_or_else(|| LxError::runtime("step: value too large", span))?;
    if step <= 0 {
        return Err(LxError::runtime("step: must be positive", span));
    }
    match &args[1] {
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            let mut items = Vec::new();
            let mut i = *start;
            let limit = if *inclusive { *end + 1 } else { *end };
            while i < limit {
                items.push(Value::Int(BigInt::from(i)));
                i += step;
            }
            Ok(Value::List(Arc::new(items)))
        }
        Value::List(l) => {
            let items: Vec<Value> = l.iter().step_by(step as usize).cloned().collect();
            Ok(Value::List(Arc::new(items)))
        }
        other => Err(LxError::type_err(
            format!("step: expects Range/List, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_require(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[1] {
        Value::Some(v) => Ok(Value::Ok(v.clone())),
        Value::None => Ok(Value::Err(Box::new(args[0].clone()))),
        other => Ok(Value::Ok(Box::new(other.clone()))),
    }
}

fn bi_parse_int(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Str(s) => match s.parse::<BigInt>() {
            Ok(n) => Ok(Value::Ok(Box::new(Value::Int(n)))),
            Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
                e.to_string().as_str(),
            ))))),
        },
        other => Err(LxError::type_err(
            format!("parse_int expects Str, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_parse_float(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Str(s) => match s.parse::<f64>() {
            Ok(f) => Ok(Value::Ok(Box::new(Value::Float(f)))),
            Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
                e.to_string().as_str(),
            ))))),
        },
        other => Err(LxError::type_err(
            format!("parse_float expects Str, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_to_int(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Int(_) => Ok(args[0].clone()),
        Value::Float(f) => Ok(Value::Int(BigInt::from(*f as i64))),
        Value::Str(s) => s
            .parse::<BigInt>()
            .map(Value::Int)
            .map_err(|e| LxError::runtime(format!("to_int: {e}"), span)),
        Value::Bool(b) => Ok(Value::Int(if *b { 1.into() } else { 0.into() })),
        other => Err(LxError::type_err(
            format!("to_int: cannot convert {}", other.type_name()),
            span,
        )),
    }
}

fn bi_to_float(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Float(_) => Ok(args[0].clone()),
        Value::Int(n) => n
            .to_f64()
            .map(Value::Float)
            .ok_or_else(|| LxError::runtime("to_float: int too large", span)),
        Value::Str(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|e| LxError::runtime(format!("to_float: {e}"), span)),
        other => Err(LxError::type_err(
            format!("to_float: cannot convert {}", other.type_name()),
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

fn bi_timeout(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let secs = match &args[0] {
        Value::Int(n) => n
            .to_f64()
            .ok_or_else(|| LxError::runtime("timeout: value too large", span))?,
        Value::Float(f) => *f,
        other => {
            return Err(LxError::type_err(
                format!("timeout expects number, got {}", other.type_name()),
                span,
            ));
        }
    };
    std::thread::sleep(std::time::Duration::from_secs_f64(secs));
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
    env.bind("to_int".into(), mk("to_int", 1, bi_to_int));
    env.bind("to_float".into(), mk("to_float", 1, bi_to_float));
    env.bind("identity".into(), mk("identity", 1, bi_identity));
    env.bind("dbg".into(), mk("dbg", 1, bi_dbg));
    env.bind("ok?".into(), mk("ok?", 1, bi_ok_q));
    env.bind("err?".into(), mk("err?", 1, bi_err_q));
    env.bind("some?".into(), mk("some?", 1, bi_some_q));
    env.bind("even?".into(), mk("even?", 1, bi_even));
    env.bind("odd?".into(), mk("odd?", 1, bi_odd));
    env.bind("collect".into(), mk("collect", 1, bi_collect));
    env.bind("step".into(), mk("step", 2, bi_step));
    env.bind("require".into(), mk("require", 2, bi_require));
    env.bind("parse_int".into(), mk("parse_int", 1, bi_parse_int));
    env.bind("parse_float".into(), mk("parse_float", 1, bi_parse_float));
    env.bind("type_of".into(), mk("type_of", 1, bi_type_of));
    env.bind("print".into(), mk("print", 1, bi_print));
    env.bind("timeout".into(), mk("timeout", 1, bi_timeout));
    str::register(env);
    coll::register(env);
    let mut log_fields = IndexMap::new();
    log_fields.insert("info".into(), make_log_builtin("info"));
    log_fields.insert("warn".into(), make_log_builtin("warn"));
    log_fields.insert("err".into(), make_log_builtin("err"));
    log_fields.insert("debug".into(), make_log_builtin("debug"));
    env.bind("log".into(), Value::Record(Arc::new(log_fields)));
    hof::register(env);
}

pub(crate) fn call_value(
    f: &Value,
    arg: Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    call::call_value(f, arg, span, ctx)
}
