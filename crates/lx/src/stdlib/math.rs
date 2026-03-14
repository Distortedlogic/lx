use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("abs".into(), mk("math.abs", 1, bi_abs));
    m.insert("ceil".into(), mk("math.ceil", 1, bi_ceil));
    m.insert("floor".into(), mk("math.floor", 1, bi_floor));
    m.insert("round".into(), mk("math.round", 1, bi_round));
    m.insert("pow".into(), mk("math.pow", 2, bi_pow));
    m.insert("sqrt".into(), mk("math.sqrt", 1, bi_sqrt));
    m.insert("min".into(), mk("math.min", 2, bi_min));
    m.insert("max".into(), mk("math.max", 2, bi_max));
    m.insert("pi".into(), Value::Float(std::f64::consts::PI));
    m.insert("e".into(), Value::Float(std::f64::consts::E));
    m.insert("inf".into(), Value::Float(f64::INFINITY));
    m
}

fn to_f64(v: &Value, span: Span) -> Result<f64, LxError> {
    match v {
        Value::Float(f) => Ok(*f),
        Value::Int(n) => n.to_f64().ok_or_else(|| LxError::runtime("math: Int too large for float", span)),
        other => Err(LxError::type_err(format!("math: expected number, got {}", other.type_name()), span)),
    }
}

fn bi_abs(args: &[Value], span: Span) -> Result<Value, LxError> {
    match &args[0] {
        Value::Int(n) => {
            if n.sign() == num_bigint::Sign::Minus { Ok(Value::Int(-n)) } else { Ok(Value::Int(n.clone())) }
        },
        Value::Float(f) => Ok(Value::Float(f.abs())),
        other => Err(LxError::type_err(format!("math.abs expects number, got {}", other.type_name()), span)),
    }
}

fn bi_ceil(args: &[Value], span: Span) -> Result<Value, LxError> {
    let f = to_f64(&args[0], span)?;
    Ok(Value::Int(BigInt::from(f.ceil() as i64)))
}

fn bi_floor(args: &[Value], span: Span) -> Result<Value, LxError> {
    let f = to_f64(&args[0], span)?;
    Ok(Value::Int(BigInt::from(f.floor() as i64)))
}

fn bi_round(args: &[Value], span: Span) -> Result<Value, LxError> {
    let f = to_f64(&args[0], span)?;
    Ok(Value::Int(BigInt::from(f.round() as i64)))
}

fn bi_pow(args: &[Value], span: Span) -> Result<Value, LxError> {
    match (&args[0], &args[1]) {
        (Value::Int(base), Value::Int(exp)) => {
            let e: u32 = exp.try_into().map_err(|_| LxError::runtime("math.pow: exponent too large or negative", span))?;
            Ok(Value::Int(base.pow(e)))
        },
        _ => {
            let b = to_f64(&args[0], span)?;
            let e = to_f64(&args[1], span)?;
            Ok(Value::Float(b.powf(e)))
        },
    }
}

fn bi_sqrt(args: &[Value], span: Span) -> Result<Value, LxError> {
    let f = to_f64(&args[0], span)?;
    Ok(Value::Float(f.sqrt()))
}

fn bi_min(args: &[Value], span: Span) -> Result<Value, LxError> {
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.min(b).clone())),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
        _ => {
            let a = to_f64(&args[0], span)?;
            let b = to_f64(&args[1], span)?;
            Ok(Value::Float(a.min(b)))
        },
    }
}

fn bi_max(args: &[Value], span: Span) -> Result<Value, LxError> {
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.max(b).clone())),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
        _ => {
            let a = to_f64(&args[0], span)?;
            let b = to_f64(&args[1], span)?;
            Ok(Value::Float(a.max(b)))
        },
    }
}
