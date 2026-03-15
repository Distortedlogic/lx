use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::ai;
use crate::stdlib::audit::{HEDGING, REFUSAL};
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("audit".into(), mk("auditor.audit", 1, bi_audit));
    m.insert("quick_audit".into(), mk("auditor.quick_audit", 1, bi_quick_audit));
    m
}

struct AuditFields {
    output: String,
    task: String,
    context: String,
    rubric: Vec<String>,
}

fn extract_fields(args: &[Value], span: Span) -> Result<AuditFields, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("auditor expects Record", span));
    };
    let output = fields.get("output")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("auditor: missing 'output' (Str)", span))?
        .to_string();
    let task = fields.get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("auditor: missing 'task' (Str)", span))?
        .to_string();
    let context = fields.get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let rubric = fields.get("rubric")
        .and_then(|v| v.as_list())
        .map(|l| l.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    Ok(AuditFields { output, task, context, rubric })
}

fn check_structural(fields: &AuditFields) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    if fields.output.trim().is_empty() {
        failures.push(("empty_output".into(), "Output is empty".into()));
    }
    let lower = fields.output.to_lowercase();
    if REFUSAL.iter().any(|r| lower.contains(r)) {
        failures.push(("refusal".into(), "Output contains refusal language".into()));
    }
    let hedge_count = HEDGING.iter().filter(|h| lower.contains(**h)).count();
    if hedge_count >= 3 {
        failures.push((
            "excessive_hedging".into(),
            format!("Output contains {hedge_count} hedging phrases"),
        ));
    }
    if !fields.task.is_empty() {
        let output_lower = fields.output.to_lowercase();
        let keywords: Vec<&str> = fields.task.split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();
        if !keywords.is_empty() {
            let hits = keywords.iter()
                .filter(|kw| output_lower.contains(&kw.to_lowercase()))
                .count();
            if hits * 3 < keywords.len() {
                failures.push((
                    "no_task_reference".into(),
                    "Output doesn't reference key terms from the task".into(),
                ));
            }
        }
    }
    failures
}

fn make_category(name: &str, passed: bool, reason: &str) -> Value {
    let mut cat = IndexMap::new();
    cat.insert("name".into(), Value::Str(Arc::from(name)));
    cat.insert("passed".into(), Value::Bool(passed));
    cat.insert("reason".into(), Value::Str(Arc::from(reason)));
    Value::Record(Arc::new(cat))
}

fn build_result(
    score: i64,
    passed: bool,
    categories: Vec<Value>,
    feedback: &str,
    failed: Vec<Value>,
) -> Value {
    let mut r = IndexMap::new();
    r.insert("score".into(), Value::Int(BigInt::from(score)));
    r.insert("passed".into(), Value::Bool(passed));
    r.insert("categories".into(), Value::List(Arc::new(categories)));
    r.insert("feedback".into(), Value::Str(Arc::from(feedback)));
    r.insert("failed".into(), Value::List(Arc::new(failed)));
    Value::Record(Arc::new(r))
}

fn structural_result(failures: &[(String, String)]) -> Value {
    let categories: Vec<Value> = failures.iter()
        .map(|(name, reason)| make_category(name, false, reason))
        .collect();
    let failed: Vec<Value> = failures.iter()
        .map(|(name, _)| Value::Str(Arc::from(name.as_str())))
        .collect();
    let feedback = failures.iter()
        .map(|(_, reason)| reason.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    build_result(0, false, categories, &feedback, failed)
}

fn build_system_prompt(rubric: &[String]) -> String {
    let mut p = String::from(
        "You are a quality auditor. Evaluate whether agent output addresses a task.\n\n\
         Criteria:\n\
         1. relevance: Does the output address the actual task?\n\
         2. context_usage: Does the output use provided context, not assumptions?\n\
         3. completeness: Is the output complete?\n\
         4. accuracy: Does the output avoid hallucinating facts, files, or APIs?\n",
    );
    for (i, item) in rubric.iter().enumerate() {
        p.push_str(&format!("{}. {item}\n", i + 5));
    }
    p.push_str(
        "\nRespond with ONLY a JSON object, no markdown fences:\n\
         {\"categories\": [{\"name\": \"...\", \"passed\": true, \"reason\": \"...\"}], \
         \"score\": 0-100, \"feedback\": \"...\"}",
    );
    p
}

fn build_user_prompt(fields: &AuditFields) -> String {
    let mut p = format!("TASK: {}\n\n", fields.task);
    if !fields.context.is_empty() {
        p.push_str(&format!("CONTEXT:\n{}\n\n", fields.context));
    }
    p.push_str(&format!("OUTPUT TO EVALUATE:\n{}", fields.output));
    p
}

fn parse_llm_result(llm_response: Value, span: Span) -> Result<Value, LxError> {
    let text = match llm_response {
        Value::Ok(inner) => match *inner {
            Value::Record(ref fields) => fields.get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            Value::Str(ref s) => s.to_string(),
            _ => return Ok(build_result(
                0, false, vec![], "LLM returned unexpected format", vec![],
            )),
        },
        Value::Err(e) => {
            let msg = match *e {
                Value::Str(ref s) => s.to_string(),
                Value::Record(ref r) => r.get("msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string(),
                _ => "LLM error".to_string(),
            };
            return Ok(build_result(
                0, false, vec![], &format!("LLM error: {msg}"), vec![],
            ));
        }
        _ => return Ok(build_result(
            0, false, vec![], "LLM returned unexpected value", vec![],
        )),
    };
    let jv: serde_json::Value = serde_json::from_str(text.trim())
        .or_else(|_| {
            let stripped = text.trim()
                .strip_prefix("```json").or_else(|| text.trim().strip_prefix("```"))
                .and_then(|s| s.strip_suffix("```"))
                .unwrap_or(&text);
            serde_json::from_str(stripped.trim())
        })
        .map_err(|e| LxError::runtime(
            format!("auditor: cannot parse LLM JSON: {e}"), span,
        ))?;
    extract_audit_from_json(&jv)
}

fn extract_audit_from_json(jv: &serde_json::Value) -> Result<Value, LxError> {
    let score = jv.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
    let feedback = jv.get("feedback").and_then(|v| v.as_str()).unwrap_or("");
    let mut categories = Vec::new();
    let mut failed = Vec::new();
    if let Some(cats) = jv.get("categories").and_then(|v| v.as_array()) {
        for cat in cats {
            let name = cat.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let passed = cat.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
            let reason = cat.get("reason").and_then(|v| v.as_str()).unwrap_or("");
            categories.push(make_category(name, passed, reason));
            if !passed {
                failed.push(Value::Str(Arc::from(name)));
            }
        }
    }
    let passed = score >= 70 && failed.is_empty();
    Ok(build_result(score, passed, categories, feedback, failed))
}

fn bi_audit(args: &[Value], span: Span) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    let structural = check_structural(&fields);
    if !structural.is_empty() {
        return Ok(structural_result(&structural));
    }
    let system = build_system_prompt(&fields.rubric);
    let user = build_user_prompt(&fields);
    let opts = ai::Opts {
        system: Some(system),
        max_turns: Some(1),
        ..ai::default_opts()
    };
    let llm_result = ai::run_claude(&user, &opts, span)?;
    parse_llm_result(llm_result, span)
}

fn bi_quick_audit(args: &[Value], span: Span) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    let structural = check_structural(&fields);
    if structural.is_empty() {
        Ok(build_result(100, true, vec![
            make_category("structure", true, "All structural checks passed"),
        ], "Structural checks passed", vec![]))
    } else {
        Ok(structural_result(&structural))
    }
}
