use std::sync::Arc;

use crate::backends::{AiOpts, RuntimeCtx};
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::ai;
use crate::stdlib::audit;
use crate::value::Value;

use super::{
    RubricCategory, build_system_prompt, build_user_prompt, categories_to_evaluate, extract_fields,
};

fn parse_llm_result(
    llm_response: &Value,
    kept: &[(String, i64)],
    rubric: &[RubricCategory],
    threshold: i64,
    span: Span,
) -> Result<Value, LxError> {
    match ai::parse_llm_json(llm_response, "grader", span)? {
        Ok(jv) => assemble_result(&jv, kept, rubric, threshold),
        Err(msg) => Ok(audit::build_eval_result(0, false, vec![], &msg, vec![])),
    }
}

pub(super) fn assemble_result(
    jv: &serde_json::Value,
    kept: &[(String, i64)],
    rubric: &[RubricCategory],
    threshold: i64,
) -> Result<Value, LxError> {
    let mut all_cats = Vec::new();
    let mut failed = Vec::new();
    let mut total_score: i64 = 0;
    let mut total_weight: i64 = 0;
    for (name, score) in kept {
        let weight = rubric
            .iter()
            .find(|c| &c.name == name)
            .map(|c| c.weight)
            .unwrap_or(1);
        all_cats.push(audit::make_eval_category(
            name,
            *score,
            true,
            "previously passed",
        ));
        total_score += score * weight;
        total_weight += weight;
    }
    if let Some(cats) = jv.get("categories").and_then(|v| v.as_array()) {
        for cat in cats {
            let name = cat
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let score = cat.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
            let fb = cat.get("feedback").and_then(|v| v.as_str()).unwrap_or("");
            let weight = rubric
                .iter()
                .find(|c| c.name == name)
                .map(|c| c.weight)
                .unwrap_or(1);
            let passed = score >= 70;
            all_cats.push(audit::make_eval_category(name, score, passed, fb));
            total_score += score * weight;
            total_weight += weight;
            if !passed {
                failed.push(Value::Str(Arc::from(name)));
            }
        }
    }
    let final_score = if total_weight > 0 {
        total_score / total_weight
    } else {
        0
    };
    let overall = final_score >= threshold && failed.is_empty();
    let feedback_parts: Vec<String> = failed
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let feedback = if feedback_parts.is_empty() {
        "All categories passed".to_string()
    } else {
        format!("Failed: {}", feedback_parts.join(", "))
    };
    Ok(audit::build_eval_result(
        final_score,
        overall,
        all_cats,
        &feedback,
        failed,
    ))
}

pub(super) fn bi_grade(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    if fields.rubric.is_empty() {
        return Ok(audit::build_eval_result(
            0,
            false,
            vec![],
            "No rubric categories provided",
            vec![],
        ));
    }
    let (to_eval, kept) = categories_to_evaluate(&fields.rubric, &fields.previous_grades);
    if to_eval.is_empty() {
        return assemble_result(
            &serde_json::json!({"categories": []}),
            &kept,
            &fields.rubric,
            fields.threshold,
        );
    }
    let system = build_system_prompt(&to_eval);
    let user = build_user_prompt(&fields);
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "categories": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "score": {"type": "integer", "minimum": 0, "maximum": 100},
                        "feedback": {"type": "string"}
                    },
                    "required": ["name", "score", "feedback"]
                }
            }
        },
        "required": ["categories"]
    });
    let opts = AiOpts {
        append_system: Some(system),
        disable_tools: true,
        json_schema: Some(schema.to_string()),
        ..AiOpts::default()
    };
    let llm_result = ctx.ai.prompt(&user, &opts, span)?;
    parse_llm_result(&llm_result, &kept, &fields.rubric, fields.threshold, span)
}

pub(super) fn bi_quick_grade(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    if fields.rubric.is_empty() {
        return Ok(audit::build_eval_result(
            0,
            false,
            vec![],
            "No rubric categories",
            vec![],
        ));
    }
    let mut categories = Vec::new();
    let mut failed = Vec::new();
    let mut total_score: i64 = 0;
    let mut total_weight: i64 = 0;
    for cat in &fields.rubric {
        let (hits, total) = audit::keyword_overlap(&fields.work, &cat.description, 3);
        let score: i64 = if total == 0 {
            50
        } else {
            ((hits as f64 / total as f64) * 100.0) as i64
        };
        let passed = score >= 50;
        categories.push(audit::make_eval_category(
            &cat.name,
            score,
            passed,
            if passed {
                "keyword match"
            } else {
                "low keyword overlap"
            },
        ));
        if !passed {
            failed.push(Value::Str(Arc::from(cat.name.as_str())));
        }
        total_score += score * cat.weight;
        total_weight += cat.weight;
    }
    let final_score = if total_weight > 0 {
        total_score / total_weight
    } else {
        0
    };
    let overall = final_score >= fields.threshold && failed.is_empty();
    let feedback = if failed.is_empty() {
        "All categories passed".to_string()
    } else {
        let names: Vec<&str> = failed.iter().filter_map(|v| v.as_str()).collect();
        format!("Failed: {}", names.join(", "))
    };
    Ok(audit::build_eval_result(
        final_score,
        overall,
        categories,
        &feedback,
        failed,
    ))
}
