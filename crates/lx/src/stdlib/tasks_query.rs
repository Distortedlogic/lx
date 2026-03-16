use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::{STORES, now, persist, store_id};

pub(super) fn bi_children(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let parent = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("tasks.children: id must be Str", span))?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("tasks: store not found", span))?;
    let kids: Vec<Value> = store
        .tasks
        .values()
        .filter(|t| {
            matches!(t, Value::Record(r) if
            r.get("parent").and_then(|v| v.as_str()) == Some(parent))
        })
        .cloned()
        .collect();
    Ok(Value::List(Arc::new(kids)))
}

pub(super) fn bi_list(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("tasks: store not found", span))?;
    let status_filter = match &args[1] {
        Value::Record(r) => r
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    };
    let items: Vec<Value> = store
        .tasks
        .values()
        .filter(|t| match &status_filter {
            Some(s) => matches!(t, Value::Record(r) if
                r.get("status").and_then(|v| v.as_str()) == Some(s)),
            None => true,
        })
        .cloned()
        .collect();
    Ok(Value::List(Arc::new(items)))
}

pub(super) fn bi_update(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let tid = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("tasks.update: id must be Str", span))?;
    let Value::Record(extra) = &args[2] else {
        return Err(LxError::type_err("tasks.update: opts must be Record", span));
    };
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
    let mut fields = (*r).clone();
    for (k, v) in extra.iter() {
        if k != "id" && k != "status" && k != "created_at" {
            fields.insert(k.clone(), v.clone());
        }
    }
    fields.insert("updated_at".into(), Value::Str(now()));
    store
        .tasks
        .insert(tid.to_string(), Value::Record(Arc::new(fields)));
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}
