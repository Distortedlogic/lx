use std::sync::Arc;

use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::{HEDGING, REFUSAL, check_references_task};

pub(super) fn bi_evaluate(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let Value::List(categories) = &args[0] else {
        return Err(LxError::type_err(
            "audit.evaluate: first arg must be rubric List",
            span,
        ));
    };
    let data = &args[1];
    let threshold: i64 = match data {
        Value::Record(r) => r
            .get("threshold")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .unwrap_or(95),
        _ => 95,
    };
    let mut total_score: i64 = 0;
    let mut total_weight: i64 = 0;
    let mut cat_results = Vec::new();
    let mut failed_names = Vec::new();
    let mut feedback_parts = Vec::new();
    for cat in categories.iter() {
        let Value::Record(r) = cat else { continue };
        let name = r.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
        let weight: i64 = r
            .get("weight")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .unwrap_or(0);
        let check_fn = r.get("check").ok_or_else(|| {
            LxError::runtime(
                format!("audit.evaluate: category '{name}' missing 'check' function"),
                span,
            )
        })?;
        let result = call_value_sync(check_fn, data.clone(), span, ctx)?;
        let passed = matches!(result, Value::Bool(true));
        let score: i64 = if passed { 100 } else { 0 };
        total_score += score * weight;
        total_weight += weight;
        cat_results.push(record! {
            "name" => Value::Str(Arc::from(name)),
            "score" => Value::Int(BigInt::from(score)),
            "passed" => Value::Bool(passed),
        });
        if !passed {
            failed_names.push(Value::Str(Arc::from(name)));
            feedback_parts.push(format!("{name}: failed"));
        }
    }
    let final_score = if total_weight > 0 {
        total_score / total_weight
    } else {
        0
    };
    let overall_passed = final_score >= threshold;
    let feedback = feedback_parts.join("; ");
    Ok(record! {
        "score" => Value::Int(BigInt::from(final_score)),
        "passed" => Value::Bool(overall_passed),
        "categories" => Value::List(Arc::new(cat_results)),
        "feedback" => Value::Str(Arc::from(feedback.as_str())),
        "failed" => Value::List(Arc::new(failed_names)),
    })
}

pub(super) fn bi_quick_check(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let Value::Record(opts) = &args[0] else {
        return Err(LxError::type_err("audit.quick_check expects Record", span));
    };
    let output = opts.get("output").and_then(|v| v.as_str()).unwrap_or("");
    let mut reasons = Vec::new();
    if output.trim().is_empty() {
        reasons.push(Value::Str(Arc::from("output is empty")));
    }
    if let Some(min) = opts.get("min_length").and_then(|v| v.as_int()) {
        let min: usize = min.try_into().unwrap_or(0);
        if output.len() < min {
            reasons.push(Value::Str(Arc::from(
                format!("output too short ({} < {min})", output.len()).as_str(),
            )));
        }
    }
    if opts
        .get("no_hedging")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let lower = output.to_lowercase();
        if HEDGING.iter().any(|h| lower.contains(h)) {
            reasons.push(Value::Str(Arc::from("output contains hedging language")));
        }
    }
    if opts
        .get("no_refusal")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let lower = output.to_lowercase();
        if REFUSAL.iter().any(|r| lower.contains(r)) {
            reasons.push(Value::Str(Arc::from("output contains refusal language")));
        }
    }
    let references_task = opts
        .get("task")
        .and_then(|v| v.as_str())
        .map(|t| check_references_task(output, t))
        .unwrap_or(true);
    if !references_task {
        reasons.push(Value::Str(Arc::from("output doesn't reference task")));
    }
    Ok(record! {
        "passed" => Value::Bool(reasons.is_empty()),
        "reasons" => Value::List(Arc::new(reasons)),
    })
}
