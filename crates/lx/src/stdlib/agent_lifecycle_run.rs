use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::agent_lifecycle::{HOOKS, get_lifecycle_id};

pub fn run_startup_hooks(agent: &Value, span: Span, ctx: &Arc<RuntimeCtx>) -> Result<(), LxError> {
    let Some(id) = get_lifecycle_id(agent) else {
        return Ok(());
    };
    let hooks: Vec<Value> = HOOKS
        .get(&id)
        .map(|h| h.startup.clone())
        .unwrap_or_default();
    for hook in &hooks {
        call_value_sync(hook, Value::Unit, span, ctx)?;
    }
    Ok(())
}

pub fn run_shutdown_hooks(
    agent: &Value,
    reason: &str,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<(), LxError> {
    let Some(id) = get_lifecycle_id(agent) else {
        return Ok(());
    };
    let hooks: Vec<Value> = HOOKS
        .get(&id)
        .map(|h| h.shutdown.clone())
        .unwrap_or_default();
    for hook in &hooks {
        call_value_sync(hook, Value::Str(Arc::from(reason)), span, ctx)?;
    }
    HOOKS.remove(&id);
    Ok(())
}

pub fn run_message_hooks(
    agent: &Value,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Option<Value>, LxError> {
    let Some(id) = get_lifecycle_id(agent) else {
        return Ok(None);
    };
    let hooks: Vec<Value> = HOOKS
        .get(&id)
        .map(|h| h.message.clone())
        .unwrap_or_default();
    for hook in &hooks {
        let result = call_value_sync(hook, msg.clone(), span, ctx)?;
        if matches!(result, Value::Err(_)) {
            return Ok(Some(result));
        }
    }
    Ok(None)
}

pub fn run_error_hooks(
    agent: &Value,
    err: &Value,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Option<Value>, LxError> {
    let Some(id) = get_lifecycle_id(agent) else {
        return Ok(None);
    };
    let hooks: Vec<Value> = HOOKS.get(&id).map(|h| h.error.clone()).unwrap_or_default();
    if hooks.is_empty() {
        return Ok(None);
    }
    for hook in &hooks {
        let partial = call_value_sync(hook, err.clone(), span, ctx)?;
        call_value_sync(&partial, msg.clone(), span, ctx)?;
    }
    Ok(Some(Value::Unit))
}

pub fn run_signal_hooks(
    agent: &Value,
    signal: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<(), LxError> {
    let Some(id) = get_lifecycle_id(agent) else {
        return Ok(());
    };
    let hooks: Vec<Value> = HOOKS.get(&id).map(|h| h.signal.clone()).unwrap_or_default();
    for hook in &hooks {
        call_value_sync(hook, signal.clone(), span, ctx)?;
    }
    Ok(())
}

pub fn get_idle_hooks(agent: &Value) -> Vec<(u64, Value)> {
    let Some(id) = get_lifecycle_id(agent) else {
        return Vec::new();
    };
    HOOKS.get(&id).map(|h| h.idle.clone()).unwrap_or_default()
}
