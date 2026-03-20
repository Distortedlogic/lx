use std::sync::Arc;

use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::durable::{WORKFLOWS, workflow_id};
use super::durable_io;

pub(super) fn mk_run() -> Value {
    mk("durable.run", 2, bi_run)
}

pub(super) fn mk_step() -> Value {
    mk("durable.step", 3, bi_step)
}

pub(super) fn mk_sleep() -> Value {
    mk("durable.sleep", 2, bi_sleep)
}

pub(super) fn mk_signal() -> Value {
    mk("durable.signal", 2, bi_signal)
}

pub(super) fn mk_send_signal() -> Value {
    mk("durable.send_signal", 3, bi_send_signal)
}

fn bi_run(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let wid = workflow_id(&args[0], span)?;
    let input = args[1].clone();
    let (handler, storage_dir, name, run_id) = {
        let wf = WORKFLOWS
            .get(&wid)
            .ok_or_else(|| LxError::runtime("durable.run: workflow not found", span))?;
        (
            wf.handler.clone(),
            wf.dir.clone(),
            wf.name.clone(),
            wf.run_id.clone(),
        )
    };
    let loaded = durable_io::load_state(&storage_dir);
    let (resumed, steps_replayed) = if let Some(ref state) = loaded {
        (state.status != "completed", state.completed_steps.len())
    } else {
        (false, 0)
    };
    let now = chrono::Utc::now().to_rfc3339();
    let started_at = loaded
        .as_ref()
        .map(|s| s.started_at.clone())
        .unwrap_or_else(|| now.clone());
    durable_io::ensure_dirs(&storage_dir, span)?;
    durable_io::save_state(
        &storage_dir,
        "running",
        "",
        &started_at,
        &loaded
            .as_ref()
            .map(|s| s.completed_steps.clone())
            .unwrap_or_default(),
        span,
    )?;
    if let Some(mut wf) = WORKFLOWS.get_mut(&wid)
        && let Some(ref state) = loaded
    {
        wf.completed_steps = state.completed_steps.clone();
    }
    let ctx_val = record! {
        "workflow_id" => Value::Str(Arc::from(name.as_str())),
        "run_id" => Value::Str(Arc::from(run_id.as_str())),
        "storage_dir" => Value::Str(Arc::from(storage_dir.to_string_lossy().as_ref())),
        "__wf_handle" => Value::Int(BigInt::from(wid)),
        "input" => input,
    };
    let result = call_value_sync(&handler, ctx_val, span, ctx);
    match result {
        Ok(val) => {
            let completed_steps = WORKFLOWS
                .get(&wid)
                .map(|wf| wf.completed_steps.clone())
                .unwrap_or_default();
            durable_io::save_state(
                &storage_dir,
                "completed",
                "",
                &started_at,
                &completed_steps,
                span,
            )?;
            if let Some(mut wf) = WORKFLOWS.get_mut(&wid) {
                wf.status = "completed".into();
            }
            Ok(record! {
                "value" => val,
                "workflow_id" => Value::Str(Arc::from(name.as_str())),
                "resumed" => Value::Bool(resumed),
                "steps_replayed" => Value::Int(BigInt::from(steps_replayed)),
            })
        }
        Err(e) => {
            let completed_steps = WORKFLOWS
                .get(&wid)
                .map(|wf| wf.completed_steps.clone())
                .unwrap_or_default();
            durable_io::save_state(
                &storage_dir,
                "failed",
                "",
                &started_at,
                &completed_steps,
                span,
            )?;
            if let Some(mut wf) = WORKFLOWS.get_mut(&wid) {
                wf.status = "failed".into();
            }
            Err(e)
        }
    }
}

fn bi_step(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let wid = extract_wf_handle(&args[0], span)?;
    let step_name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("durable.step: name must be Str", span))?;
    let body = &args[2];
    let storage_dir = {
        let wf = WORKFLOWS
            .get(&wid)
            .ok_or_else(|| LxError::runtime("durable.step: workflow not found", span))?;
        wf.dir.clone()
    };
    let already_completed = WORKFLOWS
        .get(&wid)
        .map(|wf| wf.completed_steps.contains(&step_name.to_string()))
        .unwrap_or(false);
    if already_completed && let Some(cached) = durable_io::load_step(&storage_dir, step_name) {
        return Ok(cached);
    }
    let result = call_value_sync(body, Value::Unit, span, ctx)?;
    durable_io::save_step(&storage_dir, step_name, &result, span)?;
    if let Some(mut wf) = WORKFLOWS.get_mut(&wid) {
        wf.completed_steps.push(step_name.to_string());
        durable_io::save_state(
            &storage_dir,
            "running",
            step_name,
            &wf.started_at,
            &wf.completed_steps,
            span,
        )?;
    }
    Ok(result)
}

