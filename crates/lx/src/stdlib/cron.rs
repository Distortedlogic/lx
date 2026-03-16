use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Local, Utc};
use cron::Schedule;
use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct CronJob {
    cancel: Arc<AtomicBool>,
}

static JOBS: std::sync::LazyLock<DashMap<u64, CronJob>> = std::sync::LazyLock::new(DashMap::new);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("schedule".into(), mk("cron.schedule", 2, bi_schedule));
    m.insert("every".into(), mk("cron.every", 2, bi_every));
    m.insert("after".into(), mk("cron.after", 2, bi_after));
    m.insert("at".into(), mk("cron.at", 2, bi_at));
    m.insert("cancel".into(), mk("cron.cancel", 1, bi_cancel));
    m.insert("next".into(), mk("cron.next", 1, bi_next));
    m.insert("next_n".into(), mk("cron.next_n", 2, bi_next_n));
    m.insert("list".into(), mk("cron.list", 1, bi_list));
    m.insert("active".into(), mk("cron.active", 1, bi_active));
    m.insert("run".into(), mk("cron.run", 1, bi_run));
    m
}

fn normalize_cron(expr: &str) -> String {
    let fields: Vec<&str> = expr.split_whitespace().collect();
    match fields.len() {
        5 => format!("0 {expr} *"),
        6 => format!("{expr} *"),
        _ => expr.to_string(),
    }
}

fn parse_schedule(expr: &str, span: Span) -> Result<Schedule, LxError> {
    let normalized = normalize_cron(expr);
    Schedule::from_str(&normalized)
        .map_err(|e| LxError::runtime(format!("cron: invalid expression '{expr}': {e}"), span))
}

fn positive_ms(val: &Value, name: &str, span: Span) -> Result<u64, LxError> {
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

fn require_fn(val: &Value, name: &str, span: Span) -> Result<(), LxError> {
    match val {
        Value::Func(_) | Value::BuiltinFunc(_) => Ok(()),
        _ => Err(LxError::type_err(
            format!("{name}: expected a function"),
            span,
        )),
    }
}

fn sleep_cancellable(dur: Duration, cancel: &AtomicBool) -> bool {
    let mut remaining = dur;
    while remaining > Duration::ZERO && !cancel.load(Ordering::Relaxed) {
        let chunk = remaining.min(Duration::from_millis(250));
        thread::sleep(chunk);
        remaining = remaining.saturating_sub(chunk);
    }
    cancel.load(Ordering::Relaxed)
}

fn spawn_oneshot(
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
            && let Err(e) = call_value(&callback, Value::Unit, span, &ctx)
        {
            eprintln!("[cron] {label} error: {e}");
        }
        JOBS.remove(&id);
    });
    Value::Int(BigInt::from(id))
}

fn dt_to_record(dt: DateTime<Utc>) -> Value {
    let local: DateTime<Local> = dt.with_timezone(&Local);
    record! {
        "epoch" => Value::Int(BigInt::from(dt.timestamp())),
        "ms" => Value::Int(BigInt::from(dt.timestamp_millis())),
        "iso" => Value::Str(Arc::from(dt.to_rfc3339().as_str())),
        "local" => Value::Str(Arc::from(local.to_rfc3339().as_str())),
    }
}

fn bi_schedule(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let expr = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("cron.schedule: first arg must be Str", span))?;
    let schedule = parse_schedule(expr, span)?;
    require_fn(&args[1], "cron.schedule", span)?;
    let callback = args[1].clone();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    JOBS.insert(id, CronJob { cancel });
    let ctx = Arc::clone(ctx);
    thread::spawn(move || {
        while !flag.load(Ordering::Relaxed) {
            let Some(next_time) = schedule.upcoming(Utc).next() else {
                break;
            };
            let now = Utc::now();
            let wait = (next_time - now).to_std().unwrap_or(Duration::ZERO);
            if sleep_cancellable(wait, &flag) {
                break;
            }
            if let Err(e) = call_value(&callback, Value::Unit, span, &ctx) {
                eprintln!("[cron] schedule error: {e}");
            }
        }
        JOBS.remove(&id);
    });
    Ok(Value::Int(BigInt::from(id)))
}

