use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::call_value_sync;
use crate::builtins::mk;
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::LxVal;

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("confirm".into(), mk("user.confirm", 1, bi_confirm));
  m.insert("choose".into(), mk("user.choose", 2, bi_choose));
  m.insert("ask".into(), mk("user.ask", 1, bi_ask));
  m.insert("ask_with".into(), mk("user.ask_with", 2, bi_ask_with));
  m.insert("progress".into(), mk("user.progress", 3, bi_progress));
  m.insert("progress_pct".into(), mk("user.progress_pct", 2, bi_progress_pct));
  m.insert("status".into(), mk("user.status", 2, bi_status));
  m.insert("table".into(), mk("user.table", 2, bi_table));
  m.insert("check".into(), mk("user.check", 1, bi_check));
  m
}

fn bi_confirm(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let msg = args[0].as_str().ok_or_else(|| LxError::type_err("user.confirm: expected Str message", span))?;
  ctx.user.confirm(msg).map(LxVal::Bool).map_err(|e| LxError::runtime(format!("user.confirm: {e}"), span))
}

fn bi_choose(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let msg = args[0].as_str().ok_or_else(|| LxError::type_err("user.choose: expected Str message", span))?;
  let LxVal::List(items) = &args[1] else {
    return Err(LxError::type_err("user.choose: second arg must be List", span));
  };
  let labels: Vec<String> = items.iter().map(option_label).collect();
  let idx = ctx.user.choose(msg, &labels).map_err(|e| LxError::runtime(format!("user.choose: {e}"), span))?;
  if idx >= items.len() {
    return Err(LxError::runtime("user.choose: index out of range", span));
  }
  Ok(items[idx].clone())
}

fn option_label(v: &LxVal) -> String {
  match v {
    LxVal::Str(s) => s.to_string(),
    LxVal::Record(fields) => {
      if let Some(label) = fields.get("label").and_then(|v| v.as_str()) {
        if let Some(desc) = fields.get("desc").and_then(|v| v.as_str()) {
          return format!("{label} — {desc}");
        }
        return label.to_string();
      }
      format!("{v}")
    },
    _ => format!("{v}"),
  }
}

fn bi_ask(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let msg = args[0].as_str().ok_or_else(|| LxError::type_err("user.ask: expected Str message", span))?;
  ctx.user.ask(msg, None).map(LxVal::str).map_err(|e| LxError::runtime(format!("user.ask: {e}"), span))
}

fn bi_ask_with(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let msg = args[0].as_str().ok_or_else(|| LxError::type_err("user.ask_with: expected Str message", span))?;
  let LxVal::Record(opts) = &args[1] else {
    return Err(LxError::type_err("user.ask_with: second arg must be Record", span));
  };
  let default = opts.get("default").and_then(|v| v.as_str());
  let validate = opts.get("validate");
  loop {
    let result = ctx.user.ask(msg, default).map_err(|e| LxError::runtime(format!("user.ask_with: {e}"), span))?;
    if let Some(pred) = validate {
      let val = LxVal::str(&result);
      let check = call_value_sync(pred, val, span, ctx)?;
      if check == LxVal::Bool(false) {
        continue;
      }
    }
    return Ok(LxVal::str(result));
  }
}

fn bi_progress(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let current = args[0].as_int().and_then(|n| usize::try_from(n).ok()).ok_or_else(|| LxError::type_err("user.progress: expected Int current", span))?;
  let total = args[1].as_int().and_then(|n| usize::try_from(n).ok()).ok_or_else(|| LxError::type_err("user.progress: expected Int total", span))?;
  let msg = args[2].as_str().ok_or_else(|| LxError::type_err("user.progress: expected Str message", span))?;
  ctx.user.progress(current, total, msg);
  Ok(LxVal::Unit)
}

fn bi_progress_pct(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let pct = match &args[0] {
    LxVal::Float(f) => *f,
    LxVal::Int(n) => {
      use num_traits::ToPrimitive;
      n.to_f64().ok_or_else(|| LxError::type_err("user.progress_pct: Int too large", span))?
    },
    _ => {
      return Err(LxError::type_err("user.progress_pct: expected Float pct", span));
    },
  };
  let msg = args[1].as_str().ok_or_else(|| LxError::type_err("user.progress_pct: expected Str message", span))?;
  ctx.user.progress_pct(pct, msg);
  Ok(LxVal::Unit)
}

fn bi_status(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let level = args[0].as_str().ok_or_else(|| LxError::type_err("user.status: expected Str level", span))?;
  let msg = args[1].as_str().ok_or_else(|| LxError::type_err("user.status: expected Str message", span))?;
  ctx.user.status(level, msg);
  Ok(LxVal::Unit)
}

fn bi_table(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let LxVal::List(hdrs) = &args[0] else {
    return Err(LxError::type_err("user.table: first arg must be List of Str", span));
  };
  let headers: Vec<String> = hdrs.iter().map(|v| format!("{v}")).collect();
  let LxVal::List(row_vals) = &args[1] else {
    return Err(LxError::type_err("user.table: second arg must be List of Lists", span));
  };
  let rows: Vec<Vec<String>> = row_vals
    .iter()
    .map(|row| match row {
      LxVal::List(cells) => cells.iter().map(|c| format!("{c}")).collect(),
      _ => vec![format!("{row}")],
    })
    .collect();
  ctx.user.table(&headers, &rows);
  Ok(LxVal::Unit)
}

fn bi_check(_args: &[LxVal], _span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match ctx.user.check_signal() {
    Some(signal) => Ok(LxVal::Some(Box::new(signal))),
    None => Ok(LxVal::None),
  }
}