fn bi_sleep(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let wid = extract_wf_handle(&args[0], span)?;
    let duration_ms: u64 = args[1]
        .as_int()
        .and_then(|n| n.try_into().ok())
        .ok_or_else(|| LxError::type_err("durable.sleep: duration must be Int (ms)", span))?;
    let sleep_name = format!("__sleep_{duration_ms}");
    let storage_dir = {
        let wf = WORKFLOWS
            .get(&wid)
            .ok_or_else(|| LxError::runtime("durable.sleep: workflow not found", span))?;
        wf.dir.clone()
    };
    if let Some(cached) = durable_io::load_step(&storage_dir, &sleep_name)
        && let Some(wake_str) = cached.as_str()
        && let Ok(wake_time) = chrono::DateTime::parse_from_rfc3339(wake_str)
        && chrono::Utc::now() >= wake_time
    {
        return Ok(Value::Unit);
    }
    let wake_time = chrono::Utc::now() + chrono::Duration::milliseconds(duration_ms as i64);
    let wake_str = Value::Str(Arc::from(wake_time.to_rfc3339().as_str()));
    durable_io::save_step(&storage_dir, &sleep_name, &wake_str, span)?;
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
    Ok(Value::Unit)
}

fn bi_signal(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let wid = extract_wf_handle(&args[0], span)?;
    let signal_name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("durable.signal: name must be Str", span))?;
    let storage_dir = {
        let wf = WORKFLOWS
            .get(&wid)
            .ok_or_else(|| LxError::runtime("durable.signal: workflow not found", span))?;
        wf.dir.clone()
    };
    if let Some(val) = durable_io::load_signal(&storage_dir, signal_name) {
        return Ok(val);
    }
    if let Some(mut wf) = WORKFLOWS.get_mut(&wid) {
        wf.status = format!("waiting_signal:{signal_name}");
        let dir = wf.dir.clone();
        let started = wf.started_at.clone();
        let steps = wf.completed_steps.clone();
        drop(wf);
        durable_io::save_state(
            &dir,
            &format!("waiting_signal:{signal_name}"),
            signal_name,
            &started,
            &steps,
            span,
        )?;
    }
    let mut attempts = 0;
    loop {
        if let Some(val) = durable_io::load_signal(&storage_dir, signal_name) {
            if let Some(mut wf) = WORKFLOWS.get_mut(&wid) {
                wf.status = "running".into();
            }
            return Ok(val);
        }
        attempts += 1;
        if attempts > 100 {
            return Err(LxError::runtime(
                format!("durable.signal: timeout waiting for signal '{signal_name}'"),
                span,
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn bi_send_signal(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let wf_name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("durable.send_signal: workflow_id must be Str", span))?;
    let signal_name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("durable.send_signal: name must be Str", span))?;
    let value = &args[2];
    let storage_dir = find_workflow_dir(wf_name);
    match storage_dir {
        Some(dir) => {
            durable_io::save_signal(&dir, signal_name, value, span)?;
            Ok(Value::Bool(true))
        }
        None => Err(LxError::runtime(
            format!("durable.send_signal: no active workflow '{wf_name}'"),
            span,
        )),
    }
}

fn extract_wf_handle(ctx_val: &Value, span: Span) -> Result<u64, LxError> {
    match ctx_val {
        Value::Record(r) => r
            .get("__wf_handle")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| {
                LxError::type_err("durable: expected workflow context with __wf_handle", span)
            }),
        _ => Err(LxError::type_err(
            "durable: expected workflow context Record",
            span,
        )),
    }
}

fn find_workflow_dir(name: &str) -> Option<std::path::PathBuf> {
    for entry in WORKFLOWS.iter() {
        if entry.value().name == name {
            return Some(entry.value().dir.clone());
        }
    }
    None
}
