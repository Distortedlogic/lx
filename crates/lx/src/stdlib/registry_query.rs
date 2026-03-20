use std::sync::Arc;
use std::sync::atomic::Ordering;

use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::registry_store::{self as store, REGISTRIES, Watcher};

pub fn bi_find(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let reg_id = store::get_reg_id(&args[0], span)?;
    let reg = REGISTRIES
        .get(&reg_id)
        .ok_or_else(|| LxError::runtime("registry.find: registry not found", span))?;

    let results: Vec<Value> = reg
        .agents
        .iter()
        .filter(|e| e.value().healthy && store::entry_matches(e.value(), &args[1]))
        .map(|e| store::entry_to_agent_ref(e.value()))
        .collect();

    Ok(Value::Ok(Box::new(Value::List(Arc::new(results)))))
}

pub fn bi_find_one(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let reg_id = store::get_reg_id(&args[0], span)?;
    let reg = REGISTRIES
        .get(&reg_id)
        .ok_or_else(|| LxError::runtime("registry.find_one: registry not found", span))?;

    let strategy = match &args[1] {
        Value::Record(q) => q
            .get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("first")
            .to_string(),
        _ => "first".into(),
    };

    let matches: Vec<_> = reg
        .agents
        .iter()
        .filter(|e| e.value().healthy && store::entry_matches(e.value(), &args[1]))
        .collect();

    if matches.is_empty() {
        return Ok(Value::None);
    }

    let selected = match strategy.as_str() {
        "least_loaded" => matches
            .iter()
            .min_by_key(|e| {
                let load = e.value().load.load(Ordering::Relaxed);
                let cap = e.value().capacity.max(1);
                load * 1000 / cap
            })
            .map(|e| store::entry_to_agent_ref(e.value())),
        "round_robin" | "random" => {
            let idx = reg.round_robin.fetch_add(1, Ordering::Relaxed) as usize % matches.len();
            Some(store::entry_to_agent_ref(matches[idx].value()))
        }
        _ => Some(store::entry_to_agent_ref(matches[0].value())),
    };

    match selected {
        Some(v) => Ok(Value::Ok(Box::new(v))),
        None => Ok(Value::None),
    }
}

pub fn bi_health(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let reg_id = store::get_reg_id(&args[0], span)?;
    let reg = REGISTRIES
        .get(&reg_id)
        .ok_or_else(|| LxError::runtime("registry.health: registry not found", span))?;

    let name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("registry.health: name must be Str", span))?;

    match reg.agents.get(name) {
        Some(entry) => {
            let uptime_ms = entry.registered_at.elapsed().as_millis() as i64;
            let last_seen_ms = entry.last_heartbeat.elapsed().as_millis() as i64;
            Ok(Value::Ok(Box::new(record! {
                "healthy" => Value::Bool(entry.healthy),
                "last_seen_ms" => Value::Int(BigInt::from(last_seen_ms)),
                "uptime_ms" => Value::Int(BigInt::from(uptime_ms)),
            })))
        }
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "agent not found",
        ))))),
    }
}

pub fn bi_load(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let reg_id = store::get_reg_id(&args[0], span)?;
    let reg = REGISTRIES
        .get(&reg_id)
        .ok_or_else(|| LxError::runtime("registry.load: registry not found", span))?;

    let name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("registry.load: name must be Str", span))?;

    match reg.agents.get(name) {
        Some(entry) => {
            let current = entry.load.load(Ordering::Relaxed);
            let cap = entry.capacity;
            let pct = if cap > 0 {
                current as f64 / cap as f64
            } else {
                0.0
            };
            Ok(Value::Ok(Box::new(record! {
                "current" => Value::Int(BigInt::from(current)),
                "capacity" => Value::Int(BigInt::from(cap)),
                "pct" => Value::Float(pct),
            })))
        }
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "agent not found",
        ))))),
    }
}

pub fn bi_watch(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let reg_id = store::get_reg_id(&args[0], span)?;
    let mut reg = REGISTRIES
        .get_mut(&reg_id)
        .ok_or_else(|| LxError::runtime("registry.watch: registry not found", span))?;

    let query = store::parse_watch_query(&args[1]);
    let callback = args[2].clone();
    reg.watchers.push(Watcher { query, callback });
    Ok(Value::Unit)
}

pub fn fire_watchers(
    reg_id: &u64,
    kind: &str,
    agent_ref: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) {
    let Some(reg) = REGISTRIES.get(reg_id) else {
        return;
    };

    let agent_name = match agent_ref {
        Value::Record(r) => r
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        _ => return,
    };

    let entry_opt = reg.agents.get(&agent_name);

    let callbacks: Vec<Value> = reg
        .watchers
        .iter()
        .filter(|w| match &entry_opt {
            Some(entry) => store::watcher_matches(&w.query, entry.value()),
            None => kind == "leave",
        })
        .map(|w| w.callback.clone())
        .collect();

    drop(entry_opt);
    drop(reg);

    let event = record! {
        "kind" => Value::Str(Arc::from(kind)),
        "agent" => agent_ref.clone(),
    };

    for cb in callbacks {
        let _ = call_value_sync(&cb, event.clone(), span, ctx);
    }
}
