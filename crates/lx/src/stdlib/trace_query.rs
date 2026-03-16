use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::trace::{span_to_value, store_id, STORES};
use crate::value::Value;

pub(crate) fn register(m: &mut IndexMap<String, Value>) {
    m.insert("export".into(), mk("trace.export", 2, bi_export));
    m.insert("summary".into(), mk("trace.summary", 1, bi_summary));
    m.insert("filter".into(), mk("trace.filter", 2, bi_filter));
}

fn bi_export(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let export_path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("trace.export: path must be Str", span))?;
    let sid = store_id(&args[1], span)?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("trace: store not found", span))?;
    let mut lines = Vec::new();
    for s in &store.spans {
        let entry = serde_json::json!({
            "name": s.name,
            "input": s.input,
            "output": s.output,
            "score": s.score,
            "duration_ms": s.duration_ms,
            "tags": s.tags,
            "created_at": s.created_at,
        });
        lines.push(
            serde_json::to_string(&entry)
                .map_err(|e| LxError::runtime(format!("trace: JSONL: {e}"), span))?,
        );
    }
    std::fs::write(export_path, lines.join("\n"))
        .map_err(|e| LxError::runtime(format!("trace: export write: {e}"), span))?;
    Ok(Value::Ok(Box::new(Value::Int(BigInt::from(
        lines.len() as i64,
    )))))
}

fn bi_summary(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("trace: store not found", span))?;
    let total = store.spans.len() as i64;
    let scored: Vec<f64> = store.spans.iter().filter_map(|s| s.score).collect();
    let avg_score = if scored.is_empty() {
        0.0
    } else {
        scored.iter().sum::<f64>() / scored.len() as f64
    };
    let durations: Vec<i64> = store.spans.iter().filter_map(|s| s.duration_ms).collect();
    let avg_duration = if durations.is_empty() {
        0i64
    } else {
        durations.iter().sum::<i64>() / durations.len() as i64
    };
    let mut r = IndexMap::new();
    r.insert("total".into(), Value::Int(BigInt::from(total)));
    r.insert(
        "scored".into(),
        Value::Int(BigInt::from(scored.len() as i64)),
    );
    r.insert("avg_score".into(), Value::Float(avg_score));
    r.insert(
        "avg_duration_ms".into(),
        Value::Int(BigInt::from(avg_duration)),
    );
    Ok(Value::Record(Arc::new(r)))
}

fn bi_filter(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(criteria) = &args[0] else {
        return Err(LxError::type_err(
            "trace.filter: first arg must be Record",
            span,
        ));
    };
    let sid = store_id(&args[1], span)?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("trace: store not found", span))?;
    let name_filter = criteria.get("name").and_then(|v| v.as_str());
    let min_score = criteria.get("min_score").and_then(|v| v.as_float());
    let tag_filter = criteria.get("tag").and_then(|v| v.as_str());
    let results: Vec<Value> = store
        .spans
        .iter()
        .filter(|s| {
            if let Some(name) = name_filter
                && s.name != name
            {
                return false;
            }
            if let Some(min) = min_score
                && s.score.unwrap_or(0.0) < min
            {
                return false;
            }
            if let Some(tag) = tag_filter
                && !s.tags.iter().any(|t| t == tag)
            {
                return false;
            }
            true
        })
        .map(span_to_value)
        .collect();
    Ok(Value::List(Arc::new(results)))
}
