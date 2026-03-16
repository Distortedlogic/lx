use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::agent_reconcile::{
    Quorum, ReconcileConfig, call_key, flatten_results, make_conflict_entry, make_result,
    resolve_conflict,
};

pub(super) fn do_union(
    results: &[Value],
    cfg: &ReconcileConfig,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let key_fn = cfg.key.as_ref().ok_or_else(|| {
        LxError::runtime(
            "agent.reconcile: union strategy requires 'key' function",
            span,
        )
    })?;
    let items = match &cfg.flatten {
        Some(field) => flatten_results(results, field, span)?,
        None => results
            .iter()
            .flat_map(|r| match r {
                Value::List(l) => l.as_ref().clone(),
                other => vec![other.clone()],
            })
            .collect(),
    };
    let mut seen: IndexMap<String, Value> = IndexMap::new();
    let mut conflicts = Vec::new();
    for item in &items {
        let k = call_key(key_fn, item, span, ctx)?;
        if let Some(existing) = seen.get(&k) {
            let resolved = resolve_conflict(&cfg.conflict, existing, item, span, ctx)?;
            conflicts.push(make_conflict_entry(
                Value::Str(Arc::from(k.as_str())),
                vec![existing.clone(), item.clone()],
                resolved.clone(),
            ));
            seen.insert(k, resolved);
        } else {
            seen.insert(k, item.clone());
        }
    }
    let merged: Vec<Value> = seen.into_values().collect();
    Ok(make_result(
        Value::List(Arc::new(merged)),
        results.len(),
        conflicts,
        vec![],
        0,
        vec![],
    ))
}

pub(super) fn do_intersection(
    results: &[Value],
    cfg: &ReconcileConfig,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let key_fn = cfg.key.as_ref().ok_or_else(|| {
        LxError::runtime(
            "agent.reconcile: intersection strategy requires 'key' function",
            span,
        )
    })?;
    let n = results.len();
    let mut key_counts: HashMap<String, usize> = HashMap::new();
    let mut first_seen: HashMap<String, Value> = HashMap::new();
    for result in results {
        let items: Vec<Value> = match &cfg.flatten {
            Some(field) => flatten_results(std::slice::from_ref(result), field, span)?,
            None => match result {
                Value::List(l) => l.as_ref().clone(),
                other => vec![other.clone()],
            },
        };
        let mut seen_in_result: HashSet<String> = HashSet::new();
        for item in &items {
            let k = call_key(key_fn, item, span, ctx)?;
            if !seen_in_result.contains(&k) {
                seen_in_result.insert(k.clone());
                *key_counts.entry(k.clone()).or_insert(0) += 1;
                first_seen.entry(k).or_insert_with(|| item.clone());
            }
        }
    }
    let mut merged = Vec::new();
    let mut dropped = Vec::new();
    for (k, count) in &key_counts {
        if *count >= n {
            if let Some(item) = first_seen.get(k) {
                merged.push(item.clone());
            }
        } else if let Some(item) = first_seen.get(k) {
            dropped.push(item.clone());
        }
    }
    Ok(make_result(
        Value::List(Arc::new(merged)),
        results.len(),
        vec![],
        dropped,
        0,
        vec![],
    ))
}

pub(super) fn do_vote(
    results: &[Value],
    cfg: &ReconcileConfig,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let vote_of = |r: &Value| -> Value {
        match r {
            Value::Record(rec) => rec.get(&cfg.vote_field).cloned().unwrap_or(Value::Unit),
            other => other.clone(),
        }
    };
    let mut tallies: IndexMap<String, (Value, f64)> = IndexMap::new();
    let mut total_weight: f64 = 0.0;
    for (i, result) in results.iter().enumerate() {
        let vote_val = vote_of(result);
        let w = match &cfg.weight {
            Some(wf) => {
                let w_val = call_value(wf, Value::Int(BigInt::from(i)), span, ctx)?;
                w_val
                    .as_float()
                    .or_else(|| w_val.as_int().and_then(|n| n.to_f64()))
                    .unwrap_or(1.0)
            }
            None => 1.0,
        };
        total_weight += w;
        let key = format!("{vote_val}");
        tallies.entry(key).or_insert((vote_val, 0.0)).1 += w;
    }
    let (winner_key, winner_val, winner_weight) = tallies
        .iter()
        .max_by(|a, b| {
            a.1.1
                .partial_cmp(&b.1.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(k, (v, w))| (k.clone(), v.clone(), *w))
        .unwrap_or_else(|| (String::new(), Value::Unit, 0.0));
    let quorum_met = match &cfg.quorum {
        Quorum::Any => true,
        Quorum::Majority => winner_weight > total_weight / 2.0,
        Quorum::Unanimous => (winner_weight - total_weight).abs() < 1e-10 && tallies.len() == 1,
        Quorum::N(n) => winner_weight >= *n as f64,
    };
    let mut dissenting = Vec::new();
    for (i, result) in results.iter().enumerate() {
        if format!("{}", vote_of(result)) != winner_key {
            dissenting.push(format!("{i}"));
        }
    }
    let mut dropped = Vec::new();
    if !quorum_met {
        for (key, (val, weight)) in &tallies {
            if *key != winner_key || *weight < cfg.min_agreement as f64 {
                dropped.push(val.clone());
            }
        }
    }
    let merged = if quorum_met { winner_val } else { Value::None };
    Ok(make_result(
        merged,
        results.len(),
        vec![],
        dropped,
        0,
        dissenting,
    ))
}

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
        let score_val = call_value(score_fn, r.clone(), span, ctx)?;
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
