use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Local, Utc};
use cron::Schedule;
use dashmap::DashMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub(super) static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub(super) struct CronJob {
    pub(super) cancel: Arc<AtomicBool>,
}

pub(super) static JOBS: std::sync::LazyLock<DashMap<u64, CronJob>> =
    std::sync::LazyLock::new(DashMap::new);

pub(super) fn normalize_cron(expr: &str) -> String {
    let fields: Vec<&str> = expr.split_whitespace().collect();
    match fields.len() {
        5 => format!("0 {expr} *"),
        6 => format!("{expr} *"),
        _ => expr.to_string(),
    }
}

pub(super) fn parse_schedule(expr: &str, span: Span) -> Result<Schedule, LxError> {
    let normalized = normalize_cron(expr);
    Schedule::from_str(&normalized)
        .map_err(|e| LxError::runtime(format!("cron: invalid expression '{expr}': {e}"), span))
}

pub(super) fn positive_ms(val: &Value, name: &str, span: Span) -> Result<u64, LxError> {
    match val {
        Value::Int(n) => {
            let v: i64 = n
                .try_into()
                .map_err(|_| LxError::type_err(format!("{name}: value too large"), span))?;
            if v <= 0 {
                return Err(LxError::type_err(format!("{name}: must be positive"), span));
            }
            Ok(v as u64)
        }
        Value::Float(f) => {
            if *f <= 0.0 {
                return Err(LxError::type_err(format!("{name}: must be positive"), span));
            }
            Ok(*f as u64)
        }
        _ => Err(LxError::type_err(
            format!("{name}: expected Int or Float ms"),
            span,
        )),
    }
}

pub(super) fn require_fn(val: &Value, name: &str, span: Span) -> Result<(), LxError> {
    match val {
        Value::Func(_) | Value::BuiltinFunc(_) => Ok(()),
        _ => Err(LxError::type_err(
            format!("{name}: expected a function"),
            span,
        )),
    }
}

pub(super) fn sleep_cancellable(dur: Duration, cancel: &AtomicBool) -> bool {
    let mut remaining = dur;
    while remaining > Duration::ZERO && !cancel.load(Ordering::Relaxed) {
        let chunk = remaining.min(Duration::from_millis(250));
        thread::sleep(chunk);
        remaining = remaining.saturating_sub(chunk);
    }
    cancel.load(Ordering::Relaxed)
}

pub(super) fn spawn_oneshot(
    dur: Duration,
    callback: Value,
    span: Span,
    ctx: Arc<RuntimeCtx>,
    label: &'static str,
) -> Value {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    JOBS.insert(id, CronJob { cancel });
    thread::spawn(move || {
        if !sleep_cancellable(dur, &flag)
            && let Err(e) =
                ctx.tokio_runtime
                    .block_on(call_value(&callback, Value::Unit, span, &ctx))
        {
            eprintln!("[cron] {label} error: {e}");
        }
        JOBS.remove(&id);
    });
    Value::Int(BigInt::from(id))
}

pub(super) fn dt_to_record(dt: DateTime<Utc>) -> Value {
    let local: DateTime<Local> = dt.with_timezone(&Local);
    record! {
        "epoch" => Value::Int(BigInt::from(dt.timestamp())),
        "ms" => Value::Int(BigInt::from(dt.timestamp_millis())),
        "iso" => Value::Str(Arc::from(dt.to_rfc3339().as_str())),
        "local" => Value::Str(Arc::from(local.to_rfc3339().as_str())),
    }
}
