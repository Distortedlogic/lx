use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::profile::{profile_id, PROFILES};

pub fn register(m: &mut IndexMap<String, Value>) {
    m.insert(
        "best_strategy".into(),
        mk("profile.best_strategy", 2, bi_best_strategy),
    );
    m.insert(
        "rank_strategies".into(),
        mk("profile.rank_strategies", 2, bi_rank_strategies),
    );
    m.insert(
        "adapt_strategy".into(),
        mk("profile.adapt_strategy", 2, bi_adapt_strategy),
    );
}

struct StrategyStats {
    approach: String,
    avg_score: f64,
    count: usize,
    trend: String,
}

fn collect_strategies(
    id: u64,
    problem: &str,
    span: Span,
) -> Result<Vec<StrategyStats>, LxError> {
    let p = PROFILES
        .get(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    let prefix = format!("strategy:{problem}:");
    let mut by_approach: IndexMap<String, Vec<f64>> = IndexMap::new();
    for (domain, entry) in p.knowledge.iter() {
        if let Some(suffix) = domain.strip_prefix(&prefix) {
            let score = extract_score(&entry.data);
            by_approach
                .entry(suffix.to_string())
                .or_default()
                .push(score);
        }
    }
    let mut stats: Vec<StrategyStats> = by_approach
        .into_iter()
        .map(|(approach, scores)| {
            let count = scores.len();
            let avg = scores.iter().sum::<f64>() / count as f64;
            let trend = compute_trend(&scores);
            StrategyStats {
                approach,
                avg_score: avg,
                count,
                trend,
            }
        })
        .collect();
    stats.sort_by(|a, b| b.avg_score.partial_cmp(&a.avg_score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(stats)
}

fn extract_score(data: &Value) -> f64 {
    match data {
        Value::Record(r) => match r.get("score") {
            Some(Value::Float(f)) => *f,
            Some(Value::Int(n)) => {
                use num_traits::ToPrimitive;
                n.to_f64().unwrap_or(0.0)
            }
            _ => 0.0,
        },
        Value::Float(f) => *f,
        Value::Int(n) => {
            use num_traits::ToPrimitive;
            n.to_f64().unwrap_or(0.0)
        }
        _ => 0.0,
    }
}

fn compute_trend(scores: &[f64]) -> String {
    if scores.len() < 2 {
        return "stable".to_string();
    }
    let mid = scores.len() / 2;
    let first_half: f64 = scores[..mid].iter().sum::<f64>() / mid as f64;
    let second_half: f64 = scores[mid..].iter().sum::<f64>() / (scores.len() - mid) as f64;
    let diff = second_half - first_half;
    if diff > 5.0 {
        "improving".to_string()
    } else if diff < -5.0 {
        "declining".to_string()
    } else {
        "stable".to_string()
    }
}

fn stats_to_value(s: &StrategyStats) -> Value {
    let mut f = IndexMap::new();
    f.insert(
        "approach".into(),
        Value::Str(Arc::from(s.approach.as_str())),
    );
    f.insert("avg_score".into(), Value::Float(s.avg_score));
    f.insert("count".into(), Value::Int(BigInt::from(s.count)));
    f.insert("trend".into(), Value::Str(Arc::from(s.trend.as_str())));
    Value::Record(Arc::new(f))
}

fn bi_best_strategy(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let problem = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("profile.best_strategy: problem must be Str", span))?;
    let stats = collect_strategies(id, problem, span)?;
    match stats.first() {
        Some(best) => Ok(Value::Ok(Box::new(stats_to_value(best)))),
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("no strategies for '{problem}'").as_str(),
        ))))),
    }
}

fn bi_rank_strategies(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let problem = args[1]
        .as_str()
        .ok_or_else(|| {
            LxError::type_err("profile.rank_strategies: problem must be Str", span)
        })?;
    let stats = collect_strategies(id, problem, span)?;
    let items: Vec<Value> = stats.iter().map(stats_to_value).collect();
    Ok(Value::List(Arc::new(items)))
}

fn bi_adapt_strategy(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let (problem, explore_rate) = match &args[1] {
        Value::Str(s) => (s.to_string(), 0.2),
        Value::Record(r) => {
            let prob = r
                .get("problem")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    LxError::type_err("profile.adapt_strategy: need problem field", span)
                })?
                .to_string();
            let rate = match r.get("explore_rate") {
                Some(Value::Float(f)) => *f,
                _ => 0.2,
            };
            (prob, rate)
        }
        _ => {
            return Err(LxError::type_err(
                "profile.adapt_strategy: expected Str or Record",
                span,
            ))
        }
    };
    let stats = collect_strategies(id, &problem, span)?;
    if stats.is_empty() {
        let mut f = IndexMap::new();
        f.insert("approach".into(), Value::Str(Arc::from("none")));
        f.insert("mode".into(), Value::Str(Arc::from("explore")));
        return Ok(Value::Record(Arc::new(f)));
    }
    let roll: f64 = fastrand::f64();
    let (approach, mode) = if roll < explore_rate && stats.len() > 1 {
        let idx = fastrand::usize(1..stats.len());
        (&stats[idx].approach, "explore")
    } else {
        (&stats[0].approach, "exploit")
    };
    let mut f = IndexMap::new();
    f.insert("approach".into(), Value::Str(Arc::from(approach.as_str())));
    f.insert("mode".into(), Value::Str(Arc::from(mode)));
    Ok(Value::Record(Arc::new(f)))
}

