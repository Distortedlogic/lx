use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::{AiOpts, RuntimeCtx};
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::stdlib::ai;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("review".into(), mk("reviewer.review", 1, bi_review));
    m.insert(
        "quick_review".into(),
        mk("reviewer.quick_review", 1, bi_quick_review),
    );
    m
}

struct ReviewFields {
    transcript: String,
    task: String,
    focus: Vec<String>,
}

fn extract_fields(args: &[Value], span: Span) -> Result<ReviewFields, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("reviewer expects Record", span));
    };
    let transcript = fields
        .get("transcript")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("reviewer: missing 'transcript' (Str)", span))?
        .to_string();
    let task = fields
        .get("task")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let focus = fields
        .get("focus")
        .and_then(|v| v.as_list())
        .map(|l| {
            l.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(ReviewFields {
        transcript,
        task,
        focus,
    })
}

fn make_pattern(kind: &str, description: &str, confidence: f64) -> Value {
    record! {
        "kind" => Value::Str(Arc::from(kind)),
        "description" => Value::Str(Arc::from(description)),
        "confidence" => Value::Float(confidence),
    }
}

fn make_tagged_record(kind: &str, fields: &[(&str, Value)]) -> Value {
    let mut f = IndexMap::new();
    f.insert("kind".into(), Value::Str(Arc::from(kind)));
    for (key, val) in fields {
        f.insert((*key).into(), val.clone());
    }
    Value::Record(Arc::new(f))
}

fn build_result(
    patterns: Vec<Value>,
    mistakes: Vec<Value>,
    facts: Vec<Value>,
    summary: &str,
) -> Value {
    let pattern_count = patterns.len() as i64;
    let mistake_count = mistakes.len() as i64;
    record! {
        "patterns" => Value::List(Arc::new(patterns)),
        "mistakes" => Value::List(Arc::new(mistakes)),
        "facts" => Value::List(Arc::new(facts)),
        "summary" => Value::Str(Arc::from(summary)),
        "pattern_count" => Value::Int(BigInt::from(pattern_count)),
        "mistake_count" => Value::Int(BigInt::from(mistake_count)),
    }
}

fn build_system_prompt(focus: &[String]) -> String {
    let mut p = String::from(
        "You are a transcript reviewer. Analyze a completed agent session.\n\n\
         Extract:\n\
         1. patterns: Strategies/approaches that worked (with confidence 0-1)\n\
         2. mistakes: Errors or inefficiencies to avoid (with lesson learned)\n\
         3. facts: Environment or domain facts discovered\n\n",
    );
    if !focus.is_empty() {
        p.push_str("Focus areas: ");
        p.push_str(&focus.join(", "));
        p.push('\n');
    }
    p.push_str(
        "\nRespond with ONLY a JSON object, no markdown fences:\n\
         {\"patterns\": [{\"description\": \"...\", \"confidence\": 0.8}],\n\
          \"mistakes\": [{\"description\": \"...\", \"lesson\": \"...\"}],\n\
          \"facts\": [{\"description\": \"...\"}],\n\
          \"summary\": \"...\"}",
    );
    p
}

fn build_user_prompt(fields: &ReviewFields) -> String {
    let mut p = String::new();
    if !fields.task.is_empty() {
        p.push_str(&format!("ORIGINAL TASK: {}\n\n", fields.task));
    }
    p.push_str(&format!("TRANSCRIPT:\n{}", fields.transcript));
    p
}

fn parse_llm_result(llm_response: &Value, span: Span) -> Result<Value, LxError> {
    let jv = match ai::parse_llm_json(llm_response, "reviewer", span)? {
        Ok(jv) => jv,
        Err(msg) => return Ok(build_result(vec![], vec![], vec![], &msg)),
    };
    let mut patterns = Vec::new();
    let mut mistakes = Vec::new();
    let mut facts = Vec::new();
    if let Some(arr) = jv.get("patterns").and_then(|v| v.as_array()) {
        for p in arr {
            let desc = p.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let conf = p.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);
            patterns.push(make_pattern("success", desc, conf));
        }
    }
    if let Some(arr) = jv.get("mistakes").and_then(|v| v.as_array()) {
        for m in arr {
            let desc = m.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let lesson = m.get("lesson").and_then(|v| v.as_str()).unwrap_or("");
            mistakes.push(make_tagged_record(
                "mistake",
                &[
                    ("description", Value::Str(Arc::from(desc))),
                    ("lesson", Value::Str(Arc::from(lesson))),
                ],
            ));
        }
    }
    if let Some(arr) = jv.get("facts").and_then(|v| v.as_array()) {
        for fact in arr {
            let desc = fact
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            facts.push(make_tagged_record(
                "fact",
                &[("description", Value::Str(Arc::from(desc)))],
            ));
        }
    }
    let summary = jv.get("summary").and_then(|v| v.as_str()).unwrap_or("");
    Ok(build_result(patterns, mistakes, facts, summary))
}

