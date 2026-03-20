use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

static AGENT_HANDLERS: LazyLock<DashMap<u64, Value>> = LazyLock::new(DashMap::new);
static NEXT_HANDLER_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static CURRENT_HANDLER_ID: Cell<Option<u64>> = const { Cell::new(None) };
    static PENDING_EVOLVE: RefCell<Option<Value>> = const { RefCell::new(None) };
}

pub fn handler_id_from_agent(agent: &Value) -> Option<u64> {
    match agent {
        Value::Record(r) => r
            .get("__handler_id")
            .and_then(|v| v.as_int())
            .and_then(|n| u64::try_from(n).ok()),
        _ => None,
    }
}

pub fn lookup_handler(id: u64) -> Option<Value> {
    AGENT_HANDLERS.get(&id).map(|v| v.value().clone())
}

pub fn set_current_handler_id(id: Option<u64>) {
    CURRENT_HANDLER_ID.set(id);
}

pub fn take_pending_evolve() -> Option<Value> {
    PENDING_EVOLVE.with(|cell| cell.borrow_mut().take())
}

pub fn apply_pending_evolve(handler_id: u64) {
    if let Some(new_handler) = take_pending_evolve() {
        AGENT_HANDLERS.insert(handler_id, new_handler);
    }
}

pub fn mk_reload() -> Value {
    mk("agent.reload", 2, bi_reload)
}

pub fn mk_evolve() -> Value {
    mk("agent.evolve", 1, bi_evolve)
}

pub fn mk_update_traits() -> Value {
    mk("agent.update_traits", 2, bi_update_traits)
}

fn bi_reload(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = &args[0];
    let opts = &args[1];
    let Value::Record(agent_fields) = agent else {
        return Err(LxError::type_err(
            "agent.reload: first arg must be an agent Record",
            span,
        ));
    };
    if agent_fields.contains_key("__pid") {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "cannot reload subprocess agent",
        )))));
    }
    let Value::Record(opts_fields) = opts else {
        return Err(LxError::type_err(
            "agent.reload: second arg must be a Record with handler field",
            span,
        ));
    };
    let new_handler = opts_fields
        .get("handler")
        .ok_or_else(|| LxError::runtime("agent.reload: opts must have 'handler' field", span))?;
    let handler_id = handler_id_from_agent(agent)
        .unwrap_or_else(|| NEXT_HANDLER_ID.fetch_add(1, Ordering::Relaxed));
    let old_handler = AGENT_HANDLERS
        .get(&handler_id)
        .map(|v| v.value().clone())
        .or_else(|| agent_fields.get("handler").cloned());
    AGENT_HANDLERS.insert(handler_id, new_handler.clone());
    let mut new_rec = agent_fields.as_ref().clone();
    new_rec.insert("__handler_id".into(), Value::Int(BigInt::from(handler_id)));
    if !new_rec.contains_key("handler") {
        new_rec.insert("handler".into(), new_handler.clone());
    }
    if let Some(on_reload) = opts_fields.get("on_reload") {
        let old_val = old_handler.unwrap_or(Value::None);
        let partial = call_value_sync(on_reload, old_val, span, ctx)?;
        call_value_sync(&partial, new_handler.clone(), span, ctx)?;
    }
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(new_rec)))))
}

fn bi_evolve(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let opts = &args[0];
    let Value::Record(opts_fields) = opts else {
        return Err(LxError::type_err(
            "agent.evolve: arg must be a Record with handler field",
            span,
        ));
    };
    let new_handler = opts_fields
        .get("handler")
        .ok_or_else(|| LxError::runtime("agent.evolve: opts must have 'handler' field", span))?;
    let has_ctx = CURRENT_HANDLER_ID.get().is_some();
    if !has_ctx {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "agent.evolve: not in agent handler context",
        )))));
    }
    PENDING_EVOLVE.with(|cell| {
        *cell.borrow_mut() = Some(new_handler.clone());
    });
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_update_traits(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = &args[0];
    let changes = &args[1];
    let Value::Record(agent_fields) = agent else {
        return Err(LxError::type_err(
            "agent.update_traits: first arg must be an agent Record",
            span,
        ));
    };
    let Value::Record(changes_fields) = changes else {
        return Err(LxError::type_err(
            "agent.update_traits: second arg must be a Record",
            span,
        ));
    };
    let mut traits: Vec<Value> = agent_fields
        .get("__traits")
        .and_then(|v| v.as_list())
        .map(|l| l.as_ref().clone())
        .unwrap_or_default();
    if let Some(add_list) = changes_fields.get("add").and_then(|v| v.as_list()) {
        for t in add_list.iter() {
            if !traits.iter().any(|existing| existing == t) {
                traits.push(t.clone());
            }
        }
    }
    if let Some(remove_list) = changes_fields.get("remove").and_then(|v| v.as_list()) {
        traits.retain(|t| !remove_list.contains(t));
    }
    let mut new_rec = agent_fields.as_ref().clone();
    new_rec.insert("__traits".into(), Value::List(Arc::new(traits)));
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(new_rec)))))
}
