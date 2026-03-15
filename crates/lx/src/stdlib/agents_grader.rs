use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::{AiOpts, RuntimeCtx};
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::ai;
use crate::stdlib::audit;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("grade".into(), mk("grader.grade", 1, bi_grade));
    m.insert("quick_grade".into(), mk("grader.quick_grade", 1, bi_quick_grade));
    m
}

struct GradeFields {
    work: String,
    task: String,
    rubric: Vec<RubricCategory>,
    previous_grades: Vec<PrevGrade>,
    threshold: i64,
}

struct RubricCategory {
    name: String,
    description: String,
    weight: i64,
}

struct PrevGrade {
    name: String,
    score: i64,
    passed: bool,
}

fn extract_fields(args: &[Value], span: Span) -> Result<GradeFields, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("grader expects Record", span));
    };
    let work = fields.get("work")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("grader: missing 'work' (Str)", span))?
        .to_string();
    let task = fields.get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("grader: missing 'task' (Str)", span))?
        .to_string();
    let threshold = fields.get("threshold")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
        .unwrap_or(70);
    let rubric = fields.get("rubric")
        .and_then(|v| v.as_list())
        .map(|l| l.iter().filter_map(extract_rubric_cat).collect())
        .unwrap_or_default();
    let previous_grades = fields.get("previous_grades")
        .and_then(|v| v.as_list())
        .map(|l| l.iter().filter_map(extract_prev_grade).collect())
        .unwrap_or_default();
    Ok(GradeFields { work, task, rubric, previous_grades, threshold })
}

fn extract_rubric_cat(v: &Value) -> Option<RubricCategory> {
    let Value::Record(r) = v else { return None };
    Some(RubricCategory {
        name: r.get("name").and_then(|v| v.as_str())?.to_string(),
        description: r.get("description").and_then(|v| v.as_str())
            .unwrap_or("").to_string(),
        weight: r.get("weight").and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok()).unwrap_or(1),
    })
}

fn extract_prev_grade(v: &Value) -> Option<PrevGrade> {
    let Value::Record(r) = v else { return None };
    Some(PrevGrade {
        name: r.get("name").and_then(|v| v.as_str())?.to_string(),
        score: r.get("score").and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok()).unwrap_or(0),
        passed: r.get("passed").and_then(|v| v.as_bool()).unwrap_or(false),
    })
}

fn categories_to_evaluate<'a>(
    rubric: &'a [RubricCategory],
    prev: &[PrevGrade],
) -> (Vec<&'a RubricCategory>, Vec<(String, i64)>) {
    if prev.is_empty() {
        return (rubric.iter().collect(), Vec::new());
    }
    let mut to_eval = Vec::new();
    let mut kept = Vec::new();
    for cat in rubric {
        let prev_grade = prev.iter().find(|p| p.name == cat.name);
        match prev_grade {
            Some(pg) if pg.passed => kept.push((cat.name.clone(), pg.score)),
            _ => to_eval.push(cat),
        }
    }
    (to_eval, kept)
}

fn build_system_prompt(categories: &[&RubricCategory]) -> String {
    let mut p = String::from(
        "You are a grader scoring work against a rubric. \
         For each category, assign a score 0-100 and brief feedback.\n\n\
         Categories to evaluate:\n",
    );
    for (i, cat) in categories.iter().enumerate() {
        p.push_str(&format!("{}. {} — {}\n", i + 1, cat.name, cat.description));
    }
    p.push_str(
        "\nRespond with ONLY a JSON object, no markdown fences:\n\
         {\"categories\": [{\"name\": \"...\", \"score\": 0-100, \"feedback\": \"...\"}]}",
    );
    p
}

fn build_user_prompt(fields: &GradeFields) -> String {
    format!("TASK: {}\n\nWORK TO GRADE:\n{}", fields.task, fields.work)
}

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

fn assemble_result(
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
        let weight = rubric.iter().find(|c| &c.name == name)
            .map(|c| c.weight).unwrap_or(1);
        all_cats.push(audit::make_eval_category(name, *score, true, "previously passed"));
        total_score += score * weight;
        total_weight += weight;
    }
    if let Some(cats) = jv.get("categories").and_then(|v| v.as_array()) {
        for cat in cats {
            let name = cat.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let score = cat.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
            let fb = cat.get("feedback").and_then(|v| v.as_str()).unwrap_or("");
            let weight = rubric.iter().find(|c| c.name == name)
                .map(|c| c.weight).unwrap_or(1);
            let passed = score >= 70;
            all_cats.push(audit::make_eval_category(name, score, passed, fb));
            total_score += score * weight;
            total_weight += weight;
            if !passed {
                failed.push(Value::Str(Arc::from(name)));
            }
        }
    }
    let final_score = if total_weight > 0 { total_score / total_weight } else { 0 };
    let overall = final_score >= threshold && failed.is_empty();
    let feedback_parts: Vec<String> = failed.iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let feedback = if feedback_parts.is_empty() {
        "All categories passed".to_string()
    } else {
        format!("Failed: {}", feedback_parts.join(", "))
    };
    Ok(audit::build_eval_result(final_score, overall, all_cats, &feedback, failed))
}

fn bi_grade(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    if fields.rubric.is_empty() {
        return Ok(audit::build_eval_result(0, false, vec![], "No rubric categories provided", vec![]));
    }
    let (to_eval, kept) = categories_to_evaluate(&fields.rubric, &fields.previous_grades);
    if to_eval.is_empty() {
        return assemble_result(&serde_json::json!({"categories": []}),
            &kept, &fields.rubric, fields.threshold).map_err(|_| unreachable!());
    }
    let system = build_system_prompt(&to_eval);
    let user = build_user_prompt(&fields);
    let opts = AiOpts {
        system: Some(system),
        max_turns: Some(1),
        ..AiOpts::default()
    };
    let llm_result = ctx.ai.prompt(&user, &opts, span)?;
    parse_llm_result(&llm_result, &kept, &fields.rubric, fields.threshold, span)
}

fn bi_quick_grade(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    if fields.rubric.is_empty() {
        return Ok(audit::build_eval_result(0, false, vec![], "No rubric categories", vec![]));
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
        categories.push(audit::make_eval_category(&cat.name, score, passed,
            if passed { "keyword match" } else { "low keyword overlap" }));
        if !passed {
            failed.push(Value::Str(Arc::from(cat.name.as_str())));
        }
        total_score += score * cat.weight;
        total_weight += cat.weight;
    }
    let final_score = if total_weight > 0 { total_score / total_weight } else { 0 };
    let overall = final_score >= fields.threshold && failed.is_empty();
    let feedback = if failed.is_empty() {
        "All categories passed".to_string()
    } else {
        let names: Vec<&str> = failed.iter()
            .filter_map(|v| v.as_str()).collect();
        format!("Failed: {}", names.join(", "))
    };
    Ok(audit::build_eval_result(final_score, overall, categories, &feedback, failed))
}
