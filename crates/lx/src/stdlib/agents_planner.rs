use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::{AiOpts, RuntimeCtx};
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::ai;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("plan".into(), mk("planner.plan", 1, bi_plan));
    m.insert(
        "quick_plan".into(),
        mk("planner.quick_plan", 1, bi_quick_plan),
    );
    m
}

struct PlanFields {
    task: String,
    context: String,
    max_steps: i64,
    constraints: Vec<String>,
}

fn extract_fields(args: &[Value], span: Span) -> Result<PlanFields, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("planner expects Record", span));
    };
    let task = fields
        .get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("planner: missing 'task' (Str)", span))?
        .to_string();
    let context = fields
        .get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let max_steps = fields
        .get("max_steps")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
        .unwrap_or(10);
    let constraints = fields
        .get("constraints")
        .and_then(|v| v.as_list())
        .map(|l| {
            l.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(PlanFields {
        task,
        context,
        max_steps,
        constraints,
    })
}

fn make_step(id: i64, title: &str, description: &str, deps: Vec<Value>, complexity: &str) -> Value {
    let mut s = IndexMap::new();
    s.insert("id".into(), Value::Int(BigInt::from(id)));
    s.insert("title".into(), Value::Str(Arc::from(title)));
    s.insert("description".into(), Value::Str(Arc::from(description)));
    s.insert("deps".into(), Value::List(Arc::new(deps)));
    s.insert("complexity".into(), Value::Str(Arc::from(complexity)));
    s.insert("status".into(), Value::Str(Arc::from("pending")));
    Value::Record(Arc::new(s))
}

fn build_result(steps: Vec<Value>, task: &str) -> Value {
    let mut r = IndexMap::new();
    let count = steps.len() as i64;
    r.insert("steps".into(), Value::List(Arc::new(steps)));
    r.insert("task".into(), Value::Str(Arc::from(task)));
    r.insert("step_count".into(), Value::Int(BigInt::from(count)));
    Value::Record(Arc::new(r))
}

fn build_system_prompt(max_steps: i64, constraints: &[String]) -> String {
    let mut p = format!(
        "You are a task planner. Decompose a complex task into ordered subtasks.\n\n\
         Rules:\n\
         - Maximum {max_steps} steps\n\
         - Each step must have a clear, actionable title and description\n\
         - Specify dependencies as step IDs (0-indexed)\n\
         - Estimate complexity as 'low', 'medium', or 'high'\n",
    );
    for c in constraints {
        p.push_str(&format!("- Constraint: {c}\n"));
    }
    p.push_str(
        "\nRespond with ONLY a JSON object, no markdown fences:\n\
         {\"steps\": [{\"id\": 0, \"title\": \"...\", \"description\": \"...\", \
         \"deps\": [], \"complexity\": \"low|medium|high\"}]}",
    );
    p
}

fn build_user_prompt(fields: &PlanFields) -> String {
    let mut p = format!("TASK: {}\n", fields.task);
    if !fields.context.is_empty() {
        p.push_str(&format!("\nCONTEXT:\n{}\n", fields.context));
    }
    p
}

fn parse_llm_result(llm_response: &Value, task: &str, span: Span) -> Result<Value, LxError> {
    let Ok(jv) = ai::parse_llm_json(llm_response, "planner", span)? else {
        return Ok(build_result(vec![], task));
    };
    let mut steps = Vec::new();
    if let Some(arr) = jv.get("steps").and_then(|v| v.as_array()) {
        for step in arr {
            let id = step
                .get("id")
                .and_then(|v| v.as_i64())
                .unwrap_or(steps.len() as i64);
            let title = step.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let desc = step
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let complexity = step
                .get("complexity")
                .and_then(|v| v.as_str())
                .unwrap_or("medium");
            let deps: Vec<Value> = step
                .get("deps")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_i64())
                        .map(|n| Value::Int(BigInt::from(n)))
                        .collect()
                })
                .unwrap_or_default();
            steps.push(make_step(id, title, desc, deps, complexity));
        }
    }
    Ok(build_result(steps, task))
}

fn bi_plan(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    if fields.task.trim().is_empty() {
        return Ok(build_result(vec![], ""));
    }
    let system = build_system_prompt(fields.max_steps, &fields.constraints);
    let user = build_user_prompt(&fields);
    let opts = AiOpts {
        system: Some(system),
        max_turns: Some(1),
        ..AiOpts::default()
    };
    let llm_result = ctx.ai.prompt(&user, &opts, span)?;
    parse_llm_result(&llm_result, &fields.task, span)
}

fn split_sentences(text: &str) -> Vec<String> {
    text.split(['.', ';', '\n'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() > 5)
        .collect()
}

fn bi_quick_plan(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    if fields.task.trim().is_empty() {
        return Ok(build_result(vec![], ""));
    }
    let sentences = split_sentences(&fields.task);
    let max = fields.max_steps as usize;
    let mut steps = Vec::new();
    if sentences.len() <= 1 {
        steps.push(make_step(0, &fields.task, &fields.task, vec![], "medium"));
    } else {
        for (i, sentence) in sentences.iter().take(max).enumerate() {
            let deps = if i > 0 {
                vec![Value::Int(BigInt::from(i as i64 - 1))]
            } else {
                vec![]
            };
            steps.push(make_step(i as i64, sentence, sentence, deps, "medium"));
        }
    }
    Ok(build_result(steps, &fields.task))
}
