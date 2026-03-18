use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

use super::pipeline_io;

pub(super) struct Pipeline {
    pub name: String,
    pub dir: PathBuf,
    pub stages: Vec<StageInfo>,
}

pub(super) struct StageInfo {
    pub name: String,
    pub status: String,
    pub started: Option<String>,
    pub finished: Option<String>,
    pub error: Option<String>,
}

pub(super) static PIPELINES: LazyLock<DashMap<u64, Pipeline>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("pipeline.create", 2, bi_create));
    m.insert("stage".into(), mk("pipeline.stage", 4, bi_stage));
    m.insert("complete".into(), mk("pipeline.complete", 1, bi_complete));
    m.insert("status".into(), mk("pipeline.status", 1, bi_status));
    m.insert(
        "invalidate".into(),
        mk("pipeline.invalidate", 2, bi_invalidate),
    );
    m.insert(
        "invalidate_from".into(),
        mk("pipeline.invalidate_from", 2, bi_invalidate),
    );
    m.insert("clean".into(), mk("pipeline.clean", 1, bi_clean));
    m.insert("list".into(), mk("pipeline.list", 1, bi_list));
    m
}

fn pipe_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__pipeline_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("pipeline: expected Pipeline handle", span)),
        _ => Err(LxError::type_err(
            "pipeline: expected Pipeline Record",
            span,
        )),
    }
}

fn make_handle(id: u64, name: &str) -> Value {
    Value::Ok(Box::new(record! {
        "__pipeline_id" => Value::Int(BigInt::from(id)),
        "name" => Value::Str(Arc::from(name)),
    }))
}

fn now_str() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn hash_input(input: &Value, span: Span) -> Result<String, LxError> {
    let jv = json_conv::lx_to_json(input, span)?;
    let json_str = serde_json::to_string(&jv)
        .map_err(|e| LxError::runtime(format!("pipeline: hash serialize: {e}"), span))?;
    let mut hasher = DefaultHasher::new();
    json_str.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
}

fn save(p: &Pipeline, span: Span) -> Result<(), LxError> {
    pipeline_io::save_meta(&p.name, &p.dir, &p.stages, span)
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("pipeline.create: name must be Str", span))?;
    let storage = match &args[1] {
        Value::Record(r) => r
            .get("storage")
            .and_then(|v| v.as_str())
            .unwrap_or(".lx/pipelines/")
            .to_string(),
        _ => ".lx/pipelines/".to_string(),
    };
    let dir = PathBuf::from(&storage).join(name);
    std::fs::create_dir_all(&dir)
        .map_err(|e| LxError::runtime(format!("pipeline.create: mkdir: {e}"), span))?;
    let stages = pipeline_io::load_meta(&dir, span)?;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    PIPELINES.insert(
        id,
        Pipeline {
            name: name.to_string(),
            dir,
            stages,
        },
    );
    Ok(make_handle(id, name))
}

