use std::sync::atomic::Ordering;
use std::sync::Arc;

use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::agent_pipeline::{
    get_pipeline_id, pump, OverflowPolicy, Pipeline, PipelineStage, NEXT_ID, PIPELINES,
};

fn agent_name(agent: &Value) -> String {
    if let Value::Record(r) = agent
        && let Some(Value::Str(s)) = r.get("name")
    {
        return s.to_string();
    }
    "unnamed".to_string()
}

fn parse_overflow(v: &Value, span: Span) -> Result<OverflowPolicy, LxError> {
    let s = v
        .as_str()
        .ok_or_else(|| LxError::type_err("overflow must be a tag/Str", span))?;
    match s {
        "block" => Ok(OverflowPolicy::Block),
        "drop_oldest" => Ok(OverflowPolicy::DropOldest),
        "drop_newest" => Ok(OverflowPolicy::DropNewest),
        "sample" => Ok(OverflowPolicy::Sample),
        _ => Err(LxError::runtime(
            "overflow must be :block, :drop_oldest, :drop_newest, or :sample",
            span,
        )),
    }
}

fn send_to_pipeline(
    pipeline: &mut Pipeline,
    msg: Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<(), LxError> {
    pump(pipeline, span, ctx)?;
    let buf_len = pipeline.stages[0].buffer.len();
    if buf_len >= pipeline.buffer_size {
        match pipeline.overflow {
            OverflowPolicy::Block => {
                return Err(LxError::runtime(
                    "pipeline: buffer full after drain",
                    span,
                ));
            }
            OverflowPolicy::DropOldest => {
                pipeline.stages[0].buffer.remove(0);
                pipeline.total_dropped += 1;
            }
            OverflowPolicy::DropNewest => {
                pipeline.total_dropped += 1;
                return Ok(());
            }
            OverflowPolicy::Sample => {
                pipeline.sample_counter += 1;
                let fill = buf_len as f64 / pipeline.buffer_size as f64;
                let n = (fill * 10.0).max(2.0) as u64;
                if !pipeline.sample_counter.is_multiple_of(n) {
                    pipeline.total_dropped += 1;
                    return Ok(());
                }
                pipeline.stages[0].buffer.remove(0);
                pipeline.total_dropped += 1;
            }
        }
    }
    pipeline.stages[0].buffer.push(msg);
    Ok(())
}

fn bi_pipeline(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::List(agents) = &args[0] else {
        return Err(LxError::type_err(
            "agent.pipeline: first arg must be List of agents",
            span,
        ));
    };
    if agents.is_empty() {
        return Err(LxError::runtime(
            "agent.pipeline: need at least one stage",
            span,
        ));
    }
    let mut buffer_size = 10usize;
    let mut overflow = OverflowPolicy::Block;
    if let Value::Record(opts) = &args[1] {
        if let Some(v) = opts.get("buffer") {
            buffer_size = v
                .as_int()
                .and_then(|n| n.try_into().ok())
                .ok_or_else(|| {
                    LxError::type_err("agent.pipeline: buffer must be positive Int", span)
                })?;
            if buffer_size == 0 {
                return Err(LxError::runtime(
                    "agent.pipeline: buffer must be > 0",
                    span,
                ));
            }
        }
        if let Some(v) = opts.get("overflow") {
            overflow = parse_overflow(v, span)?;
        }
    }
    let stages = agents
        .iter()
        .map(|agent| PipelineStage {
            name: agent_name(agent),
            agents: vec![agent.clone()],
            next_worker: 0,
            buffer: Vec::new(),
            processed: 0,
            total_ms: 0,
        })
        .collect();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    PIPELINES.insert(
        id,
        Pipeline {
            stages,
            buffer_size,
            overflow,
            output: Vec::new(),
            paused: false,
            total_processed: 0,
            total_dropped: 0,
            sample_counter: 0,
            pressure_callbacks: Vec::new(),
        },
    );
    Ok(Value::Ok(Box::new(record! {
        "__pipeline_id" => Value::Int(BigInt::from(id)),
    })))
}

fn bi_pipeline_send(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    {
        let mut pipeline = PIPELINES
            .get_mut(&id)
            .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
        send_to_pipeline(&mut pipeline, args[1].clone(), span, ctx)?;
    }
    super::agent_pipeline_ctrl::fire_pressure_callbacks(id, span, ctx)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_pipeline_collect(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    let mut pipeline = PIPELINES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
    pump(&mut pipeline, span, ctx)?;
    if pipeline.output.is_empty() {
        Ok(Value::Ok(Box::new(Value::None)))
    } else {
        Ok(Value::Ok(Box::new(pipeline.output.remove(0))))
    }
}

fn bi_pipeline_batch(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = get_pipeline_id(&args[0], span)?;
    let Value::List(items) = &args[1] else {
        return Err(LxError::type_err(
            "pipeline_batch: items must be List",
            span,
        ));
    };
    let mut pipeline = PIPELINES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("pipeline not found", span))?;
    for item in items.iter() {
        send_to_pipeline(&mut pipeline, item.clone(), span, ctx)?;
    }
    pump(&mut pipeline, span, ctx)?;
    let results = std::mem::take(&mut pipeline.output);
    Ok(Value::Ok(Box::new(Value::List(Arc::new(results)))))
}

pub fn mk_pipeline_create() -> Value {
    mk("agent.pipeline", 2, bi_pipeline)
}

pub fn mk_pipeline_send() -> Value {
    mk("agent.pipeline_send", 2, bi_pipeline_send)
}

pub fn mk_pipeline_collect() -> Value {
    mk("agent.pipeline_collect", 1, bi_pipeline_collect)
}

pub fn mk_pipeline_batch() -> Value {
    mk("agent.pipeline_batch", 2, bi_pipeline_batch)
}
