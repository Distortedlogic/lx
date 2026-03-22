use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use chrono::{DateTime, Local, Utc};
use cron::Schedule;
use dashmap::DashMap;

use crate::builtins::call_value;
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

pub(super) static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub(super) struct CronJob {
  pub(super) cancel: Arc<AtomicBool>,
}

pub(super) static JOBS: std::sync::LazyLock<DashMap<u64, CronJob>> = std::sync::LazyLock::new(DashMap::new);

pub(super) fn normalize_cron(expr: &str) -> String {
  let fields: Vec<&str> = expr.split_whitespace().collect();
  match fields.len() {
    5 => format!("0 {expr} *"),
    6 => format!("{expr} *"),
    _ => expr.to_string(),
  }
}

pub(super) fn parse_schedule(expr: &str, span: SourceSpan) -> Result<Schedule, LxError> {
  let normalized = normalize_cron(expr);
  Schedule::from_str(&normalized).map_err(|e| LxError::runtime(format!("cron: invalid expression '{expr}': {e}"), span))
}

pub(super) fn positive_ms(val: &LxVal, name: &str, span: SourceSpan) -> Result<u64, LxError> {
  match val {
    LxVal::Int(n) => {
      let v: i64 = n.try_into().map_err(|_| LxError::type_err(format!("{name}: value too large"), span))?;
      if v <= 0 {
        return Err(LxError::type_err(format!("{name}: must be positive"), span));
      }
      Ok(v as u64)
    },
    LxVal::Float(f) => {
      if *f <= 0.0 {
        return Err(LxError::type_err(format!("{name}: must be positive"), span));
      }
      Ok(*f as u64)
    },
    _ => Err(LxError::type_err(format!("{name}: expected Int or Float ms"), span)),
  }
}

pub(super) fn require_fn(val: &LxVal, name: &str, span: SourceSpan) -> Result<(), LxError> {
  match val {
    LxVal::Func(_) | LxVal::BuiltinFunc(_) => Ok(()),
    _ => Err(LxError::type_err(format!("{name}: expected a function"), span)),
  }
}

pub(super) fn sleep_cancellable(dur: Duration, cancel: &AtomicBool) -> bool {
  let mut remaining = dur;
  while remaining > Duration::ZERO && !cancel.load(Ordering::Relaxed) {
    let chunk = remaining.min(Duration::from_millis(250));
    std::thread::sleep(chunk);
    remaining = remaining.saturating_sub(chunk);
  }
  cancel.load(Ordering::Relaxed)
}

pub(super) fn spawn_oneshot(dur: Duration, callback: LxVal, span: SourceSpan, ctx: Arc<RuntimeCtx>, label: &'static str) -> LxVal {
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  let cancel = Arc::new(AtomicBool::new(false));
  let flag = cancel.clone();
  JOBS.insert(id, CronJob { cancel });
  tokio::task::spawn_blocking(move || {
    if !sleep_cancellable(dur, &flag)
      && let Err(e) = tokio::runtime::Handle::current().block_on(call_value(&callback, LxVal::Unit, span, &ctx))
    {
      eprintln!("[cron] {label} error: {e}");
    }
    JOBS.remove(&id);
  });
  LxVal::int(id)
}

pub(super) fn dt_to_record(dt: DateTime<Utc>) -> LxVal {
  let local: DateTime<Local> = dt.with_timezone(&Local);
  record! {
      "epoch" => LxVal::int(dt.timestamp()),
      "ms" => LxVal::int(dt.timestamp_millis()),
      "iso" => LxVal::str(dt.to_rfc3339()),
      "local" => LxVal::str(local.to_rfc3339()),
  }
}