fn bi_every(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let ms = positive_ms(&args[0], "cron.every", span)?;
    require_fn(&args[1], "cron.every", span)?;
    let callback = args[1].clone();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    JOBS.insert(id, CronJob { cancel });
    let ctx = Arc::clone(ctx);
    thread::spawn(move || {
        while !flag.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(ms));
            if flag.load(Ordering::Relaxed) {
                break;
            }
            if let Err(e) = call_value(&callback, Value::Unit, span, &ctx) {
                eprintln!("[cron] every error: {e}");
            }
        }
        JOBS.remove(&id);
    });
    Ok(Value::Int(BigInt::from(id)))
}

fn bi_after(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let ms = positive_ms(&args[0], "cron.after", span)?;
    require_fn(&args[1], "cron.after", span)?;
    Ok(spawn_oneshot(
        Duration::from_millis(ms),
        args[1].clone(),
        span,
        Arc::clone(ctx),
        "after",
    ))
}

fn bi_at(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let time_str = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("cron.at: first arg must be ISO time Str", span))?;
    let target = DateTime::parse_from_rfc3339(time_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| LxError::runtime(format!("cron.at: invalid time '{time_str}': {e}"), span))?;
    let now = Utc::now();
    if target <= now {
        return Err(LxError::runtime("cron.at: time is in the past", span));
    }
    let wait = (target - now).to_std().unwrap_or(Duration::ZERO);
    require_fn(&args[1], "cron.at", span)?;
    Ok(spawn_oneshot(
        wait,
        args[1].clone(),
        span,
        Arc::clone(ctx),
        "at",
    ))
}

fn bi_cancel(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = args[0]
        .as_int()
        .ok_or_else(|| LxError::type_err("cron.cancel expects Int handle", span))?;
    let id: u64 = id
        .try_into()
        .map_err(|_| LxError::type_err("cron.cancel: invalid handle", span))?;
    match JOBS.remove(&id) {
        Some((_, job)) => {
            job.cancel.store(true, Ordering::Relaxed);
            Ok(Value::Unit)
        }
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("cron.cancel: no job with id {id}").as_str(),
        ))))),
    }
}

fn bi_next(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let expr = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("cron.next: expected Str expression", span))?;
    let schedule = match parse_schedule(expr, span) {
        Ok(s) => s,
        Err(e) => {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                e.to_string().as_str(),
            )))));
        }
    };
    match schedule.upcoming(Utc).next() {
        Some(dt) => Ok(Value::Ok(Box::new(dt_to_record(dt)))),
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "no upcoming occurrence",
        ))))),
    }
}

fn bi_next_n(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let n = match &args[0] {
        Value::Int(n) => {
            let v: i64 = n
                .try_into()
                .map_err(|_| LxError::type_err("cron.next_n: count too large", span))?;
            if v <= 0 {
                return Err(LxError::type_err(
                    "cron.next_n: count must be positive",
                    span,
                ));
            }
            v as usize
        }
        _ => {
            return Err(LxError::type_err(
                "cron.next_n: first arg must be Int",
                span,
            ));
        }
    };
    let expr = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("cron.next_n: second arg must be Str", span))?;
    let schedule = match parse_schedule(expr, span) {
        Ok(s) => s,
        Err(e) => {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                e.to_string().as_str(),
            )))));
        }
    };
    let times: Vec<Value> = schedule.upcoming(Utc).take(n).map(dt_to_record).collect();
    Ok(Value::List(Arc::new(times)))
}

fn bi_list(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    let ids: Vec<Value> = JOBS
        .iter()
        .map(|e| Value::Int(BigInt::from(*e.key())))
        .collect();
    Ok(Value::List(Arc::new(ids)))
}

fn bi_active(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    Ok(Value::Int(BigInt::from(JOBS.len())))
}

fn bi_run(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    loop {
        if JOBS.is_empty() {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }
    Ok(Value::Unit)
}
