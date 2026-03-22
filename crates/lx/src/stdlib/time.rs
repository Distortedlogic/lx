use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use indexmap::IndexMap;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::std_module;
use crate::stdlib::helpers::datetime_to_record;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<crate::sym::Sym, LxVal> {
  std_module! {
    "now"    => "time.now",    1, bi_now;
    "sleep"  => "time.sleep",  1, bi_sleep;
    "format" => "time.format", 2, bi_format;
    "parse"  => "time.parse",  2, bi_parse
  }
}

fn bi_now(args: &[LxVal], _span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _ = &args[0];
  Ok(datetime_to_record(Utc::now()))
}

fn bi_sleep(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let ms = match &args[0] {
    LxVal::Int(n) => {
      let v: i64 = n.try_into().map_err(|_| LxError::type_err("time.sleep: ms too large", span, None))?;
      if v < 0 {
        return Err(LxError::type_err("time.sleep: ms must be non-negative", span, None));
      }
      v as u64
    },
    LxVal::Float(f) => {
      if *f < 0.0 {
        return Err(LxError::type_err("time.sleep: ms must be non-negative", span, None));
      }
      *f as u64
    },
    _ => {
      return Err(LxError::type_err("time.sleep expects Int or Float ms", span, None));
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
    Ok(dt) => Ok(LxVal::ok(datetime_to_record(dt.with_timezone(&Utc)))),
    Err(e) => Ok(LxVal::err_str(format!("time.parse: {e}"))),
  }
}

fn record_to_datetime(val: &LxVal, span: SourceSpan) -> Result<DateTime<Utc>, LxError> {
  match val {
    LxVal::Record(fields) => {
      if let Some(LxVal::Int(epoch)) = fields.get(&crate::sym::intern("epoch")) {
        let secs: i64 = epoch.try_into().map_err(|_| LxError::type_err("time: epoch too large", span, None))?;
        return Utc.timestamp_opt(secs, 0).single().ok_or_else(|| LxError::runtime("time: invalid epoch", span));
      }
      if let Some(LxVal::Int(ms)) = fields.get(&crate::sym::intern("ms")) {
        let millis: i64 = ms.try_into().map_err(|_| LxError::type_err("time: ms too large", span, None))?;
        return Utc.timestamp_millis_opt(millis).single().ok_or_else(|| LxError::runtime("time: invalid ms", span));
      }
      if let Some(LxVal::Str(iso)) = fields.get(&crate::sym::intern("iso")) {
        return DateTime::parse_from_rfc3339(iso).map(|dt| dt.with_timezone(&Utc)).map_err(|e| LxError::runtime(format!("time: bad iso: {e}"), span));
      }
      Err(LxError::type_err("time: record needs epoch, ms, or iso field", span, None))
    },
    _ => Err(LxError::type_err("time: expected timestamp Record", span, None)),
  }
}
