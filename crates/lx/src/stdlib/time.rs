use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone, Utc};
use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("now".into(), mk("time.now", 1, bi_now));
  m.insert("sleep".into(), mk("time.sleep", 1, bi_sleep));
  m.insert("format".into(), mk("time.format", 2, bi_format));
  m.insert("parse".into(), mk("time.parse", 2, bi_parse));
  m
}

fn bi_now(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  let now = Utc::now();
  Ok(timestamp_to_record(now))
}

fn bi_sleep(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let ms = match &args[0] {
    LxVal::Int(n) => {
      let v: i64 = n.try_into().map_err(|_| LxError::type_err("time.sleep: ms too large", span))?;
      if v < 0 {
        return Err(LxError::type_err("time.sleep: ms must be non-negative", span));
      }
      v as u64
    },
    LxVal::Float(f) => {
      if *f < 0.0 {
        return Err(LxError::type_err("time.sleep: ms must be non-negative", span));
      }
      *f as u64
    },
    _ => {
      return Err(LxError::type_err("time.sleep expects Int or Float ms", span));
    },
  };
  std::thread::sleep(std::time::Duration::from_millis(ms));
  Ok(LxVal::Unit)
}

fn bi_format(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let fmt = args[0].require_str("time.format", span)?;
  let ts = record_to_datetime(&args[1], span)?;
  let formatted = ts.format(fmt).to_string();
  Ok(LxVal::str(formatted))
}

fn bi_parse(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let fmt = args[0].require_str("time.parse", span)?;
  let input = args[1].require_str("time.parse", span)?;
  match DateTime::parse_from_str(input, fmt) {
    Ok(dt) => Ok(LxVal::ok(timestamp_to_record(dt.with_timezone(&Utc)))),
    Err(e) => Ok(LxVal::err_str(format!("time.parse: {e}"))),
  }
}

fn timestamp_to_record(dt: DateTime<Utc>) -> LxVal {
  let local: DateTime<Local> = dt.with_timezone(&Local);
  record! {
      "epoch" => LxVal::int(dt.timestamp()),
      "ms" => LxVal::int(dt.timestamp_millis()),
      "iso" => LxVal::str(dt.to_rfc3339()),
      "local" => LxVal::str(local.to_rfc3339()),
  }
}

fn record_to_datetime(val: &LxVal, span: SourceSpan) -> Result<DateTime<Utc>, LxError> {
  match val {
    LxVal::Record(fields) => {
      if let Some(LxVal::Int(epoch)) = fields.get("epoch") {
        let secs: i64 = epoch.try_into().map_err(|_| LxError::type_err("time: epoch too large", span))?;
        return Utc.timestamp_opt(secs, 0).single().ok_or_else(|| LxError::runtime("time: invalid epoch", span));
      }
      if let Some(LxVal::Int(ms)) = fields.get("ms") {
        let millis: i64 = ms.try_into().map_err(|_| LxError::type_err("time: ms too large", span))?;
        return Utc.timestamp_millis_opt(millis).single().ok_or_else(|| LxError::runtime("time: invalid ms", span));
      }
      if let Some(LxVal::Str(iso)) = fields.get("iso") {
        return DateTime::parse_from_rfc3339(iso).map(|dt| dt.with_timezone(&Utc)).map_err(|e| LxError::runtime(format!("time: bad iso: {e}"), span));
      }
      Err(LxError::type_err("time: record needs epoch, ms, or iso field", span))
    },
    _ => Err(LxError::type_err("time: expected timestamp Record", span)),
  }
}
