use std::sync::Arc;

use indexmap::IndexMap;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::agent_reconcile::{ReconcileConfig, make_conflict_entry, make_result, resolve_conflict};

pub(super) fn do_highest_confidence(results: &[Value], span: Span) -> Result<Value, LxError> {
    let mut best: Option<(f64, &Value)> = None;
    for r in results {
        let conf = match r {
            Value::Record(rec) => rec
                .get("confidence")
                .and_then(|v| v.as_float().or_else(|| v.as_int().and_then(|n| n.to_f64())))
                .ok_or_else(|| {
                    LxError::type_err(
                        "agent.reconcile: highest_confidence requires 'confidence' field",
                        span,
                    )
                })?,
            _ => {
                return Err(LxError::type_err(
                    "agent.reconcile: highest_confidence requires Record results",
                    span,
                ));
            }
        };
        match best {
            Some((best_conf, _)) if conf <= best_conf => {}
            _ => best = Some((conf, r)),
        }
    }
    let merged = best.map(|(_, v)| v.clone()).unwrap_or(Value::Unit);
    Ok(make_result(
        merged,
        results.len(),
        vec![],
        vec![],
        0,
        vec![],
    ))
}

pub(super) fn do_max_score(
    results: &[Value],
    cfg: &ReconcileConfig,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let score_fn = cfg.score.as_ref().ok_or_else(|| {
        LxError::runtime(
            "agent.reconcile: max_score strategy requires 'score' function",
            span,
        )
    })?;
    let mut best: Option<(f64, Value)> = None;
    for r in results {
        let score_val = call_value_sync(score_fn, r.clone(), span, ctx)?;
        let s = score_val
            .as_float()
            .or_else(|| score_val.as_int().and_then(|n| n.to_f64()))
            .ok_or_else(|| {
                LxError::type_err(
                    "agent.reconcile: score function must return Float or Int",
                    span,
                )
            })?;
        if let Some(threshold) = cfg.early_stop
            && s >= threshold
        {
            return Ok(make_result(
                r.clone(),
                results.len(),
                vec![],
                vec![],
                0,
                vec![],
            ));
        }
        match best {
            Some((best_s, _)) if s <= best_s => {}
            _ => best = Some((s, r.clone())),
        }
    }
    let merged = best.map(|(_, v)| v).unwrap_or(Value::Unit);
    Ok(make_result(
        merged,
        results.len(),
        vec![],
        vec![],
        0,
        vec![],
    ))
}

pub(super) fn do_merge_fields(
    results: &[Value],
    cfg: &ReconcileConfig,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let mut merged: IndexMap<String, Value> = IndexMap::new();
    let mut conflicts = Vec::new();
    for result in results {
        let Value::Record(rec) = result else {
            return Err(LxError::type_err(
                "agent.reconcile: merge_fields requires Record results",
                span,
            ));
        };
        for (k, v) in rec.iter() {
            if let Some(existing) = merged.get(k) {
                match (existing, v) {
                    (Value::List(a), Value::List(b)) => {
                        let mut combined = a.as_ref().clone();
                        combined.extend(b.as_ref().iter().cloned());
                        merged.insert(k.clone(), Value::List(Arc::new(combined)));
                    }
                    _ => {
                        let resolved = resolve_conflict(&cfg.conflict, existing, v, span, ctx)?;
                        conflicts.push(make_conflict_entry(
                            Value::Str(Arc::from(k.as_str())),
                            vec![existing.clone(), v.clone()],
                            resolved.clone(),
                        ));
                        merged.insert(k.clone(), resolved);
                    }
                }
            } else {
                merged.insert(k.clone(), v.clone());
            }
        }
    }
    Ok(make_result(
        Value::Record(Arc::new(merged)),
        results.len(),
        conflicts,
        vec![],
        0,
        vec![],
    ))
}
