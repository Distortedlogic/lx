use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::trace::{store_id, STORES};
use crate::value::Value;

pub(crate) fn register(m: &mut IndexMap<String, Value>) {
    m.insert(
        "improvement_rate".into(),
        mk("trace.improvement_rate", 2, bi_improvement_rate),
    );
    m.insert(
        "should_stop".into(),
        mk("trace.should_stop", 2, bi_should_stop),
    );
}

fn progress_scores(sid: u64) -> Vec<f64> {
    STORES
        .get(&sid)
        .map(|store| {
            store
                .spans
                .iter()
                .filter(|s| s.name == "progress" && s.score.is_some())
                .filter_map(|s| s.score)
                .collect()
        })
        .unwrap_or_default()
}

fn classify_trend(avg_delta: f64, recent_delta: f64) -> &'static str {
    if recent_delta < 0.0 {
        return "regressing";
    }
    if recent_delta.abs() < 0.001 {
        return "plateau";
    }
    if avg_delta.abs() < 0.001 {
        return "improving";
    }
    if recent_delta > avg_delta * 1.2 {
        return "improving";
    }
    if recent_delta < avg_delta * 0.8 {
        return "diminishing";
    }
    "steady"
}

fn bi_improvement_rate(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let window: usize = args[0]
        .as_int()
        .and_then(|n| n.try_into().ok())
        .ok_or_else(|| {
            LxError::type_err("trace.improvement_rate: window must be Int", span)
        })?;
    let sid = store_id(&args[1], span)?;
    let scores = progress_scores(sid);
    let samples = scores.len().min(window);
    if samples < 2 {
        let mut r = IndexMap::new();
        r.insert("avg_delta".into(), Value::Float(0.0));
        r.insert("recent_delta".into(), Value::Float(0.0));
        r.insert("trend".into(), Value::Str(Arc::from("insufficient")));
        r.insert(
            "samples".into(),
            Value::Int(BigInt::from(samples as i64)),
        );
        return Ok(Value::Record(Arc::new(r)));
    }
    let start = scores.len().saturating_sub(window);
    let windowed = &scores[start..];
    let deltas: Vec<f64> = windowed.windows(2).map(|w| w[1] - w[0]).collect();
    let avg_delta = deltas.iter().sum::<f64>() / deltas.len() as f64;
    let recent_delta = deltas.last().copied().unwrap_or(0.0);
    let trend = classify_trend(avg_delta, recent_delta);
    let mut r = IndexMap::new();
    r.insert("avg_delta".into(), Value::Float(avg_delta));
    r.insert("recent_delta".into(), Value::Float(recent_delta));
    r.insert("trend".into(), Value::Str(Arc::from(trend)));
    r.insert(
        "samples".into(),
        Value::Int(BigInt::from(windowed.len() as i64)),
    );
    Ok(Value::Record(Arc::new(r)))
}

fn bi_should_stop(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let Value::Record(opts) = &args[0] else {
        return Err(LxError::type_err(
            "trace.should_stop: first arg must be Record",
            span,
        ));
    };
    let min_delta = opts.get("min_delta").and_then(|v| v.as_float()).ok_or_else(
        || LxError::type_err("trace.should_stop: min_delta required (Float)", span),
    )?;
    let window: usize = opts
        .get("window")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
        .ok_or_else(|| {
            LxError::type_err("trace.should_stop: window required (Int)", span)
        })?;
    let sid = store_id(&args[1], span)?;
    let scores = progress_scores(sid);
    if scores.len() < 2 {
        return Ok(Value::Bool(false));
    }
    let needed = window + 1;
    let start = scores.len().saturating_sub(needed);
    let windowed = &scores[start..];
    let deltas: Vec<f64> = windowed.windows(2).map(|w| w[1] - w[0]).collect();
    if deltas.is_empty() {
        return Ok(Value::Bool(false));
    }
    let all_below = deltas.iter().all(|d| *d <= min_delta);
    Ok(Value::Bool(all_below))
}
