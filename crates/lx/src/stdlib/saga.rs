use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use indexmap::IndexMap;

use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("run".into(), mk("saga.run", 1, bi_run));
    m.insert("run_with".into(), mk("saga.run_with", 2, bi_run_with));
    m.insert("define".into(), mk("saga.define", 1, bi_define));
    m.insert("execute".into(), mk("saga.execute", 2, bi_execute));
    m
}

#[derive(Default)]
struct SagaOpts {
    on_compensate: Option<Value>,
    timeout_secs: Option<u64>,
    max_retries: usize,
}

fn parse_opts(v: &Value, span: Span) -> Result<SagaOpts, LxError> {
    let Value::Record(r) = v else {
        return Err(LxError::type_err("saga options must be a Record", span));
    };
    Ok(SagaOpts {
        on_compensate: r.get("on_compensate").cloned(),
        timeout_secs: r.get("timeout").and_then(|v| v.as_int()).and_then(|n| n.try_into().ok()),
        max_retries: r.get("max_retries")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .unwrap_or(0),
    })
}

fn step_id(step: &Value) -> Option<&str> {
    match step {
        Value::Record(r) => r.get("id").and_then(|v| v.as_str()),
        _ => None,
    }
}

fn step_deps(step: &Value) -> Vec<String> {
    match step {
        Value::Record(r) => r.get("depends")
            .and_then(|v| v.as_list())
            .map(|l| l.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        _ => vec![],
    }
}

fn next_ready(remaining: &[Value], completed: &HashSet<String>) -> Option<usize> {
    remaining.iter().position(|step| {
        step_deps(step).iter().all(|d| completed.contains(d))
    })
}

fn build_prev(completed: &[(String, Value)]) -> Value {
    let mut fields = IndexMap::new();
    for (id, result) in completed {
        fields.insert(id.clone(), result.clone());
    }
    Value::Record(Arc::new(fields))
}

fn step_fn(step: &Value, field: &str) -> Option<Value> {
    match step {
        Value::Record(r) => r.get(field).cloned(),
        _ => None,
    }
}

fn try_step(do_fn: &Value, prev: &Value, retries: usize, span: Span) -> Result<Value, Value> {
    for attempt in 0..=retries {
        let last = attempt == retries;
        match call_value(do_fn, prev.clone(), span) {
            Ok(Value::Err(e)) if last => return Err(*e),
            Ok(Value::Err(_)) => continue,
            Ok(v) => return Ok(v),
            Err(LxError::Propagate { value, .. }) if last => return Err(*value),
            Err(LxError::Propagate { .. }) => continue,
            Err(e) if last => return Err(Value::Str(Arc::from(e.to_string()))),
            Err(_) => continue,
        }
    }
    unreachable!()
}

fn compensate(
    completed: &[(String, Value, Value)],
    opts: &SagaOpts,
    span: Span,
) -> (Vec<String>, Vec<Value>) {
    let mut compensated = Vec::new();
    let mut comp_errors = Vec::new();
    for (id, result, undo_fn) in completed.iter().rev() {
        if let Some(ref on_comp) = opts.on_compensate {
            let id_val = Value::Str(Arc::from(id.as_str()));
            let _ = call_value(on_comp, id_val, span)
                .and_then(|partial| call_value(&partial, result.clone(), span));
        }
        let undo_result = call_value(undo_fn, result.clone(), span);
        match undo_result {
            Ok(Value::Err(e)) => {
                let mut f = IndexMap::new();
                f.insert("step".into(), Value::Str(Arc::from(id.as_str())));
                f.insert("error".into(), *e);
                comp_errors.push(Value::Record(Arc::new(f)));
            }
            Ok(_) => compensated.push(id.clone()),
            Err(e) => {
                let mut f = IndexMap::new();
                f.insert("step".into(), Value::Str(Arc::from(id.as_str())));
                f.insert("error".into(), Value::Str(Arc::from(e.to_string())));
                comp_errors.push(Value::Record(Arc::new(f)));
            }
        }
    }
    (compensated, comp_errors)
}

fn make_err(failed_step: &str, error: Value, compensated: &[String], comp_errors: Vec<Value>) -> Value {
    let mut f = IndexMap::new();
    f.insert("failed_step".into(), Value::Str(Arc::from(failed_step)));
    f.insert("error".into(), error);
    let comp_vals: Vec<Value> = compensated.iter()
        .map(|s| Value::Str(Arc::from(s.as_str()))).collect();
    f.insert("compensated".into(), Value::List(Arc::new(comp_vals)));
    if !comp_errors.is_empty() {
        f.insert("compensation_errors".into(), Value::List(Arc::new(comp_errors)));
    }
    Value::Err(Box::new(Value::Record(Arc::new(f))))
}

fn run_saga(
    steps: &[Value],
    initial_prev: Vec<(String, Value)>,
    opts: &SagaOpts,
    span: Span,
) -> Result<Value, LxError> {
    let start = Instant::now();
    let mut remaining: Vec<Value> = steps.to_vec();
    let mut completed: Vec<(String, Value, Value)> = Vec::new();
    let mut results: Vec<(String, Value)> = initial_prev;
    let mut completed_ids: HashSet<String> = results.iter().map(|(id, _)| id.clone()).collect();

    loop {
        if remaining.is_empty() {
            break;
        }
        if let Some(timeout) = opts.timeout_secs && start.elapsed().as_secs() >= timeout {
            let (comp, comp_err) = compensate(&completed, opts, span);
            return Ok(make_err("__timeout", Value::Str(Arc::from("saga timeout exceeded")), &comp, comp_err));
        }
        let Some(idx) = next_ready(&remaining, &completed_ids) else {
            return Err(LxError::runtime("saga: cycle or unmet dependencies", span));
        };
        let step = remaining.remove(idx);
        let sid = step_id(&step).unwrap_or("unknown").to_string();
        let do_fn = step_fn(&step, "do")
            .ok_or_else(|| LxError::type_err(format!("saga step '{sid}' missing 'do' function"), span))?;
        let undo_fn = step_fn(&step, "undo")
            .ok_or_else(|| LxError::type_err(format!("saga step '{sid}' missing 'undo' function"), span))?;
        let prev = build_prev(&results);

        match try_step(&do_fn, &prev, opts.max_retries, span) {
            Ok(result) => {
                results.push((sid.clone(), result.clone()));
                completed.push((sid.clone(), result, undo_fn));
                completed_ids.insert(sid);
            }
            Err(error) => {
                let (comp, comp_err) = compensate(&completed, opts, span);
                return Ok(make_err(&sid, error, &comp, comp_err));
            }
        }
    }

    let result_fields: IndexMap<String, Value> = results.into_iter().collect();
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(result_fields)))))
}