fn bi_stage(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = pipe_id(&args[0], span)?;
    let stage_name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("pipeline.stage: name must be Str", span))?;
    let input = &args[2];
    let body = &args[3];
    let input_hash = hash_input(input, span)?;
    let (dir, cached) = {
        let p = PIPELINES
            .get(&pid)
            .ok_or_else(|| LxError::runtime("pipeline.stage: pipeline not found", span))?;
        let hash_file = p.dir.join(format!("{stage_name}.hash"));
        let data_file = p.dir.join(format!("{stage_name}.json"));
        let hit = hash_file.exists()
            && data_file.exists()
            && std::fs::read_to_string(&hash_file)
                .map(|h| h.trim() == input_hash)
                .unwrap_or(false);
        (p.dir.clone(), hit)
    };
    if cached {
        let data_file = dir.join(format!("{stage_name}.json"));
        let content = std::fs::read_to_string(&data_file)
            .map_err(|e| LxError::runtime(format!("pipeline.stage: read cache: {e}"), span))?;
        let jv: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| LxError::runtime(format!("pipeline.stage: parse cache: {e}"), span))?;
        return Ok(Value::Ok(Box::new(json_conv::json_to_lx(jv))));
    }
    let started = now_str();
    let result = call_value(body, input.clone(), span, ctx);
    match result {
        Ok(Value::Err(inner)) => {
            let msg = format!("{}", *inner);
            if let Some(mut p) = PIPELINES.get_mut(&pid) {
                p.stages.push(StageInfo {
                    name: stage_name.to_string(),
                    status: "failed".to_string(),
                    started: Some(started),
                    finished: None,
                    error: Some(msg),
                });
                save(&p, span)?;
            }
            Ok(Value::Err(inner))
        }
        Ok(val) => {
            let jv = json_conv::lx_to_json(&val, span)?;
            let json_str = serde_json::to_string_pretty(&jv)
                .map_err(|e| LxError::runtime(format!("pipeline.stage: serialize: {e}"), span))?;
            std::fs::write(dir.join(format!("{stage_name}.json")), &json_str)
                .map_err(|e| LxError::runtime(format!("pipeline.stage: write: {e}"), span))?;
            std::fs::write(dir.join(format!("{stage_name}.hash")), &input_hash)
                .map_err(|e| LxError::runtime(format!("pipeline.stage: write hash: {e}"), span))?;
            let finished = now_str();
            if let Some(mut p) = PIPELINES.get_mut(&pid) {
                p.stages.push(StageInfo {
                    name: stage_name.to_string(),
                    status: "complete".to_string(),
                    started: Some(started),
                    finished: Some(finished),
                    error: None,
                });
                save(&p, span)?;
            }
            Ok(Value::Ok(Box::new(val)))
        }
        Err(e) => {
            if let Some(mut p) = PIPELINES.get_mut(&pid) {
                p.stages.push(StageInfo {
                    name: stage_name.to_string(),
                    status: "failed".to_string(),
                    started: Some(started),
                    finished: None,
                    error: Some(format!("{e}")),
                });
                save(&p, span)?;
            }
            Ok(Value::Err(Box::new(Value::Str(Arc::from(
                format!("{e}").as_str(),
            )))))
        }
    }
}

fn bi_complete(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = pipe_id(&args[0], span)?;
    let mut p = PIPELINES
        .get_mut(&pid)
        .ok_or_else(|| LxError::runtime("pipeline.complete: pipeline not found", span))?;
    save(&p, span)?;
    let name = p.name.clone();
    p.stages.retain(|s| s.status != "failed");
    Ok(Value::Ok(Box::new(Value::Str(Arc::from(name.as_str())))))
}

fn bi_status(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = pipe_id(&args[0], span)?;
    let p = PIPELINES
        .get(&pid)
        .ok_or_else(|| LxError::runtime("pipeline.status: pipeline not found", span))?;
    let stages: Vec<Value> = p
        .stages
        .iter()
        .map(|s| {
            record! {
                "name" => Value::Str(Arc::from(s.name.as_str())),
                "status" => Value::Str(Arc::from(s.status.as_str())),
            }
        })
        .collect();
    Ok(record! {
        "name" => Value::Str(Arc::from(p.name.as_str())),
        "stages" => Value::List(Arc::new(stages)),
    })
}

fn bi_invalidate(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = pipe_id(&args[0], span)?;
    let stage_name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("pipeline.invalidate: stage must be Str", span))?;
    let mut p = PIPELINES
        .get_mut(&pid)
        .ok_or_else(|| LxError::runtime("pipeline.invalidate: pipeline not found", span))?;
    let idx = p.stages.iter().position(|s| s.name == stage_name);
    if let Some(i) = idx {
        let names: Vec<String> = p.stages[i..].iter().map(|s| s.name.clone()).collect();
        for name in &names {
            let _ = std::fs::remove_file(p.dir.join(format!("{name}.json")));
            let _ = std::fs::remove_file(p.dir.join(format!("{name}.hash")));
        }
        p.stages.truncate(i);
        save(&p, span)?;
    }
    Ok(Value::Unit)
}

fn bi_clean(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = pipe_id(&args[0], span)?;
    let mut p = PIPELINES
        .get_mut(&pid)
        .ok_or_else(|| LxError::runtime("pipeline.clean: pipeline not found", span))?;
    if p.dir.exists() {
        std::fs::remove_dir_all(&p.dir)
            .map_err(|e| LxError::runtime(format!("pipeline.clean: {e}"), span))?;
    }
    p.stages.clear();
    Ok(Value::Unit)
}

fn bi_list(_args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items: Vec<Value> = PIPELINES
        .iter()
        .map(|entry| {
            let p = entry.value();
            record! {
                "name" => Value::Str(Arc::from(p.name.as_str())),
                "stages" => Value::Int(BigInt::from(p.stages.len())),
                "status" => Value::Str(Arc::from(
                    if p.stages.iter().any(|s| s.status == "failed") { "failed" }
                    else if p.stages.is_empty() { "empty" }
                    else { "in_progress" }
                )),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(items)))
}
