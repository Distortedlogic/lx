use std::sync::Arc;

use num_traits::ToPrimitive;

use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::LxVal;
use miette::SourceSpan;

pub(super) fn bi_replace(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let old = args[0].require_str("replace", span)?;
  let new = args[1].require_str("replace", span)?;
  let s = args[2].require_str("replace", span)?;
  Ok(LxVal::str(s.replacen(old, new, 1)))
}

pub(super) fn bi_replace_all(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let old = args[0].require_str("replace_all", span)?;
  let new = args[1].require_str("replace_all", span)?;
  let s = args[2].require_str("replace_all", span)?;
  Ok(LxVal::str(s.replace(old, new)))
}

pub(super) fn bi_repeat(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let n = args[0].require_int("repeat", span)?;
  let s = args[1].require_str("repeat", span)?;
  let count = n.to_usize().ok_or_else(|| LxError::runtime("repeat: count out of range", span))?;
  Ok(LxVal::str(s.repeat(count)))
}

pub(super) fn bi_starts(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let prefix = args[0].require_str("starts?", span)?;
  let s = args[1].require_str("starts?", span)?;
  Ok(LxVal::Bool(s.starts_with(prefix)))
}

pub(super) fn bi_ends(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let suffix = args[0].require_str("ends?", span)?;
  let s = args[1].require_str("ends?", span)?;
  Ok(LxVal::Bool(s.ends_with(suffix)))
}

fn pad(args: &[LxVal], span: SourceSpan, name: &str, left: bool) -> Result<LxVal, LxError> {
  let width = args[0].require_int(name, span)?.to_usize().ok_or_else(|| LxError::runtime(format!("{name}: width out of range"), span))?;
  let s = args[1].require_str(name, span)?;
  let char_count = s.chars().count();
  if char_count >= width {
    Ok(LxVal::str(s))
  } else {
    let padding = " ".repeat(width - char_count);
    let result = if left { format!("{padding}{s}") } else { format!("{s}{padding}") };
    Ok(LxVal::str(result))
  }
}

pub(super) fn bi_pad_left(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  pad(args, span, "pad_left", true)
}

pub(super) fn bi_pad_right(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  pad(args, span, "pad_right", false)
}
