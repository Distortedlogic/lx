use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct CronJob {
    cancel: Arc<AtomicBool>,
    handle: thread::JoinHandle<()>,
}

static JOBS: std::sync::LazyLock<DashMap<u64, CronJob>> =
    std::sync::LazyLock::new(DashMap::new);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("every".into(), mk("cron.every", 2, bi_every));
    m.insert("cancel".into(), mk("cron.cancel", 1, bi_cancel));
    m.insert("run".into(), mk("cron.run", 1, bi_run));
    m.insert("active".into(), mk("cron.active", 1, bi_active));
    m
}

fn bi_every(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let interval_ms = match &args[0] {
        Value::Int(n) => {
            let v: i64 = n.try_into()
                .map_err(|_| LxError::type_err("cron.every: interval too large", span))?;
            if v <= 0 {
                return Err(LxError::type_err("cron.every: interval must be positive", span));
            }
            v as u64
        }
        Value::Float(f) => {
            if *f <= 0.0 {
                return Err(LxError::type_err("cron.every: interval must be positive", span));
            }
            *f as u64
        }
        _ => return Err(LxError::type_err("cron.every: first arg must be Int ms", span)),
    };

    let callback = args[1].clone();
    match &callback {
        Value::Func(_) | Value::BuiltinFunc(_) => {}
        _ => return Err(LxError::type_err("cron.every: second arg must be a function", span)),
    }

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_flag = cancel.clone();

    let ctx = Arc::clone(ctx);
    let handle = thread::spawn(move || {
        while !cancel_flag.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(interval_ms));
            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }
            if let Err(e) = call_value(&callback, Value::Unit, span, &ctx) {
                eprintln!("[cron] job {id} error: {e}");
            }
        }
    });

    JOBS.insert(id, CronJob { cancel, handle });
    Ok(Value::Int(BigInt::from(id)))
}

fn bi_cancel(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = args[0].as_int()
        .ok_or_else(|| LxError::type_err("cron.cancel expects Int handle", span))?;
    let id: u64 = id.try_into()
        .map_err(|_| LxError::type_err("cron.cancel: invalid handle", span))?;

    match JOBS.remove(&id) {
        Some((_, job)) => {
            job.cancel.store(true, Ordering::Relaxed);
            let _ = job.handle.join();
            Ok(Value::Unit)
        }
        None => Ok(Value::Err(Box::new(Value::Str(
            Arc::from(format!("cron.cancel: no job with id {id}").as_str()),
        )))),
    }
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

fn bi_active(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    Ok(Value::Int(BigInt::from(JOBS.len())))
}
