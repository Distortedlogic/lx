use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::{STORES, now, persist, store_id};

fn transition(
    store_val: &Value,
    task_id_val: &Value,
    extra: Option<&Value>,
    from: &[&str],
    to: &str,
    span: Span,
) -> Result<Value, LxError> {
    let sid = store_id(store_val, span)?;
    let tid = task_id_val
        .as_str()
        .ok_or_else(|| LxError::type_err("tasks: id must be Str", span))?;
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("tasks: store not found", span))?;
    let task = store
        .tasks
        .get(tid)
        .ok_or_else(|| LxError::runtime(format!("tasks: task '{tid}' not found"), span))?
        .clone();
    let Value::Record(r) = task else {
        return Err(LxError::runtime("tasks: corrupt task record", span));
    };
    let status = r.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if !from.contains(&status) {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("tasks: cannot transition '{status}' -> '{to}'").as_str(),
        )))));
    }
    let mut fields = (*r).clone();
    fields.insert("status".into(), Value::Str(Arc::from(to)));
    fields.insert("updated_at".into(), Value::Str(now()));
    if let Some(Value::Record(ef)) = extra {
        for (k, v) in ef.iter() {
            if k != "id" && k != "status" && k != "created_at" {
                fields.insert(k.clone(), v.clone());
            }
        }
    }
    store
        .tasks
        .insert(tid.to_string(), Value::Record(Arc::new(fields)));
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

pub(super) fn bi_start(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    transition(&args[0], &args[1], None, &["todo"], "in_progress", span)
}

pub(super) fn bi_submit(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    transition(
        &args[0],
        &args[1],
        Some(&args[2]),
        &["in_progress", "revision"],
        "submitted",
        span,
    )
}

pub(super) fn bi_audit(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    transition(
        &args[0],
        &args[1],
        None,
        &["submitted"],
        "pending_audit",
        span,
    )
}

pub(super) fn bi_pass(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    transition(&args[0], &args[1], None, &["pending_audit"], "passed", span)
}

pub(super) fn bi_fail(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    transition(
        &args[0],
        &args[1],
        Some(&args[2]),
        &["pending_audit"],
        "failed",
        span,
    )
}

pub(super) fn bi_revise(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    transition(&args[0], &args[1], None, &["failed"], "revision", span)
}

pub(super) fn bi_complete(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    transition(
        &args[0],
        &args[1],
        Some(&args[2]),
        &["passed"],
        "complete",
        span,
    )
}
