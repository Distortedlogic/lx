use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::agent_lifecycle_run;

static NEXT_LIFECYCLE_ID: AtomicU64 = AtomicU64::new(1);

pub(super) struct LifecycleHooks {
    pub(super) startup: Vec<Value>,
    pub(super) shutdown: Vec<Value>,
    pub(super) error: Vec<Value>,
    pub(super) idle: Vec<(u64, Value)>,
    pub(super) message: Vec<Value>,
    pub(super) signal: Vec<Value>,
}

impl LifecycleHooks {
    pub(super) fn new() -> Self {
        Self {
            startup: Vec::new(),
            shutdown: Vec::new(),
            error: Vec::new(),
            idle: Vec::new(),
            message: Vec::new(),
            signal: Vec::new(),
        }
    }
}

pub(super) static HOOKS: LazyLock<DashMap<u64, LifecycleHooks>> = LazyLock::new(DashMap::new);

pub(super) fn get_lifecycle_id(agent: &Value) -> Option<u64> {
    match agent {
        Value::Record(r) => r
            .get("__lifecycle_id")
            .and_then(|v| v.as_int())
            .and_then(|n| u64::try_from(n).ok()),
        Value::Object { id, .. } => Some(*id),
        _ => None,
    }
}

fn ensure_lifecycle_id(agent: &Value) -> (u64, Value) {
    if let Some(id) = get_lifecycle_id(agent) {
        return (id, agent.clone());
    }
    let id = NEXT_LIFECYCLE_ID.fetch_add(1, Ordering::Relaxed);
    match agent {
        Value::Record(r) => {
            let mut new_rec = r.as_ref().clone();
            new_rec.insert("__lifecycle_id".into(), Value::Int(BigInt::from(id)));
            (id, Value::Record(Arc::new(new_rec)))
        }
        _ => (id, agent.clone()),
    }
}

fn parse_event_name(val: &Value, span: Span) -> Result<String, LxError> {
    match val {
        Value::Str(s) => Ok(s.to_string()),
        Value::Tagged { tag, .. } => Ok(tag.to_string()),
        _ => Err(LxError::type_err(
            format!(
                "agent.on: event must be Str or tagged atom, got {}",
                val.type_name()
            ),
            span,
        )),
    }
}

fn validate_event(name: &str, span: Span) -> Result<(), LxError> {
    match name {
        "startup" | "shutdown" | "error" | "idle" | "message" | "signal" => Ok(()),
        _ => Err(LxError::runtime(
            format!(
                "agent.on: unknown event '{name}'. \
                 Expected: startup, shutdown, error, idle, message, signal"
            ),
            span,
        )),
    }
}

pub fn builtins() -> Vec<(&'static str, Value)> {
    vec![
        ("on", mk("agent.on", 3, bi_on)),
        ("on_remove", mk("agent.on_remove", 2, bi_on_remove)),
        ("startup", mk("agent.startup", 1, bi_startup)),
        ("shutdown", mk("agent.shutdown", 2, bi_shutdown)),
        ("signal", mk("agent.signal", 2, bi_signal)),
        ("idle_hooks", mk("agent.idle_hooks", 1, bi_idle_hooks)),
    ]
}

fn bi_startup(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    agent_lifecycle_run::run_startup_hooks(&args[0], span, ctx)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_shutdown(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let reason = match &args[1] {
        Value::Str(s) => s.to_string(),
        _ => "shutdown".to_string(),
    };
    agent_lifecycle_run::run_shutdown_hooks(&args[0], &reason, span, ctx)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_signal(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    agent_lifecycle_run::run_signal_hooks(&args[0], &args[1], span, ctx)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_idle_hooks(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let hooks = agent_lifecycle_run::get_idle_hooks(&args[0]);
    let items: Vec<Value> = hooks
        .into_iter()
        .map(|(secs, cb)| {
            let mut rec = indexmap::IndexMap::new();
            rec.insert("seconds".into(), Value::Int(BigInt::from(secs)));
            rec.insert("callback".into(), cb);
            Value::Record(Arc::new(rec))
        })
        .collect();
    Ok(Value::List(Arc::new(items)))
}

fn bi_on(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = &args[0];
    let event_val = &args[1];
    let event = parse_event_name(event_val, span)?;
    validate_event(&event, span)?;

    let (id, new_agent) = ensure_lifecycle_id(agent);

    let callback = &args[2];
    if event == "idle" {
        return register_idle_with_duration(id, callback, new_agent, span);
    }
    register_hook(id, &event, callback);
    Ok(Value::Ok(Box::new(new_agent)))
}

fn register_idle_with_duration(
    id: u64,
    callback: &Value,
    new_agent: Value,
    span: Span,
) -> Result<Value, LxError> {
    match callback {
        Value::Int(n) => {
            let secs: u64 = n.try_into().map_err(|_| {
                LxError::runtime("agent.on: idle duration must be positive integer", span)
            })?;
            let partial = Value::BuiltinFunc(crate::value::BuiltinFunc {
                name: "agent.on.idle_partial",
                arity: 4,
                kind: crate::value::BuiltinKind::Sync(bi_on_idle_partial),
                applied: vec![
                    new_agent,
                    Value::Int(BigInt::from(id)),
                    Value::Int(BigInt::from(secs)),
                ],
            });
            Ok(partial)
        }
        _ => Err(LxError::runtime(
            "agent.on: :idle requires a duration (seconds) \
             before the callback: agent.on me \"idle\" 30 callback",
            span,
        )),
    }
}

fn bi_on_idle_partial(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let new_agent = &args[0];
    let id: u64 = args[1]
        .as_int()
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| LxError::runtime("agent.on: internal error (bad id)", span))?;
    let secs: u64 = args[2]
        .as_int()
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| LxError::runtime("agent.on: internal error (bad secs)", span))?;
    let callback = &args[3];

    HOOKS
        .entry(id)
        .or_insert_with(LifecycleHooks::new)
        .idle
        .push((secs, callback.clone()));

    Ok(Value::Ok(Box::new(new_agent.clone())))
}

fn register_hook(id: u64, event: &str, callback: &Value) {
    let mut entry = HOOKS.entry(id).or_insert_with(LifecycleHooks::new);
    match event {
        "startup" => entry.startup.push(callback.clone()),
        "shutdown" => entry.shutdown.push(callback.clone()),
        "error" => entry.error.push(callback.clone()),
        "message" => entry.message.push(callback.clone()),
        "signal" => entry.signal.push(callback.clone()),
        _ => {}
    }
}

fn bi_on_remove(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = &args[0];
    let event_val = &args[1];
    let event = parse_event_name(event_val, span)?;
    validate_event(&event, span)?;

    let Some(id) = get_lifecycle_id(agent) else {
        return Ok(Value::Ok(Box::new(Value::Unit)));
    };

    if let Some(mut hooks) = HOOKS.get_mut(&id) {
        match event.as_str() {
            "startup" => hooks.startup.clear(),
            "shutdown" => hooks.shutdown.clear(),
            "error" => hooks.error.clear(),
            "idle" => hooks.idle.clear(),
            "message" => hooks.message.clear(),
            "signal" => hooks.signal.clear(),
            _ => {}
        }
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}
