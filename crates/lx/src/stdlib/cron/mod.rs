#[path = "cron_helpers.rs"]
mod cron_helpers;
use cron_helpers::{CronJob, JOBS, NEXT_ID, dt_to_record, parse_schedule, positive_ms, require_fn, sleep_cancellable, spawn_oneshot};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use chrono::{DateTime, Utc};
use indexmap::IndexMap;

use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::LxVal;

pub fn build() -> IndexMap<String, LxVal> {
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

fn bi_schedule(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let expr = args[0].as_str().ok_or_else(|| LxError::type_err("cron.schedule: first arg must be Str", span))?;
  let schedule = parse_schedule(expr, span)?;
  require_fn(&args[1], "cron.schedule", span)?;
  let callback = args[1].clone();
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  let cancel = Arc::new(AtomicBool::new(false));
  let flag = cancel.clone();
  JOBS.insert(id, CronJob { cancel });
  let ctx = Arc::clone(ctx);
  tokio::task::spawn_blocking(move || {
    while !flag.load(Ordering::Relaxed) {
      let Some(next_time) = schedule.upcoming(Utc).next() else {
        break;
      };
      let now = Utc::now();
      let wait = (next_time - now).to_std().unwrap_or(Duration::ZERO);
      if sleep_cancellable(wait, &flag) {
        break;
      }
      if let Err(e) = tokio::runtime::Handle::current().block_on(call_value(&callback, LxVal::Unit, span, &ctx)) {
        eprintln!("[cron] schedule error: {e}");
      }
    }
    JOBS.remove(&id);
  });
  Ok(LxVal::int(id))
}

fn bi_every(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let ms = positive_ms(&args[0], "cron.every", span)?;
  require_fn(&args[1], "cron.every", span)?;
  let callback = args[1].clone();
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  let cancel = Arc::new(AtomicBool::new(false));
  let flag = cancel.clone();
  JOBS.insert(id, CronJob { cancel });
  let ctx = Arc::clone(ctx);
  tokio::task::spawn_blocking(move || {
    while !flag.load(Ordering::Relaxed) {
      std::thread::sleep(Duration::from_millis(ms));
      if flag.load(Ordering::Relaxed) {
        break;
      }
      if let Err(e) = tokio::runtime::Handle::current().block_on(call_value(&callback, LxVal::Unit, span, &ctx)) {
        eprintln!("[cron] every error: {e}");
      }
    }
    JOBS.remove(&id);
  });
  Ok(LxVal::int(id))
}

fn bi_after(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let ms = positive_ms(&args[0], "cron.after", span)?;
  require_fn(&args[1], "cron.after", span)?;
  Ok(spawn_oneshot(Duration::from_millis(ms), args[1].clone(), span, Arc::clone(ctx), "after"))
}

fn bi_at(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let time_str = args[0].as_str().ok_or_else(|| LxError::type_err("cron.at: first arg must be ISO time Str", span))?;
  let target = DateTime::parse_from_rfc3339(time_str)
    .map(|dt| dt.with_timezone(&Utc))
    .map_err(|e| LxError::runtime(format!("cron.at: invalid time '{time_str}': {e}"), span))?;
  let now = Utc::now();
  if target <= now {
    return Err(LxError::runtime("cron.at: time is in the past", span));
  }
  let wait = (target - now).to_std().unwrap_or(Duration::ZERO);
  require_fn(&args[1], "cron.at", span)?;
  Ok(spawn_oneshot(wait, args[1].clone(), span, Arc::clone(ctx), "at"))
}

fn bi_cancel(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = args[0].as_int().ok_or_else(|| LxError::type_err("cron.cancel expects Int handle", span))?;
  let id: u64 = id.try_into().map_err(|_| LxError::type_err("cron.cancel: invalid handle", span))?;
  match JOBS.remove(&id) {
    Some((_, job)) => {
      job.cancel.store(true, Ordering::Relaxed);
      Ok(LxVal::Unit)
    },
    None => Ok(LxVal::Err(Box::new(LxVal::str(format!("cron.cancel: no job with id {id}"))))),
  }
}

fn bi_next(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let expr = args[0].as_str().ok_or_else(|| LxError::type_err("cron.next: expected Str expression", span))?;
  let schedule = match parse_schedule(expr, span) {
    Ok(s) => s,
    Err(e) => {
      return Ok(LxVal::Err(Box::new(LxVal::str(e.to_string()))));
    },
  };
  match schedule.upcoming(Utc).next() {
    Some(dt) => Ok(LxVal::Ok(Box::new(dt_to_record(dt)))),
    None => Ok(LxVal::Err(Box::new(LxVal::str("no upcoming occurrence")))),
  }
}

fn bi_next_n(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let n = match &args[0] {
    LxVal::Int(n) => {
      let v: i64 = n.try_into().map_err(|_| LxError::type_err("cron.next_n: count too large", span))?;
      if v <= 0 {
        return Err(LxError::type_err("cron.next_n: count must be positive", span));
      }
      v as usize
    },
    _ => {
      return Err(LxError::type_err("cron.next_n: first arg must be Int", span));
    },
  };
  let expr = args[1].as_str().ok_or_else(|| LxError::type_err("cron.next_n: second arg must be Str", span))?;
  let schedule = match parse_schedule(expr, span) {
    Ok(s) => s,
    Err(e) => {
      return Ok(LxVal::Err(Box::new(LxVal::str(e.to_string()))));
    },
  };
  let times: Vec<LxVal> = schedule.upcoming(Utc).take(n).map(dt_to_record).collect();
  Ok(LxVal::list(times))
}

fn bi_list(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  let ids: Vec<LxVal> = JOBS.iter().map(|e| LxVal::int(*e.key())).collect();
  Ok(LxVal::list(ids))
}

fn bi_active(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  Ok(LxVal::int(JOBS.len()))
}

fn bi_run(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  loop {
    if JOBS.is_empty() {
      break;
    }
    std::thread::sleep(Duration::from_millis(500));
  }
  Ok(LxVal::Unit)
}
