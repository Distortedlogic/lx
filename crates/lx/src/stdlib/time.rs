use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone, Utc};
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("now".into(), mk("time.now", 1, bi_now));
    m.insert("sleep".into(), mk("time.sleep", 1, bi_sleep));
    m.insert("format".into(), mk("time.format", 2, bi_format));
    m.insert("parse".into(), mk("time.parse", 2, bi_parse));
    m
}

fn bi_now(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    let now = Utc::now();
    Ok(timestamp_to_record(now))
}

fn bi_sleep(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let ms = match &args[0] {
        Value::Int(n) => {
            let v: i64 = n
                .try_into()
                .map_err(|_| LxError::type_err("time.sleep: ms too large", span))?;
            if v < 0 {
                return Err(LxError::type_err(
                    "time.sleep: ms must be non-negative",
                    span,
                ));
            }
            v as u64
        }
        Value::Float(f) => {
            if *f < 0.0 {
                return Err(LxError::type_err(
                    "time.sleep: ms must be non-negative",
                    span,
                ));
            }
            *f as u64
        }
        _ => {
            return Err(LxError::type_err(
                "time.sleep expects Int or Float ms",
                span,
            ));
        }
    };
    std::thread::sleep(std::time::Duration::from_millis(ms));
    Ok(Value::Unit)
}

fn bi_format(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let fmt = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("time.format expects Str format", span))?;
    let ts = record_to_datetime(&args[1], span)?;
    let formatted = ts.format(fmt).to_string();
    Ok(Value::Str(Arc::from(formatted.as_str())))
}

fn bi_parse(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let fmt = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("time.parse expects Str format", span))?;
    let input = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("time.parse expects Str input", span))?;
    match DateTime::parse_from_str(input, fmt) {
        Ok(dt) => Ok(Value::Ok(Box::new(timestamp_to_record(
            dt.with_timezone(&Utc),
        )))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("time.parse: {e}").as_str(),
        ))))),
    }
}

fn timestamp_to_record(dt: DateTime<Utc>) -> Value {
    let local: DateTime<Local> = dt.with_timezone(&Local);
    record! {
        "epoch" => Value::Int(BigInt::from(dt.timestamp())),
        "ms" => Value::Int(BigInt::from(dt.timestamp_millis())),
        "iso" => Value::Str(Arc::from(dt.to_rfc3339().as_str())),
        "local" => Value::Str(Arc::from(local.to_rfc3339().as_str())),
    }
}

fn record_to_datetime(val: &Value, span: Span) -> Result<DateTime<Utc>, LxError> {
    match val {
        Value::Record(fields) => {
            if let Some(Value::Int(epoch)) = fields.get("epoch") {
                let secs: i64 = epoch
                    .try_into()
                    .map_err(|_| LxError::type_err("time: epoch too large", span))?;
                return Utc
                    .timestamp_opt(secs, 0)
                    .single()
                    .ok_or_else(|| LxError::runtime("time: invalid epoch", span));
            }
            if let Some(Value::Int(ms)) = fields.get("ms") {
                let millis: i64 = ms
                    .try_into()
                    .map_err(|_| LxError::type_err("time: ms too large", span))?;
                return Utc
                    .timestamp_millis_opt(millis)
                    .single()
                    .ok_or_else(|| LxError::runtime("time: invalid ms", span));
            }
            if let Some(Value::Str(iso)) = fields.get("iso") {
                return DateTime::parse_from_rfc3339(iso)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| LxError::runtime(format!("time: bad iso: {e}"), span));
            }
            Err(LxError::type_err(
                "time: record needs epoch, ms, or iso field",
                span,
            ))
        }
        _ => Err(LxError::type_err("time: expected timestamp Record", span)),
    }
}
