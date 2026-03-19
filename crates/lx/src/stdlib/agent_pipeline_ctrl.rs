use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::agent_pipeline::{PIPELINES, Pipeline, get_pipeline_id, pump};

fn pressure_rank(level: &str) -> u8 {
    match level {
        "critical" => 4,
        "high" => 3,
        "moderate" => 2,
        _ => 1,
    }
}

fn max_pressure(pipeline: &Pipeline) -> &'static str {
    let max_ratio = pipeline
        .stages
        .iter()
        .map(|s| s.buffer.len() as f64 / pipeline.buffer_size as f64)
        .fold(0.0f64, f64::max);
    if max_ratio > 0.9 {
        "critical"
    } else if max_ratio > 0.75 {
        "high"
    } else if max_ratio > 0.5 {
        "moderate"
    } else {
        "low"
    }
}

fn build_stats(pipeline: &Pipeline) -> Value {
    let mut bottleneck_name = String::new();
    let mut bottleneck_queued = 0usize;
    let stages: Vec<Value> = pipeline
        .stages
        .iter()
        .map(|s| {
            let avg_ms = if s.processed > 0 {
                s.total_ms as f64 / s.processed as f64
            } else {
                0.0
            };
            let queued = s.buffer.len();
            if queued > bottleneck_queued {
                bottleneck_queued = queued;
                bottleneck_name = s.name.clone();
            }
            let mut rec = IndexMap::new();
            rec.insert("name".into(), Value::Str(Arc::from(s.name.as_str())));
            rec.insert("queued".into(), Value::Int(BigInt::from(queued)));
            rec.insert("processed".into(), Value::Int(BigInt::from(s.processed)));
            rec.insert("avg_ms".into(), Value::Float(avg_ms));
            Value::Record(Arc::new(rec))
        })
        .collect();
    let throughput = pipeline
        .stages
        .last()
        .filter(|s| s.total_ms > 0)
        .map(|s| s.processed as f64 / (s.total_ms as f64 / 1000.0))
        .unwrap_or(0.0);
    let pressure = max_pressure(pipeline);
    let mut rec = IndexMap::new();
    rec.insert("stages".into(), Value::List(Arc::new(stages)));
    rec.insert(
        "total_processed".into(),
        Value::Int(BigInt::from(pipeline.total_processed)),
    );
    rec.insert(
        "total_dropped".into(),
        Value::Int(BigInt::from(pipeline.total_dropped)),
    );
    rec.insert(
        "bottleneck".into(),
        Value::Str(Arc::from(bottleneck_name.as_str())),
    );
    rec.insert("throughput".into(), Value::Float(throughput));
    rec.insert("pressure".into(), Value::Str(Arc::from(pressure)));
    Value::Record(Arc::new(rec))
}

pub(super) fn fire_pressure_callbacks(
    id: u64,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<(), LxError> {
    let (callbacks, stats) = {
        let Some(pipeline) = PIPELINES.get(&id) else {
            return Ok(());
        };
        if pipeline.pressure_callbacks.is_empty() {
            return Ok(());
        }
        let level = max_pressure(&pipeline);
        let matching: Vec<Value> = pipeline
            .pressure_callbacks
            .iter()
            .filter(|(threshold, _)| pressure_rank(level) >= pressure_rank(threshold))
            .map(|(_, cb)| cb.clone())
            .collect();
        if matching.is_empty() {
            return Ok(());
        }
        (matching, build_stats(&pipeline))
    };
    for cb in callbacks {
        call_value(&cb, stats.clone(), span, ctx)?;
    }
    Ok(())
}

fn bi_pipeline_stats(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    let pipeline = PIPELINES
        .get(&id)
        .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
    Ok(build_stats(&pipeline))
}

fn bi_pipeline_on_pressure(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    let level = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("pipeline_on_pressure: level must be Str/tag", span))?;
    if !matches!(level, "low" | "moderate" | "high" | "critical") {
        return Err(LxError::runtime(
            "pipeline_on_pressure: level must be :low, :moderate, :high, or :critical",
            span,
        ));
    }
    let callback = args[2].clone();
    let mut pipeline = PIPELINES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
    pipeline
        .pressure_callbacks
        .push((level.to_string(), callback));
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_pipeline_pause(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    let mut pipeline = PIPELINES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
    pipeline.paused = true;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_pipeline_resume(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    let mut pipeline = PIPELINES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
    pipeline.paused = false;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_pipeline_drain(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    let mut pipeline = PIPELINES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
    let was_paused = pipeline.paused;
    pipeline.paused = false;
    pump(&mut pipeline, span, ctx)?;
    pipeline.paused = was_paused;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_pipeline_close(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    {
        let mut pipeline = PIPELINES
            .get_mut(&id)
            .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
        pipeline.paused = false;
        pump(&mut pipeline, span, ctx)?;
    }
    PIPELINES.remove(&id);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_pipeline_add_worker(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    let stage_name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("pipeline_add_worker: stage name must be Str", span))?;
    let agent = args[2].clone();
    let mut pipeline = PIPELINES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
    let stage = pipeline
        .stages
        .iter_mut()
        .find(|s| s.name == stage_name)
        .ok_or_else(|| {
            LxError::runtime(format!("pipeline: no stage named '{stage_name}'"), span)
        })?;
    stage.agents.push(agent);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

pub fn mk_pipeline_stats() -> Value {
    mk("agent.pipeline_stats", 1, bi_pipeline_stats)
}

pub fn mk_pipeline_on_pressure() -> Value {
    mk("agent.pipeline_on_pressure", 3, bi_pipeline_on_pressure)
}

pub fn mk_pipeline_pause() -> Value {
    mk("agent.pipeline_pause", 1, bi_pipeline_pause)
}

pub fn mk_pipeline_resume() -> Value {
    mk("agent.pipeline_resume", 1, bi_pipeline_resume)
}

pub fn mk_pipeline_drain() -> Value {
    mk("agent.pipeline_drain", 1, bi_pipeline_drain)
}

pub fn mk_pipeline_close() -> Value {
    mk("agent.pipeline_close", 1, bi_pipeline_close)
}

pub fn mk_pipeline_add_worker() -> Value {
    mk("agent.pipeline_add_worker", 3, bi_pipeline_add_worker)
}
