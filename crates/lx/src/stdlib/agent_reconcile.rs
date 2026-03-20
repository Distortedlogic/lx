use std::sync::Arc;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::agent_reconcile_score as score;
use super::agent_reconcile_strat as strat;

pub fn mk_reconcile() -> Value {
    mk("agent.reconcile", 2, bi_reconcile)
}

pub(super) struct ReconcileConfig {
    pub(super) strategy: Strategy,
    pub(super) key: Option<Value>,
    pub(super) conflict: Option<Value>,
    pub(super) flatten: Option<String>,
    pub(super) min_agreement: usize,
    pub(super) quorum: Quorum,
    pub(super) vote_field: String,
    pub(super) weight: Option<Value>,
    pub(super) score: Option<Value>,
    pub(super) early_stop: Option<f64>,
}

pub(super) enum Strategy {
    Union,
    Intersection,
    Vote,
    HighestConfidence,
    MaxScore,
    MergeFields,
    Custom(Value),
}

#[derive(Clone)]
pub(super) enum Quorum {
    Any,
    Majority,
    Unanimous,
    N(usize),
}

fn parse_config(cfg: &Value, span: Span) -> Result<ReconcileConfig, LxError> {
    let Value::Record(r) = cfg else {
        return Err(LxError::type_err(
            "agent.reconcile: config must be a Record",
            span,
        ));
    };
    let strategy_val = r
        .get("strategy")
        .ok_or_else(|| LxError::runtime("agent.reconcile: config missing 'strategy'", span))?;
    let strategy = match strategy_val {
        Value::Str(s) => match s.as_ref() {
            "union" => Strategy::Union,
            "intersection" => Strategy::Intersection,
            "vote" => Strategy::Vote,
            "highest_confidence" => Strategy::HighestConfidence,
            "max_score" => Strategy::MaxScore,
            "merge_fields" => Strategy::MergeFields,
            other => {
                return Err(LxError::runtime(
                    format!("agent.reconcile: unknown strategy '{other}'"),
                    span,
                ));
            }
        },
        Value::Func(_) | Value::BuiltinFunc(_) => Strategy::Custom(strategy_val.clone()),
        _ => {
            return Err(LxError::type_err(
                "agent.reconcile: strategy must be Str or Fn",
                span,
            ));
        }
    };
    let quorum = match r.get("quorum") {
        Some(Value::Str(s)) => match s.as_ref() {
            "unanimous" => Quorum::Unanimous,
            "majority" => Quorum::Majority,
            "any" => Quorum::Any,
            other => {
                return Err(LxError::runtime(
                    format!("agent.reconcile: unknown quorum '{other}'"),
                    span,
                ));
            }
        },
        Some(Value::Record(qr)) => {
            let n = qr
                .get("n")
                .and_then(|v| v.as_int())
                .and_then(|n| n.to_usize())
                .ok_or_else(|| {
                    LxError::type_err("agent.reconcile: quorum {n: Int} expected", span)
                })?;
            Quorum::N(n)
        }
        Some(Value::Int(n)) => Quorum::N(n.to_usize().unwrap_or(1)),
        None => Quorum::Any,
        _ => {
            return Err(LxError::type_err(
                "agent.reconcile: quorum must be Str, {n: Int}, or Int",
                span,
            ));
        }
    };
    Ok(ReconcileConfig {
        strategy,
        key: r.get("key").cloned(),
        conflict: r.get("conflict").cloned(),
        flatten: r.get("flatten").and_then(|v| v.as_str()).map(String::from),
        min_agreement: r
            .get("min_agreement")
            .and_then(|v| v.as_int())
            .and_then(|n| n.to_usize())
            .unwrap_or(1),
        quorum,
        vote_field: r
            .get("vote_field")
            .and_then(|v| v.as_str())
            .unwrap_or("approved")
            .to_string(),
        weight: r.get("weight").cloned(),
        score: r.get("score").cloned(),
        early_stop: r.get("early_stop").and_then(|v| v.as_float()),
    })
}

pub(super) fn make_result(
    merged: Value,
    sources: usize,
    conflicts: Vec<Value>,
    dropped: Vec<Value>,
    rounds: usize,
    dissenting: Vec<String>,
) -> Value {
    let diss: Vec<Value> = dissenting
        .into_iter()
        .map(|s| Value::Str(Arc::from(s.as_str())))
        .collect();
    record! {
        "merged" => merged,
        "sources" => Value::Int(BigInt::from(sources)),
        "conflicts" => Value::List(Arc::new(conflicts)),
        "dropped" => Value::List(Arc::new(dropped)),
        "rounds" => Value::Int(BigInt::from(rounds)),
        "dissenting" => Value::List(Arc::new(diss)),
    }
}

pub(super) fn make_conflict_entry(key: Value, values: Vec<Value>, resolved: Value) -> Value {
    record! {
        "key" => key,
        "values" => Value::List(Arc::new(values)),
        "resolved" => resolved,
    }
}

pub(super) fn call_key(
    key_fn: &Value,
    item: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<String, LxError> {
    let result = call_value_sync(key_fn, item.clone(), span, ctx)?;
    Ok(format!("{result}"))
}

pub(super) fn flatten_results(
    results: &[Value],
    field: &str,
    span: Span,
) -> Result<Vec<Value>, LxError> {
    let mut items = Vec::new();
    for r in results {
        match r {
            Value::Record(rec) => match rec.get(field) {
                Some(Value::List(l)) => items.extend(l.as_ref().iter().cloned()),
                Some(_) => {
                    return Err(LxError::type_err(
                        format!("agent.reconcile: flatten field '{field}' must be a List"),
                        span,
                    ));
                }
                None => {}
            },
            _ => {
                return Err(LxError::type_err(
                    "agent.reconcile: flatten requires Record results",
                    span,
                ));
            }
        }
    }
    Ok(items)
}

pub(super) fn resolve_conflict(
    conflict_fn: &Option<Value>,
    a: &Value,
    b: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    match conflict_fn {
        Some(f) => {
            let partial = call_value_sync(f, a.clone(), span, ctx)?;
            call_value_sync(&partial, b.clone(), span, ctx)
        }
        None => Ok(a.clone()),
    }
}

fn bi_reconcile(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let results = args[0]
        .as_list()
        .ok_or_else(|| LxError::type_err("agent.reconcile: first arg must be List", span))?;
    let cfg = parse_config(&args[1], span)?;
    match cfg.strategy {
        Strategy::Union => strat::do_union(results, &cfg, span, ctx),
        Strategy::Intersection => strat::do_intersection(results, &cfg, span, ctx),
        Strategy::Vote => strat::do_vote(results, &cfg, span, ctx),
        Strategy::HighestConfidence => score::do_highest_confidence(results, span),
        Strategy::MaxScore => score::do_max_score(results, &cfg, span, ctx),
        Strategy::MergeFields => score::do_merge_fields(results, &cfg, span, ctx),
        Strategy::Custom(f) => {
            let merged = call_value_sync(&f, args[0].clone(), span, ctx)?;
            Ok(make_result(
                merged,
                results.len(),
                vec![],
                vec![],
                0,
                vec![],
            ))
        }
    }
}
