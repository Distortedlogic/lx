use std::path::{Path, PathBuf};
use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

pub(super) fn storage_dir(base: &str, workflow_name: &str, run_id: &str) -> PathBuf {
    PathBuf::from(base).join(workflow_name).join(run_id)
}

pub(super) fn ensure_dirs(dir: &Path, span: Span) -> Result<(), LxError> {
    std::fs::create_dir_all(dir.join("steps"))
        .map_err(|e| LxError::runtime(format!("durable: mkdir steps: {e}"), span))?;
    std::fs::create_dir_all(dir.join("signals"))
        .map_err(|e| LxError::runtime(format!("durable: mkdir signals: {e}"), span))?;
    Ok(())
}

pub(super) fn save_state(
    dir: &Path,
    status: &str,
    current_step: &str,
    started_at: &str,
    completed_steps: &[String],
    span: Span,
) -> Result<(), LxError> {
    let steps_val: Vec<Value> = completed_steps
        .iter()
        .map(|s| Value::Str(Arc::from(s.as_str())))
        .collect();
    let mut fields = IndexMap::new();
    fields.insert("status".into(), Value::Str(Arc::from(status)));
    fields.insert("current_step".into(), Value::Str(Arc::from(current_step)));
    fields.insert("started_at".into(), Value::Str(Arc::from(started_at)));
    fields.insert("completed_steps".into(), Value::List(Arc::new(steps_val)));
    let val = Value::Record(Arc::new(fields));
    let jv = json_conv::lx_to_json(&val, span)?;
    let json_str = serde_json::to_string_pretty(&jv)
        .map_err(|e| LxError::runtime(format!("durable: serialize state: {e}"), span))?;
    let tmp = dir.join("state.json.tmp");
    std::fs::write(&tmp, &json_str)
        .map_err(|e| LxError::runtime(format!("durable: write state tmp: {e}"), span))?;
    std::fs::rename(&tmp, dir.join("state.json"))
        .map_err(|e| LxError::runtime(format!("durable: rename state: {e}"), span))?;
    Ok(())
}

pub(super) struct LoadedState {
    pub status: String,
    pub started_at: String,
    pub completed_steps: Vec<String>,
}

pub(super) fn load_state(dir: &Path) -> Option<LoadedState> {
    let path = dir.join("state.json");
    let content = std::fs::read_to_string(&path).ok()?;
    let jv: serde_json::Value = serde_json::from_str(&content).ok()?;
    let val = json_conv::json_to_lx(jv);
    let Value::Record(r) = &val else { return None };
    let status = r.get("status")?.as_str()?.to_string();
    let started_at = r
        .get("started_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let completed_steps = r
        .get("completed_steps")
        .and_then(|v| v.as_list())
        .map(|list| {
            list.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Some(LoadedState {
        status,
        started_at,
        completed_steps,
    })
}

pub(super) fn save_step(
    dir: &Path,
    step_name: &str,
    val: &Value,
    span: Span,
) -> Result<(), LxError> {
    let jv = json_conv::lx_to_json(val, span)?;
    let json_str = serde_json::to_string_pretty(&jv)
        .map_err(|e| LxError::runtime(format!("durable: serialize step: {e}"), span))?;
    let path = dir.join("steps").join(format!("{step_name}.json"));
    std::fs::write(&path, &json_str)
        .map_err(|e| LxError::runtime(format!("durable: write step {step_name}: {e}"), span))?;
    Ok(())
}

pub(super) fn load_step(dir: &Path, step_name: &str) -> Option<Value> {
    let path = dir.join("steps").join(format!("{step_name}.json"));
    let content = std::fs::read_to_string(&path).ok()?;
    let jv: serde_json::Value = serde_json::from_str(&content).ok()?;
    Some(json_conv::json_to_lx(jv))
}

pub(super) fn save_signal(
    dir: &Path,
    signal_name: &str,
    val: &Value,
    span: Span,
) -> Result<(), LxError> {
    let jv = json_conv::lx_to_json(val, span)?;
    let json_str = serde_json::to_string_pretty(&jv)
        .map_err(|e| LxError::runtime(format!("durable: serialize signal: {e}"), span))?;
    let path = dir.join("signals").join(format!("{signal_name}.json"));
    std::fs::write(&path, &json_str)
        .map_err(|e| LxError::runtime(format!("durable: write signal: {e}"), span))?;
    Ok(())
}

pub(super) fn load_signal(dir: &Path, signal_name: &str) -> Option<Value> {
    let path = dir.join("signals").join(format!("{signal_name}.json"));
    let content = std::fs::read_to_string(&path).ok()?;
    let jv: serde_json::Value = serde_json::from_str(&content).ok()?;
    Some(json_conv::json_to_lx(jv))
}
