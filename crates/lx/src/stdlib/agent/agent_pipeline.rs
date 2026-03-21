use std::sync::atomic::AtomicU64;
use std::sync::{Arc, LazyLock};
use std::time::Instant;

use dashmap::DashMap;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub(super) static NEXT_ID: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(1));
pub(super) static PIPELINES: LazyLock<DashMap<u64, Pipeline>> = LazyLock::new(DashMap::new);

#[derive(Clone, Copy)]
pub(super) enum OverflowPolicy {
    Block,
    DropOldest,
    DropNewest,
    Sample,
}

pub(super) struct PipelineStage {
    pub(super) name: String,
    pub(super) agents: Vec<Value>,
    pub(super) next_worker: usize,
    pub(super) buffer: Vec<Value>,
    pub(super) processed: u64,
    pub(super) total_ms: u64,
}

pub(super) struct Pipeline {
    pub(super) stages: Vec<PipelineStage>,
    pub(super) buffer_size: usize,
    pub(super) overflow: OverflowPolicy,
    pub(super) output: Vec<Value>,
    pub(super) paused: bool,
    pub(super) total_processed: u64,
    pub(super) total_dropped: u64,
    pub(super) sample_counter: u64,
    pub(super) pressure_callbacks: Vec<(String, Value)>,
}

pub(super) fn get_pipeline_id(v: &Value, span: Span) -> Result<u64, LxError> {
    if let Value::Record(r) = v
        && let Some(id) = r
            .get("__pipeline_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
    {
        return Ok(id);
    }
    Err(LxError::type_err("expected pipeline handle", span))
}

pub(super) fn ask_pipeline_agent(
    agent: &Value,
    msg: Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    if let Value::Record(r) = agent {
        if let Some(handler) = r
            .get("handler")
            .filter(|h| matches!(h, Value::Func(_) | Value::BuiltinFunc(_)))
        {
            return call_value_sync(handler, msg, span, ctx);
        }
        if let Some(pid) = r
            .get("__pid")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
        {
            return super::agent::ask_subprocess(pid, &msg, span);
        }
    }
    Err(LxError::type_err(
        "pipeline: agent must have handler or __pid",
        span,
    ))
}

pub(super) fn pump(
    pipeline: &mut Pipeline,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<(), LxError> {
    if pipeline.paused {
        return Ok(());
    }
    let num_stages = pipeline.stages.len();
    let mut changed = true;
    while changed {
        changed = false;
        for i in (0..num_stages).rev() {
            if pipeline.stages[i].buffer.is_empty() {
                continue;
            }
            let has_room = if i == num_stages - 1 {
                true
            } else {
                pipeline.stages[i + 1].buffer.len() < pipeline.buffer_size
            };
            if !has_room {
                continue;
            }
            let item = pipeline.stages[i].buffer.remove(0);
            let worker_idx = pipeline.stages[i].next_worker % pipeline.stages[i].agents.len();
            pipeline.stages[i].next_worker += 1;
            let agent = pipeline.stages[i].agents[worker_idx].clone();
            let start = Instant::now();
            let result = ask_pipeline_agent(&agent, item, span, ctx)?;
            let elapsed = start.elapsed().as_millis() as u64;
            pipeline.stages[i].processed += 1;
            pipeline.stages[i].total_ms += elapsed;
            let val = match result {
                Value::Ok(inner) => *inner,
                Value::Err(_) => {
                    pipeline.output.push(result);
                    pipeline.total_processed += 1;
                    changed = true;
                    continue;
                }
                other => other,
            };
            if i == num_stages - 1 {
                pipeline.output.push(val);
                pipeline.total_processed += 1;
            } else {
                pipeline.stages[i + 1].buffer.push(val);
            }
            changed = true;
        }
    }
    Ok(())
}
