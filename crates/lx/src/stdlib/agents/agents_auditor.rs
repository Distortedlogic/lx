use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::{AiOpts, RuntimeCtx};
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use num_bigint::BigInt;

use crate::record;
use crate::stdlib::ai;
use crate::value::Value;

const HEDGING: &[&str] = &[
    "i think", "maybe", "possibly", "i'm not sure", "perhaps",
    "it might", "it could be", "i believe", "not entirely sure",
];

const REFUSAL: &[&str] = &[
    "i can't", "i'm unable", "as an ai", "i cannot",
    "i'm not able", "i don't have the ability",
];

fn make_eval_category(name: &str, score: i64, passed: bool, detail: &str) -> Value {
    record! {
        "name" => Value::Str(Arc::from(name)),
        "score" => Value::Int(BigInt::from(score)),
        "passed" => Value::Bool(passed),
        "feedback" => Value::Str(Arc::from(detail)),
    }
}

fn build_eval_result(score: i64, passed: bool, categories: Vec<Value>, feedback: &str, failed: Vec<Value>) -> Value {
    record! {
        "score" => Value::Int(BigInt::from(score)),
        "passed" => Value::Bool(passed),
        "categories" => Value::List(Arc::new(categories)),
        "feedback" => Value::Str(Arc::from(feedback)),
        "failed" => Value::List(Arc::new(failed)),
    }
}

fn check_empty(s: &str) -> bool { s.trim().is_empty() }

fn check_refusal(s: &str) -> bool {
    let lower = s.to_lowercase();
    REFUSAL.iter().any(|r| lower.contains(r))
}

fn check_references_task(output: &str, task: &str) -> bool {
    let haystack_lower = output.to_lowercase();
    let keywords: Vec<String> = task.split_whitespace()
        .filter(|w| w.len() > 3).map(|w| w.to_lowercase()).collect();
    if keywords.is_empty() { return true; }
    let hits = keywords.iter().filter(|kw| haystack_lower.contains(kw.as_str())).count();
    hits * 3 >= keywords.len()
}

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("audit".into(), mk("auditor.audit", 1, bi_audit));
    m.insert(
        "quick_audit".into(),
        mk("auditor.quick_audit", 1, bi_quick_audit),
    );
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
    let output = fields
        .get("output")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("auditor: missing 'output' (Str)", span))?
        .to_string();
    let task = fields
        .get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("auditor: missing 'task' (Str)", span))?
        .to_string();
    let context = fields
        .get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let rubric = fields
        .get("rubric")
        .and_then(|v| v.as_list())
        .map(|l| {
            l.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(AuditFields {
        output,
        task,
        context,
        rubric,
    })
}

fn check_structural(fields: &AuditFields) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    if check_empty(&fields.output) {
        failures.push(("empty_output".into(), "Output is empty".into()));
    }
    if check_refusal(&fields.output) {
        failures.push(("refusal".into(), "Output contains refusal language".into()));
    }
    let lower = fields.output.to_lowercase();
    let hedge_count = HEDGING
        .iter()
        .filter(|h| lower.contains(**h))
        .count();
    if hedge_count >= 3 {
        failures.push((
            "excessive_hedging".into(),
            format!("Output contains {hedge_count} hedging phrases"),
        ));
    }
    if !fields.task.is_empty() && !check_references_task(&fields.output, &fields.task) {
        failures.push((
            "no_task_reference".into(),
            "Output doesn't reference key terms from the task".into(),
        ));
    }
    failures
}

fn structural_result(failures: &[(String, String)]) -> Value {
    let categories: Vec<Value> = failures
        .iter()
        .map(|(name, reason)| make_eval_category(name, 0, false, reason))
        .collect();
    let failed: Vec<Value> = failures
        .iter()
        .map(|(name, _)| Value::Str(Arc::from(name.as_str())))
        .collect();
    let feedback = failures
        .iter()
        .map(|(_, reason)| reason.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    build_eval_result(0, false, categories, &feedback, failed)
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

fn parse_llm_result(llm_response: &Value, span: Span) -> Result<Value, LxError> {
    match ai::parse_llm_json(llm_response, "auditor", span)? {
        Ok(jv) => extract_audit_from_json(&jv),
        Err(msg) => Ok(build_eval_result(0, false, vec![], &msg, vec![])),
    }
}

fn extract_audit_from_json(jv: &serde_json::Value) -> Result<Value, LxError> {
    let score = jv.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
    let feedback = jv.get("feedback").and_then(|v| v.as_str()).unwrap_or("");
    let mut categories = Vec::new();
    let mut failed = Vec::new();
    if let Some(cats) = jv.get("categories").and_then(|v| v.as_array()) {
        for cat in cats {
            let name = cat
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let passed = cat.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
            let reason = cat.get("reason").and_then(|v| v.as_str()).unwrap_or("");
            let cat_score = if passed { 100 } else { 0 };
            categories.push(make_eval_category(name, cat_score, passed, reason));
            if !passed {
                failed.push(Value::Str(Arc::from(name)));
            }
        }
    }
    let passed = score >= 70 && failed.is_empty();
    Ok(build_eval_result(
        score, passed, categories, feedback, failed,
    ))
}

fn bi_audit(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    let structural = check_structural(&fields);
    if !structural.is_empty() {
        return Ok(structural_result(&structural));
    }
    let system = build_system_prompt(&fields.rubric);
    let user = build_user_prompt(&fields);
    let opts = AiOpts {
        append_system: Some(system),
        max_turns: Some(1),
        tools: Some(vec![]),
        ..AiOpts::default()
    };
    let llm_result = ctx.ai.prompt(&user, &opts, span)?;
    parse_llm_result(&llm_result, span)
}

fn bi_quick_audit(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    let structural = check_structural(&fields);
    if structural.is_empty() {
        Ok(build_eval_result(
            100,
            true,
            vec![make_eval_category(
                "structure",
                100,
                true,
                "All structural checks passed",
            )],
            "Structural checks passed",
            vec![],
        ))
    } else {
        Ok(structural_result(&structural))
    }
}
