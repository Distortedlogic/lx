#[path = "audit_score.rs"]
mod audit_score;

use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("is_empty".into(), mk("audit.is_empty", 1, bi_is_empty));
    m.insert(
        "is_too_short".into(),
        mk("audit.is_too_short", 2, bi_is_too_short),
    );
    m.insert(
        "is_repetitive".into(),
        mk("audit.is_repetitive", 1, bi_is_repetitive),
    );
    m.insert(
        "is_hedging".into(),
        mk("audit.is_hedging", 1, bi_is_hedging),
    );
    m.insert(
        "is_refusal".into(),
        mk("audit.is_refusal", 1, bi_is_refusal),
    );
    m.insert(
        "references_task".into(),
        mk("audit.references_task", 2, bi_references_task),
    );
    m.insert(
        "files_exist".into(),
        mk("audit.files_exist", 1, bi_files_exist),
    );
    m.insert("has_diff".into(), mk("audit.has_diff", 1, bi_has_diff));
    m.insert("rubric".into(), mk("audit.rubric", 1, bi_rubric));
    m.insert(
        "evaluate".into(),
        mk("audit.evaluate", 2, audit_score::bi_evaluate),
    );
    m.insert(
        "quick_check".into(),
        mk("audit.quick_check", 1, audit_score::bi_quick_check),
    );
    m
}

pub(crate) fn make_eval_category(name: &str, score: i64, passed: bool, detail: &str) -> Value {
    record! {
        "name" => Value::Str(Arc::from(name)),
        "score" => Value::Int(BigInt::from(score)),
        "passed" => Value::Bool(passed),
        "feedback" => Value::Str(Arc::from(detail)),
    }
}

pub(crate) fn build_eval_result(
    score: i64,
    passed: bool,
    categories: Vec<Value>,
    feedback: &str,
    failed: Vec<Value>,
) -> Value {
    record! {
        "score" => Value::Int(BigInt::from(score)),
        "passed" => Value::Bool(passed),
        "categories" => Value::List(Arc::new(categories)),
        "feedback" => Value::Str(Arc::from(feedback)),
        "failed" => Value::List(Arc::new(failed)),
    }
}

pub(crate) fn keyword_overlap(
    haystack: &str,
    keywords_source: &str,
    min_word_len: usize,
) -> (usize, usize) {
    let haystack_lower = haystack.to_lowercase();
    let keywords: Vec<String> = keywords_source
        .split_whitespace()
        .filter(|w| w.len() > min_word_len)
        .map(|w| w.to_lowercase())
        .collect();
    if keywords.is_empty() {
        return (0, 0);
    }
    let hits = keywords
        .iter()
        .filter(|kw| haystack_lower.contains(kw.as_str()))
        .count();
    (hits, keywords.len())
}

fn as_str_arg<'a>(v: &'a Value, name: &str, span: Span) -> Result<&'a str, LxError> {
    v.as_str()
        .ok_or_else(|| LxError::type_err(format!("{name} expects Str"), span))
}

pub(crate) fn check_empty(s: &str) -> bool {
    s.trim().is_empty()
}

fn bi_is_empty(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_empty", span)?;
    Ok(Value::Bool(check_empty(s)))
}

fn bi_is_too_short(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_too_short", span)?;
    let min = args[1]
        .as_int()
        .ok_or_else(|| LxError::type_err("audit.is_too_short: min_len must be Int", span))?;
    let min: usize = min
        .try_into()
        .map_err(|_| LxError::runtime("audit.is_too_short: min_len too large", span))?;
    Ok(Value::Bool(s.len() < min))
}

fn bi_is_repetitive(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_repetitive", span)?;
    let sentences: Vec<&str> = s
        .split(['.', '\n'])
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

pub(super) const HEDGING: &[&str] = &[
    "i think",
    "maybe",
    "possibly",
    "i'm not sure",
    "perhaps",
    "it might",
    "it could be",
    "i believe",
    "not entirely sure",
];

pub(crate) fn check_hedging(s: &str) -> bool {
    let lower = s.to_lowercase();
    HEDGING.iter().any(|h| lower.contains(h))
}

fn bi_is_hedging(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_hedging", span)?;
    Ok(Value::Bool(check_hedging(s)))
}

pub(super) const REFUSAL: &[&str] = &[
    "i can't",
    "i'm unable",
    "as an ai",
    "i cannot",
    "i'm not able",
    "i don't have the ability",
];

pub(crate) fn check_refusal(s: &str) -> bool {
    let lower = s.to_lowercase();
    REFUSAL.iter().any(|r| lower.contains(r))
}

fn bi_is_refusal(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.is_refusal", span)?;
    Ok(Value::Bool(check_refusal(s)))
}

fn bi_references_task(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let output = as_str_arg(&args[0], "audit.references_task(output)", span)?;
    let task = as_str_arg(&args[1], "audit.references_task(task)", span)?;
    Ok(Value::Bool(check_references_task(output, task)))
}

pub(super) fn check_references_task(output: &str, task: &str) -> bool {
    let (hits, total) = keyword_overlap(output, task, 3);
    if total == 0 {
        return true;
    }
    hits * 3 >= total
}

fn bi_files_exist(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::List(paths) = &args[0] else {
        return Err(LxError::type_err(
            "audit.files_exist expects List of Str",
            span,
        ));
    };
    for p in paths.iter() {
        let s = p
            .as_str()
            .ok_or_else(|| LxError::type_err("audit.files_exist: path must be Str", span))?;
        if !std::path::Path::new(s).exists() {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}

fn bi_has_diff(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let s = as_str_arg(&args[0], "audit.has_diff", span)?;
    let has =
        s.contains("diff --git") || s.contains("@@") || (s.contains("+++") && s.contains("---"));
    Ok(Value::Bool(has))
}

fn bi_rubric(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::List(_) = &args[0] else {
        return Err(LxError::type_err(
            "audit.rubric expects List of categories",
            span,
        ));
    };
    Ok(args[0].clone())
}
