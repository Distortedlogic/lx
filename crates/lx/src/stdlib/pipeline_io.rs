use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

use super::pipeline::StageInfo;

pub(super) fn save_meta(
    name: &str,
    dir: &std::path::Path,
    stages: &[StageInfo],
    span: Span,
) -> Result<(), LxError> {
    let stage_vals: Vec<Value> = stages
        .iter()
        .map(|s| {
            let mut r = IndexMap::new();
            r.insert("name".into(), Value::Str(Arc::from(s.name.as_str())));
            r.insert("status".into(), Value::Str(Arc::from(s.status.as_str())));
            if let Some(ref t) = s.started {
                r.insert("started".into(), Value::Str(Arc::from(t.as_str())));
            }
            if let Some(ref t) = s.finished {
                r.insert("finished".into(), Value::Str(Arc::from(t.as_str())));
            }
            if let Some(ref e) = s.error {
                r.insert("error".into(), Value::Str(Arc::from(e.as_str())));
            }
            Value::Record(Arc::new(r))
        })
        .collect();
    let meta = record! {
        "name" => Value::Str(Arc::from(name)),
        "stages" => Value::List(Arc::new(stage_vals)),
    };
    let jv = json_conv::lx_to_json(&meta, span)?;
    let s = serde_json::to_string_pretty(&jv)
        .map_err(|e| LxError::runtime(format!("pipeline: meta serialize: {e}"), span))?;
    std::fs::write(dir.join("meta.json"), s)
        .map_err(|e| LxError::runtime(format!("pipeline: write meta: {e}"), span))
}

pub(super) fn load_meta(dir: &std::path::Path, span: Span) -> Result<Vec<StageInfo>, LxError> {
    let meta_path = dir.join("meta.json");
    if !meta_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&meta_path)
        .map_err(|e| LxError::runtime(format!("pipeline: read meta: {e}"), span))?;
    let jv: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| LxError::runtime(format!("pipeline: parse meta: {e}"), span))?;
    let val = json_conv::json_to_lx(jv);
    let Value::Record(r) = &val else {
        return Err(LxError::runtime(
            "pipeline: meta.json must be a Record",
            span,
        ));
    };
    let Some(Value::List(stages_list)) = r.get("stages") else {
        return Ok(Vec::new());
    };
    let mut stages = Vec::new();
    for sv in stages_list.iter() {
        let Value::Record(sr) = sv else { continue };
        stages.push(StageInfo {
            name: sr
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            status: sr
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("pending")
                .to_string(),
            started: sr
                .get("started")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            finished: sr
                .get("finished")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            error: sr
                .get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        });
    }
    Ok(stages)
}
