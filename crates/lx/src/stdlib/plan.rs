use std::collections::HashSet;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::step_deps;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("run".into(), mk("plan.run", 3, bi_run));
    m.insert("replan".into(), mk("plan.replan", 1, bi_replan));
    m.insert("abort".into(), mk("plan.abort", 1, bi_abort));
    m.insert(
        "insert_after".into(),
        mk("plan.insert_after", 2, bi_insert_after),
    );
    m.insert("continue".into(), make_action("continue"));
    m.insert("skip".into(), make_action("skip"));
    m
}

fn make_action(action: &str) -> Value {
    record! {
        "__action" => Value::Str(Arc::from(action)),
    }
}

fn bi_replan(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::List(_) = &args[0] else {
        return Err(LxError::type_err("plan.replan expects List of steps", span));
    };
    Ok(record! {
        "__action" => Value::Str(Arc::from("replan")),
        "steps" => args[0].clone(),
    })
}

fn bi_abort(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let reason = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("plan.abort expects Str reason", span))?;
    Ok(record! {
        "__action" => Value::Str(Arc::from("abort")),
        "reason" => Value::Str(Arc::from(reason)),
    })
}

fn bi_insert_after(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let after_id = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("plan.insert_after: first arg must be Str", span))?;
    let Value::List(_) = &args[1] else {
        return Err(LxError::type_err(
            "plan.insert_after: second arg must be List",
            span,
        ));
    };
    Ok(record! {
        "__action" => Value::Str(Arc::from("insert_after")),
        "after" => Value::Str(Arc::from(after_id)),
        "steps" => args[1].clone(),
    })
}

fn call2(
    f: &Value,
    a: Value,
    b: Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let partial = call_value_sync(f, a, span, ctx)?;
    call_value_sync(&partial, b, span, ctx)
}

fn call3(
    f: &Value,
    a: Value,
    b: Value,
    c: Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let p1 = call_value_sync(f, a, span, ctx)?;
    let p2 = call_value_sync(&p1, b, span, ctx)?;
    call_value_sync(&p2, c, span, ctx)
}

fn build_context(completed_results: &[(String, Value)]) -> Value {
    let items: Vec<Value> = completed_results
        .iter()
        .map(|(id, result)| {
            record! {
                "id" => Value::Str(Arc::from(id.as_str())),
                "result" => result.clone(),
            }
        })
        .collect();
    record! {
        "completed" => Value::List(Arc::new(items)),
    }
}

fn build_plan_state(
    completed_results: &[(String, Value)],
    remaining: &[Value],
    current: &Value,
) -> Value {
    let completed: Vec<Value> = completed_results
        .iter()
        .map(|(id, result)| {
            record! {
                "id" => Value::Str(Arc::from(id.as_str())),
                "result" => result.clone(),
            }
        })
        .collect();
    record! {
        "completed" => Value::List(Arc::new(completed)),
        "remaining" => Value::List(Arc::new(remaining.to_vec())),
        "current" => current.clone(),
    }
}

fn get_action(v: &Value) -> Option<&str> {
    match v {
        Value::Record(r) => r.get("__action").and_then(|v| v.as_str()),
        _ => None,
    }
}

fn bi_run(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::List(initial_steps) = &args[0] else {
        return Err(LxError::type_err(
            "plan.run: first arg must be List of steps",
            span,
        ));
    };
    let executor = &args[1];
    let on_step = &args[2];
    let mut remaining: Vec<Value> = initial_steps.as_ref().clone();
    let mut completed_results: Vec<(String, Value)> = Vec::new();
    let mut completed_ids: HashSet<String> = HashSet::new();
    loop {
        if remaining.is_empty() {
            break;
        }
        let Some(idx) = step_deps::next_ready(&remaining, &completed_ids) else {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                "plan: cycle or unmet dependencies in remaining steps",
            )))));
        };
        let step = remaining.remove(idx);
        let sid = step_deps::step_id(&step).unwrap_or("unknown").to_string();
        let context = build_context(&completed_results);
        let result = call2(executor, step.clone(), context, span, ctx)?;
        let plan_state = build_plan_state(&completed_results, &remaining, &step);
        let action = call3(on_step, step.clone(), result.clone(), plan_state, span, ctx)?;
        completed_results.push((sid.clone(), result));
        completed_ids.insert(sid.clone());
        match get_action(&action) {
            Some("continue") | None => {}
            Some("skip") => {
                let to_skip = find_successors(&sid, &remaining);
                remaining.retain(|s| step_deps::step_id(s).is_none_or(|id| !to_skip.contains(id)));
            }
            Some("abort") => {
                let reason = match &action {
                    Value::Record(r) => r
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("aborted"),
                    _ => "aborted",
                };
                return Ok(Value::Err(Box::new(Value::Str(Arc::from(reason)))));
            }
            Some("replan") => {
                if let Value::Record(r) = &action
                    && let Some(Value::List(new_steps)) = r.get("steps")
                {
                    remaining = new_steps.as_ref().clone();
                }
            }
            Some("insert_after") => {
                if let Value::Record(r) = &action
                    && let Some(Value::List(new_steps)) = r.get("steps")
                {
                    for (i, ns) in new_steps.iter().enumerate() {
                        remaining.insert(i, ns.clone());
                    }
                }
            }
            Some(other) => {
                return Err(LxError::runtime(
                    format!("plan: unknown action '{other}'"),
                    span,
                ));
            }
        }
    }
    let results: Vec<Value> = completed_results
        .iter()
        .map(|(id, result)| {
            record! {
                "id" => Value::Str(Arc::from(id.as_str())),
                "result" => result.clone(),
            }
        })
        .collect();
    Ok(Value::Ok(Box::new(Value::List(Arc::new(results)))))
}

fn find_successors(current_id: &str, remaining: &[Value]) -> HashSet<String> {
    let mut to_skip = HashSet::new();
    to_skip.insert(current_id.to_string());
    let mut changed = true;
    while changed {
        changed = false;
        for step in remaining {
            let sid = step_deps::step_id(step).unwrap_or("").to_string();
            if to_skip.contains(&sid) {
                continue;
            }
            if step_deps::step_deps(step)
                .iter()
                .any(|d| to_skip.contains(d))
            {
                to_skip.insert(sid);
                changed = true;
            }
        }
    }
    to_skip.remove(current_id);
    to_skip
}