fn bi_review(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    static WARNED: std::sync::Once = std::sync::Once::new();
    WARNED.call_once(|| {
        eprintln!("[DEPRECATED] std/agents/reviewer is deprecated — use pkg/ai/reviewer instead");
    });
    let fields = extract_fields(args, span)?;
    if fields.transcript.trim().is_empty() {
        return Ok(build_result(vec![], vec![], vec![], "Empty transcript"));
    }
    let system = build_system_prompt(&fields.focus);
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

fn extract_structural(transcript: &str) -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    let mut patterns = Vec::new();
    let mut mistakes = Vec::new();
    let mut facts = Vec::new();
    let lines: Vec<&str> = transcript.lines().collect();
    let has_error = lines.iter().any(|l| {
        let lower = l.to_lowercase();
        lower.contains("error") || lower.contains("failed") || lower.contains("bug")
    });
    let has_retry = lines.iter().any(|l| l.to_lowercase().contains("retry"));
    let has_success = lines.iter().any(|l| {
        let lower = l.to_lowercase();
        lower.contains("success") || lower.contains("passed") || lower.contains("complete")
    });
    if has_error && has_success {
        patterns.push(make_pattern("recovery", "Recovered from errors", 0.6));
    }
    if has_retry {
        patterns.push(make_pattern("persistence", "Used retry strategy", 0.5));
    }
    if has_error {
        facts.push(make_tagged_record(
            "observation",
            &[(
                "description",
                Value::Str(Arc::from("Errors occurred during execution")),
            )],
        ));
    }
    let duplicate_lines: usize = {
        let mut seen = std::collections::HashSet::new();
        lines
            .iter()
            .filter(|l| l.len() > 10 && !seen.insert(**l))
            .count()
    };
    if duplicate_lines > 3 {
        mistakes.push(make_tagged_record(
            "mistake",
            &[
                (
                    "description",
                    Value::Str(Arc::from("Excessive repeated actions")),
                ),
                ("lesson", Value::Str(Arc::from("Detect loops earlier"))),
            ],
        ));
    }
    (patterns, mistakes, facts)
}

fn bi_quick_review(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    static WARNED: std::sync::Once = std::sync::Once::new();
    WARNED.call_once(|| {
        eprintln!("[DEPRECATED] std/agents/reviewer is deprecated — use pkg/ai/reviewer instead");
    });
    let fields = extract_fields(args, span)?;
    if fields.transcript.trim().is_empty() {
        return Ok(build_result(vec![], vec![], vec![], "Empty transcript"));
    }
    let (patterns, mistakes, facts) = extract_structural(&fields.transcript);
    let summary = format!(
        "{} patterns, {} mistakes, {} facts extracted structurally",
        patterns.len(),
        mistakes.len(),
        facts.len(),
    );
    Ok(build_result(patterns, mistakes, facts, &summary))
}
