use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("is_empty".into(), mk("audit.is_empty", 1, bi_is_empty));
    m.insert("is_too_short".into(), mk("audit.is_too_short", 2, bi_is_too_short));
    m.insert("is_repetitive".into(), mk("audit.is_repetitive", 1, bi_is_repetitive));
    m.insert("is_hedging".into(), mk("audit.is_hedging", 1, bi_is_hedging));
    m.insert("is_refusal".into(), mk("audit.is_refusal", 1, bi_is_refusal));
    m.insert("references_task".into(), mk("audit.references_task", 2, bi_references_task));
    m.insert("files_exist".into(), mk("audit.files_exist", 1, bi_files_exist));
    m.insert("has_diff".into(), mk("audit.has_diff", 1, bi_has_diff));
    m.insert("rubric".into(), mk("audit.rubric", 1, bi_rubric));
    m.insert("evaluate".into(), mk("audit.evaluate", 2, bi_evaluate));
    m.insert("quick_check".into(), mk("audit.quick_check", 1, bi_quick_check));
    m
}

fn as_str_arg<'a>(v: &'a Value, name: &str, span: Span) -> Result<&'a str, LxError> {
    v.as_str().ok_or_else(|| LxError::type_err(format!("{name} expects Str"), span))
}

fn bi_is_empty(args: &[Value], span: Span) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_empty", span)?;
    Ok(Value::Bool(s.trim().is_empty()))
}

fn bi_is_too_short(args: &[Value], span: Span) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_too_short", span)?;
    let min = args[1].as_int()
        .ok_or_else(|| LxError::type_err("audit.is_too_short: min_len must be Int", span))?;
    let min: usize = min.try_into()
        .map_err(|_| LxError::runtime("audit.is_too_short: min_len too large", span))?;
    Ok(Value::Bool(s.len() < min))
}

fn bi_is_repetitive(args: &[Value], span: Span) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_repetitive", span)?;
    let sentences: Vec<&str> = s.split(['.', '\n'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if sentences.len() < 2 {
        return Ok(Value::Bool(false));
    }
    let mut seen = std::collections::HashSet::new();
    let mut dupes = 0usize;
    for sent in &sentences {
        let norm = sent.to_lowercase();
        if !seen.insert(norm) {
            dupes += 1;
        }
    }
    Ok(Value::Bool(dupes * 2 >= sentences.len()))
}

pub(crate) const HEDGING: &[&str] = &[
    "i think", "maybe", "possibly", "i'm not sure", "perhaps",
    "it might", "it could be", "i believe", "not entirely sure",
];

fn bi_is_hedging(args: &[Value], span: Span) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_hedging", span)?;
    let lower = s.to_lowercase();
    Ok(Value::Bool(HEDGING.iter().any(|h| lower.contains(h))))
}

pub(crate) const REFUSAL: &[&str] = &[
    "i can't", "i'm unable", "as an ai", "i cannot",
    "i'm not able", "i don't have the ability",
];

fn bi_is_refusal(args: &[Value], span: Span) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_refusal", span)?;
    let lower = s.to_lowercase();
    Ok(Value::Bool(REFUSAL.iter().any(|r| lower.contains(r))))
}

fn bi_references_task(args: &[Value], span: Span) -> Result<Value, LxError> {
    let output = as_str_arg(&args[0], "audit.references_task(output)", span)?;
    let task = as_str_arg(&args[1], "audit.references_task(task)", span)?;
    let output_lower = output.to_lowercase();
    let keywords: Vec<&str> = task.split_whitespace()
        .filter(|w| w.len() > 3)
        .collect();
    if keywords.is_empty() {
        return Ok(Value::Bool(true));
    }
    let hits = keywords.iter()
        .filter(|kw| output_lower.contains(&kw.to_lowercase()))
        .count();
    Ok(Value::Bool(hits * 3 >= keywords.len()))
}