fn bi_run(args: &[Value], span: Span) -> Result<Value, LxError> {
    let steps = args[0].as_list()
        .ok_or_else(|| LxError::type_err("saga.run expects List of steps", span))?;
    run_saga(steps, Vec::new(), &SagaOpts::default(), span)
}

fn bi_run_with(args: &[Value], span: Span) -> Result<Value, LxError> {
    let steps = args[0].as_list()
        .ok_or_else(|| LxError::type_err("saga.run_with: first arg must be List of steps", span))?;
    let opts = parse_opts(&args[1], span)?;
    run_saga(steps, Vec::new(), &opts, span)
}

fn bi_define(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::List(_) = &args[0] else {
        return Err(LxError::type_err("saga.define expects List of steps", span));
    };
    let mut fields = IndexMap::new();
    fields.insert("__saga".into(), Value::Bool(true));
    fields.insert("steps".into(), args[0].clone());
    Ok(Value::Record(Arc::new(fields)))
}

fn bi_execute(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::Record(def) = &args[0] else {
        return Err(LxError::type_err("saga.execute: first arg must be a saga definition", span));
    };
    let steps = def.get("steps")
        .and_then(|v| v.as_list())
        .ok_or_else(|| LxError::type_err("saga.execute: invalid saga definition", span))?;
    let initial = match &args[1] {
        Value::Record(r) => r.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        _ => Vec::new(),
    };
    run_saga(steps, initial, &SagaOpts::default(), span)
}
