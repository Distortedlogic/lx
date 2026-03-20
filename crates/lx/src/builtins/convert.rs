use std::sync::Arc;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::mk;

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
        Value::Stream { rx, .. } => {
            let items: Vec<Value> = rx.lock().iter().collect();
            Ok(Value::List(Arc::new(items)))
        }
        other => Ok(other.clone()),
    }
}

fn bi_step(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let step = args[0].as_int().ok_or_else(|| {
        LxError::type_err(
            format!("step: first arg must be Int, got {}", args[0].type_name()),
            span,
        )
    })?;
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

pub(super) fn register(env: &mut Env) {
    env.bind("collect".into(), mk("collect", 1, bi_collect));
    env.bind("step".into(), mk("step", 2, bi_step));
    env.bind("require".into(), mk("require", 2, bi_require));
    env.bind("parse_int".into(), mk("parse_int", 1, bi_parse_int));
    env.bind("parse_float".into(), mk("parse_float", 1, bi_parse_float));
    env.bind("to_int".into(), mk("to_int", 1, bi_to_int));
    env.bind("to_float".into(), mk("to_float", 1, bi_to_float));
    env.bind("timeout".into(), mk("timeout", 1, bi_timeout));
}
