use std::sync::Arc;
use std::time::Instant;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::registry_query;
use super::registry_store::{self as store, AgentEntry, REGISTRIES};

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("start".into(), mk("registry.start", 1, bi_start));
    m.insert("stop".into(), mk("registry.stop", 1, bi_stop));
    m.insert("connect".into(), mk("registry.connect", 1, bi_connect));
    m.insert("register".into(), mk("registry.register", 2, bi_register));
    m.insert(
        "deregister".into(),
        mk("registry.deregister", 2, bi_deregister),
    );
    m.insert(
        "find".into(),
        mk("registry.find", 2, registry_query::bi_find),
    );
    m.insert(
        "find_one".into(),
        mk("registry.find_one", 2, registry_query::bi_find_one),
    );
    m.insert(
        "health".into(),
        mk("registry.health", 2, registry_query::bi_health),
    );
    m.insert(
        "load".into(),
        mk("registry.load", 2, registry_query::bi_load),
    );
    m.insert(
        "watch".into(),
        mk("registry.watch", 3, registry_query::bi_watch),
    );
    m
}

fn bi_start(_args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store::create_registry();
    Ok(Value::Ok(Box::new(store::reg_handle(id))))
}

fn bi_stop(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store::get_reg_id(&args[0], span)?;
    REGISTRIES.remove(&id);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_connect(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let port = match &args[0] {
        Value::Record(r) => r
            .get("port")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok()),
        _ => None,
    };
    if let Some(port_id) = port.filter(|id| REGISTRIES.contains_key(id)) {
        return Ok(Value::Ok(Box::new(store::conn_handle(port_id))));
    }
    let id = store::create_registry();
    Ok(Value::Ok(Box::new(store::conn_handle(id))))
}

fn extract_str_list(v: &Value) -> Vec<String> {
    match v {
        Value::List(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

fn bi_register(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let reg_id = store::get_reg_id(&args[0], span)?;
    let reg = REGISTRIES
        .get(&reg_id)
        .ok_or_else(|| LxError::runtime("registry.register: registry not found", span))?;

    let Value::Record(info) = &args[1] else {
        return Err(LxError::type_err(
            "registry.register: expected Record with agent info",
            span,
        ));
    };

    let name = info
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::type_err("registry.register: name required", span))?
        .to_string();

    let traits = info.get("traits").map(extract_str_list).unwrap_or_default();
    let domains = info
        .get("domains")
        .map(extract_str_list)
        .unwrap_or_default();
    let capacity: u64 = info
        .get("capacity")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
        .unwrap_or(10);
    let metadata = info
        .get("metadata")
        .cloned()
        .unwrap_or(Value::Record(Arc::new(IndexMap::new())));

    let now = Instant::now();
    let entry = AgentEntry {
        name: name.clone(),
        traits,
        domains,
        capacity,
        metadata,
        load: std::sync::atomic::AtomicU64::new(0),
        registered_at: now,
        last_heartbeat: now,
        healthy: true,
    };

    let agent_ref = store::entry_to_agent_ref(&entry);
    reg.agents.insert(name, entry);
    drop(reg);

    registry_query::fire_watchers(&reg_id, "join", &agent_ref, span, ctx);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_deregister(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let reg_id = store::get_reg_id(&args[0], span)?;
    let reg = REGISTRIES
        .get(&reg_id)
        .ok_or_else(|| LxError::runtime("registry.deregister: registry not found", span))?;

    let name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("registry.deregister: name must be Str", span))?;

    match reg.agents.remove(name) {
        Some((_, entry)) => {
            let agent_ref = store::entry_to_agent_ref(&entry);
            drop(reg);
            registry_query::fire_watchers(&reg_id, "leave", &agent_ref, span, ctx);
            Ok(Value::Ok(Box::new(Value::Unit)))
        }
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "agent not found",
        ))))),
    }
}
