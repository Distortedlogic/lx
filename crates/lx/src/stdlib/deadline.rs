use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

struct DeadlineState {
    expires_at: Instant,
}

static DEADLINES: LazyLock<DashMap<u64, DeadlineState>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static SCOPE_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("deadline.create", 1, bi_create));
    m.insert(
        "create_at".into(),
        mk("deadline.create_at", 1, bi_create_at),
    );
    m.insert("scope".into(), mk("deadline.scope", 2, bi_scope));
    m.insert(
        "remaining".into(),
        mk("deadline.remaining", 1, bi_remaining),
    );
    m.insert("expired".into(), mk("deadline.expired", 1, bi_expired));
    m.insert("check".into(), mk("deadline.check", 1, bi_check));
    m.insert("slice".into(), mk("deadline.slice", 1, bi_slice));
    m.insert("extend".into(), mk("deadline.extend", 2, bi_extend));
    m
}

pub fn current_remaining_ms() -> Option<i64> {
    SCOPE_STACK.with(|stack| {
        let stack = stack.borrow();
        let id = stack.last()?;
        let dl = DEADLINES.get(id)?;
        let now = Instant::now();
        if now >= dl.expires_at {
            Some(0)
        } else {
            Some(dl.expires_at.duration_since(now).as_millis() as i64)
        }
    })
}

fn deadline_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__deadline_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("deadline: expected deadline handle", span)),
        _ => Err(LxError::type_err(
            "deadline: expected deadline Record",
            span,
        )),
    }
}

fn make_handle(id: u64) -> Value {
    crate::record! {
        "__deadline_id" => Value::Int(BigInt::from(id)),
    }
}

fn current_scope_id(span: Span) -> Result<u64, LxError> {
    SCOPE_STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .copied()
            .ok_or_else(|| LxError::runtime("deadline: no active deadline scope", span))
    })
}

fn remaining_ms_for(id: u64, span: Span) -> Result<i64, LxError> {
    let dl = DEADLINES
        .get(&id)
        .ok_or_else(|| LxError::runtime("deadline: not found", span))?;
    let now = Instant::now();
    if now >= dl.expires_at {
        Ok(0)
    } else {
        Ok(dl.expires_at.duration_since(now).as_millis() as i64)
    }
}

fn extract_ms(v: &Value, name: &str, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Int(n) => {
            let val: i64 = n
                .try_into()
                .map_err(|_| LxError::type_err(format!("{name}: ms too large"), span))?;
            if val < 0 {
                return Err(LxError::type_err(
                    format!("{name}: ms must be non-negative"),
                    span,
                ));
            }
            Ok(val as u64)
        }
        Value::Float(f) => {
            if *f < 0.0 {
                return Err(LxError::type_err(
                    format!("{name}: ms must be non-negative"),
                    span,
                ));
            }
            Ok(*f as u64)
        }
        _ => Err(LxError::type_err(
            format!("{name} expects Int or Float ms"),
            span,
        )),
    }
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let ms = extract_ms(&args[0], "deadline.create", span)?;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    DEADLINES.insert(
        id,
        DeadlineState {
            expires_at: Instant::now() + Duration::from_millis(ms),
        },
    );
    Ok(Value::Ok(Box::new(make_handle(id))))
}

fn bi_create_at(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let epoch_ms = extract_ms(&args[0], "deadline.create_at", span)?;
    let now_sys = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| LxError::runtime(format!("deadline: system time error: {e}"), span))?;
    let target = Duration::from_millis(epoch_ms);
    let expires_at = if target > now_sys {
        Instant::now() + (target - now_sys)
    } else {
        Instant::now()
    };
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    DEADLINES.insert(id, DeadlineState { expires_at });
    Ok(Value::Ok(Box::new(make_handle(id))))
}

fn bi_scope(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = deadline_id(&args[0], span)?;
    if !DEADLINES.contains_key(&id) {
        return Err(LxError::runtime("deadline: not found", span));
    }
    SCOPE_STACK.with(|stack| stack.borrow_mut().push(id));
    let result = call_value(&args[1], Value::Unit, span, ctx);
    SCOPE_STACK.with(|stack| stack.borrow_mut().pop());
    match result {
        Ok(v) => Ok(Value::Ok(Box::new(v))),
        Err(LxError::Propagate { value, .. }) => Ok(*value),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("{e}").as_str(),
        ))))),
    }
}

fn bi_remaining(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    let id = current_scope_id(span)?;
    let ms = remaining_ms_for(id, span)?;
    Ok(Value::Ok(Box::new(Value::Int(BigInt::from(ms)))))
}

fn bi_expired(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    let id = current_scope_id(span)?;
    let ms = remaining_ms_for(id, span)?;
    Ok(Value::Ok(Box::new(Value::Bool(ms <= 0))))
}

fn bi_check(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    let id = current_scope_id(span)?;
    let ms = remaining_ms_for(id, span)?;
    if ms <= 0 {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "deadline exceeded",
        )))))
    } else {
        Ok(Value::Ok(Box::new(Value::Unit)))
    }
}

fn bi_slice(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pct = match &args[0] {
        Value::Float(f) => *f,
        Value::Int(n) => n.to_f64().unwrap_or(0.0),
        _ => {
            return Err(LxError::type_err(
                "deadline.slice expects Float percentage",
                span,
            ));
        }
    };
    if !(0.0..=1.0).contains(&pct) {
        return Err(LxError::runtime(
            "deadline.slice: percentage must be 0.0..1.0",
            span,
        ));
    }
    let scope_id = current_scope_id(span)?;
    let ms = remaining_ms_for(scope_id, span)?;
    let slice_ms = (ms as f64 * pct) as u64;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    DEADLINES.insert(
        id,
        DeadlineState {
            expires_at: Instant::now() + Duration::from_millis(slice_ms),
        },
    );
    Ok(Value::Ok(Box::new(make_handle(id))))
}

fn bi_extend(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = deadline_id(&args[0], span)?;
    let ms = extract_ms(&args[1], "deadline.extend", span)?;
    let mut dl = DEADLINES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("deadline: not found", span))?;
    dl.expires_at += Duration::from_millis(ms);
    Ok(Value::Ok(Box::new(make_handle(id))))
}