fn bi_files_exist(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::List(paths) = &args[0] else {
        return Err(LxError::type_err("audit.files_exist expects List of Str", span));
    };
    for p in paths.iter() {
        let s = p.as_str().ok_or_else(||
            LxError::type_err("audit.files_exist: path must be Str", span))?;
        if !std::path::Path::new(s).exists() {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}

fn bi_has_diff(args: &[Value], span: Span) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.has_diff", span)?;
    let has = s.contains("diff --git") || s.contains("@@")
        || (s.contains("+++") && s.contains("---"));
    Ok(Value::Bool(has))
}

fn bi_rubric(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::List(_) = &args[0] else {
        return Err(LxError::type_err("audit.rubric expects List of categories", span));
    };
    Ok(args[0].clone())
}

fn bi_evaluate(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::List(categories) = &args[0] else {
        return Err(LxError::type_err("audit.evaluate: first arg must be rubric List", span));
    };
    let data = &args[1];
    let threshold: i64 = match data {
        Value::Record(r) => r.get("threshold")
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
        let weight: i64 = r.get("weight").and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok()).unwrap_or(0);
        let check_fn = r.get("check")
            .ok_or_else(|| LxError::runtime(
                format!("audit.evaluate: category '{name}' missing 'check' function"), span,
            ))?;
        let result = call_value(check_fn, data.clone(), span)?;
        let passed = matches!(result, Value::Bool(true));
        let score: i64 = if passed { 100 } else { 0 };
        total_score += score * weight;
        total_weight += weight;
        let mut cr = IndexMap::new();
        cr.insert("name".into(), Value::Str(Arc::from(name)));
        cr.insert("score".into(), Value::Int(BigInt::from(score)));
        cr.insert("passed".into(), Value::Bool(passed));
        cat_results.push(Value::Record(Arc::new(cr)));
        if !passed {
            failed_names.push(Value::Str(Arc::from(name)));
            feedback_parts.push(format!("{name}: failed"));
        }
    }
    let final_score = if total_weight > 0 { total_score / total_weight } else { 0 };
    let overall_passed = final_score >= threshold;
    let feedback = feedback_parts.join("; ");
    let mut result = IndexMap::new();
    result.insert("score".into(), Value::Int(BigInt::from(final_score)));
    result.insert("passed".into(), Value::Bool(overall_passed));
    result.insert("categories".into(), Value::List(Arc::new(cat_results)));
    result.insert("feedback".into(), Value::Str(Arc::from(feedback.as_str())));
    result.insert("failed".into(), Value::List(Arc::new(failed_names)));
    Ok(Value::Record(Arc::new(result)))
}

fn bi_quick_check(args: &[Value], span: Span) -> Result<Value, LxError> {
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
                format!("output too short ({} < {min})", output.len()).as_str()
            )));
        }
    }
    if opts.get("no_hedging").and_then(|v| v.as_bool()).unwrap_or(false) {
        let lower = output.to_lowercase();
        if HEDGING.iter().any(|h| lower.contains(h)) {
            reasons.push(Value::Str(Arc::from("output contains hedging language")));
        }
    }
    if opts.get("no_refusal").and_then(|v| v.as_bool()).unwrap_or(false) {
        let lower = output.to_lowercase();
        if REFUSAL.iter().any(|r| lower.contains(r)) {
            reasons.push(Value::Str(Arc::from("output contains refusal language")));
        }
    }
    if let Some(task) = opts.get("task").and_then(|v| v.as_str()) {
        let output_lower = output.to_lowercase();
        let keywords: Vec<&str> = task.split_whitespace()
            .filter(|w| w.len() > 3).collect();
        if !keywords.is_empty() {
            let hits = keywords.iter()
                .filter(|kw| output_lower.contains(&kw.to_lowercase())).count();
            if hits * 3 < keywords.len() {
                reasons.push(Value::Str(Arc::from("output doesn't reference task")));
            }
        }
    }
    let mut result = IndexMap::new();
    result.insert("passed".into(), Value::Bool(reasons.is_empty()));
    result.insert("reasons".into(), Value::List(Arc::new(reasons)));
    Ok(Value::Record(Arc::new(result)))
}
