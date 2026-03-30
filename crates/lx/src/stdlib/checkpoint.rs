use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use indexmap::IndexMap;

use crate::BuiltinCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::std_module;
use crate::value::{BuiltinFunc, BuiltinKind, LxVal};
use miette::SourceSpan;

pub fn build() -> IndexMap<crate::sym::Sym, LxVal> {
  std_module! {
      "scope" => "checkpoint.scope", 3, bi_scope;
      "step"  => "checkpoint.step",  2, bi_step_outside;
      "clear" => "checkpoint.clear", 1, bi_clear
  }
}

fn checkpoint_dir(store_path: &str, scope_name: &str) -> PathBuf {
  PathBuf::from(store_path).join(scope_name)
}

fn write_checkpoint(dir: &Path, step_name: &str, value: &LxVal) -> Result<(), LxError> {
  fs::create_dir_all(dir).map_err(|e| LxError::runtime(format!("checkpoint: failed to create dir: {e}"), (0, 0).into()))?;
  let json_val = serde_json::Value::from(value);
  let pretty = serde_json::to_string_pretty(&json_val).map_err(|e| LxError::runtime(format!("checkpoint: serialize failed: {e}"), (0, 0).into()))?;
  let target = dir.join(format!("{step_name}.json"));
  let tmp = dir.join(format!(".{step_name}.tmp"));
  fs::write(&tmp, &pretty).map_err(|e| LxError::runtime(format!("checkpoint: write failed: {e}"), (0, 0).into()))?;
  fs::rename(&tmp, &target).map_err(|e| LxError::runtime(format!("checkpoint: rename failed: {e}"), (0, 0).into()))?;
  Ok(())
}

fn read_checkpoint(dir: &Path, step_name: &str) -> Option<LxVal> {
  let path = dir.join(format!("{step_name}.json"));
  let content = fs::read_to_string(path).ok()?;
  let json_val: serde_json::Value = serde_json::from_str(&content).ok()?;
  Some(LxVal::from(json_val))
}

fn clear_checkpoints(dir: &Path) -> Result<(), LxError> {
  if dir.exists() {
    fs::remove_dir_all(dir).map_err(|e| LxError::runtime(format!("checkpoint.clear: {e}"), (0, 0).into()))?;
  }
  Ok(())
}

fn contains_func(val: &LxVal) -> bool {
  match val {
    LxVal::Func(_) | LxVal::BuiltinFunc(_) | LxVal::MultiFunc(_) => true,
    LxVal::List(items) => items.iter().any(contains_func),
    LxVal::Tuple(items) => items.iter().any(contains_func),
    LxVal::Record(fields) => fields.values().any(contains_func),
    LxVal::Ok(inner) | LxVal::Err(inner) | LxVal::Some(inner) => contains_func(inner),
    LxVal::Tagged { values, .. } => values.iter().any(contains_func),
    _ => false,
  }
}

fn bi_clear(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let name = args[0].require_str("checkpoint.clear", span)?;
  let dir = checkpoint_dir(".lx-checkpoints", name);
  clear_checkpoints(&dir)?;
  Ok(LxVal::Unit)
}

fn bi_step_outside(_args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  Err(LxError::runtime("checkpoint.step: must be called inside checkpoint.scope", span))
}

fn is_callable(val: &LxVal) -> bool {
  matches!(val, LxVal::Func(_) | LxVal::BuiltinFunc(_) | LxVal::MultiFunc(_))
}

fn bi_step(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let dir_str = args[0].require_str("checkpoint.step", span)?;
  let step_name = args[1].require_str("checkpoint.step", span)?;
  let body = &args[2];
  let dir = PathBuf::from(dir_str);
  if let Some(cached) = read_checkpoint(&dir, step_name) {
    return Ok(cached);
  }
  let result = if is_callable(body) { call_value_sync(body, LxVal::Unit, span, ctx)? } else { body.clone() };
  if contains_func(&result) {
    return Ok(LxVal::err_str("checkpoint.step: cannot checkpoint a value containing Func"));
  }
  write_checkpoint(&dir, step_name, &result)?;
  Ok(result)
}

fn bi_scope(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let name = args[0].require_str("checkpoint.scope", span)?;
  let opts = &args[1];
  let body = &args[2];
  let store_path = opts.str_field("store_path").unwrap_or(".lx-checkpoints");
  let dir = checkpoint_dir(store_path, name);
  let dir_str = dir.to_string_lossy().to_string();
  let step_fn = LxVal::BuiltinFunc(BuiltinFunc { name: "checkpoint.step", arity: 3, kind: BuiltinKind::Sync(bi_step), applied: vec![LxVal::str(dir_str)] });
  call_value_sync(body, step_fn, span, ctx)
}
