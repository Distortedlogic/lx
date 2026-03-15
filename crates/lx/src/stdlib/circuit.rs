use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Instant;

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

struct Breaker {
    turns: u64,
    max_turns: u64,
    max_time_secs: f64,
    max_actions: u64,
    repetition_window: usize,
    actions: Vec<String>,
    start: Instant,
    tripped: Option<String>,
}

static BREAKERS: LazyLock<DashMap<u64, Breaker>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("circuit.create", 1, bi_create));
    m.insert("tick".into(), mk("circuit.tick", 1, bi_tick));
    m.insert("record".into(), mk("circuit.record", 2, bi_record));
    m.insert("check".into(), mk("circuit.check", 1, bi_check));
    m.insert("is_tripped".into(), mk("circuit.is_tripped", 1, bi_is_tripped));
    m.insert("reset".into(), mk("circuit.reset", 1, bi_reset));
    m.insert("status".into(), mk("circuit.status", 1, bi_status));
    m
}

fn breaker_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r.get("__breaker_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("circuit: expected breaker record", span)),
        _ => Err(LxError::type_err("circuit: expected breaker Record", span)),
    }
}

fn make_handle(id: u64) -> Value {
    let mut rec = IndexMap::new();
    rec.insert("__breaker_id".into(), Value::Int(BigInt::from(id)));
    Value::Record(Arc::new(rec))
}

fn int_field(r: &IndexMap<String, Value>, key: &str, default: u64) -> u64 {
    r.get(key)
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
        .unwrap_or(default)
}

fn float_field(r: &IndexMap<String, Value>, key: &str, default: f64) -> f64 {
    r.get(key)
        .and_then(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Int(n) => n.to_string().parse().ok(),
            _ => None,
        })
        .unwrap_or(default)
}

fn bi_create(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::Record(opts) = &args[0] else {
        return Err(LxError::type_err("circuit.create expects Record", span));
    };
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let breaker = Breaker {
        turns: 0,
        max_turns: int_field(opts, "max_turns", 100),
        max_time_secs: float_field(opts, "max_time", 300.0),
        max_actions: int_field(opts, "max_actions", 1000),
        repetition_window: int_field(opts, "repetition_window", 5) as usize,
        actions: Vec::new(),
        start: Instant::now(),
        tripped: None,
    };
    BREAKERS.insert(id, breaker);
    Ok(make_handle(id))
}

fn trip_check(b: &mut Breaker) {
    if b.tripped.is_some() {
        return;
    }
    if b.turns >= b.max_turns {
        b.tripped = Some(format!("max_turns: {} >= {}", b.turns, b.max_turns));
        return;
    }
    let elapsed = b.start.elapsed().as_secs_f64();
    if elapsed >= b.max_time_secs {
        b.tripped = Some(format!("max_time: {elapsed:.1}s >= {}s", b.max_time_secs));
        return;
    }
    if b.actions.len() as u64 >= b.max_actions {
        b.tripped = Some(format!("max_actions: {} >= {}", b.actions.len(), b.max_actions));
        return;
    }
    if b.repetition_window > 0 && b.actions.len() >= b.repetition_window {
        let window = &b.actions[b.actions.len() - b.repetition_window..];
        if window.iter().all(|a| a == &window[0]) {
            b.tripped = Some(format!("repetition: last {} actions identical", b.repetition_window));
        }
    }
}

fn bi_tick(args: &[Value], span: Span) -> Result<Value, LxError> {
    let id = breaker_id(&args[0], span)?;
    let mut b = BREAKERS.get_mut(&id)
        .ok_or_else(|| LxError::runtime("circuit: breaker not found", span))?;
    b.turns += 1;
    trip_check(&mut b);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_record(args: &[Value], span: Span) -> Result<Value, LxError> {
    let id = breaker_id(&args[0], span)?;
    let action = args[1].as_str()
        .ok_or_else(|| LxError::type_err("circuit.record: action must be Str", span))?;
    let mut b = BREAKERS.get_mut(&id)
        .ok_or_else(|| LxError::runtime("circuit: breaker not found", span))?;
    b.actions.push(action.to_string());
    trip_check(&mut b);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_check(args: &[Value], span: Span) -> Result<Value, LxError> {
    let id = breaker_id(&args[0], span)?;
    let mut b = BREAKERS.get_mut(&id)
        .ok_or_else(|| LxError::runtime("circuit: breaker not found", span))?;
    trip_check(&mut b);
    match &b.tripped {
        Some(reason) => {
            let mut fields = IndexMap::new();
            fields.insert("reason".into(), Value::Str(Arc::from(reason.as_str())));
            Ok(Value::Err(Box::new(Value::Record(Arc::new(fields)))))
        }
        None => Ok(Value::Ok(Box::new(Value::Unit))),
    }
}

fn bi_is_tripped(args: &[Value], span: Span) -> Result<Value, LxError> {
    let id = breaker_id(&args[0], span)?;
    let b = BREAKERS.get(&id)
        .ok_or_else(|| LxError::runtime("circuit: breaker not found", span))?;
    Ok(Value::Bool(b.tripped.is_some()))
}

fn bi_reset(args: &[Value], span: Span) -> Result<Value, LxError> {
    let id = breaker_id(&args[0], span)?;
    let mut b = BREAKERS.get_mut(&id)
        .ok_or_else(|| LxError::runtime("circuit: breaker not found", span))?;
    b.turns = 0;
    b.actions.clear();
    b.start = Instant::now();
    b.tripped = None;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_status(args: &[Value], span: Span) -> Result<Value, LxError> {
    let id = breaker_id(&args[0], span)?;
    let b = BREAKERS.get(&id)
        .ok_or_else(|| LxError::runtime("circuit: breaker not found", span))?;
    let elapsed = b.start.elapsed().as_secs_f64();
    let actions: Vec<Value> = b.actions.iter()
        .map(|a| Value::Str(Arc::from(a.as_str())))
        .collect();
    let mut fields = IndexMap::new();
    fields.insert("turns".into(), Value::Int(BigInt::from(b.turns)));
    fields.insert("elapsed".into(), Value::Float(elapsed));
    fields.insert("action_count".into(), Value::Int(BigInt::from(b.actions.len())));
    fields.insert("actions".into(), Value::List(Arc::new(actions)));
    fields.insert("tripped".into(), Value::Bool(b.tripped.is_some()));
    if let Some(ref reason) = b.tripped {
        fields.insert("reason".into(), Value::Str(Arc::from(reason.as_str())));
    }
    Ok(Value::Record(Arc::new(fields)))
}
