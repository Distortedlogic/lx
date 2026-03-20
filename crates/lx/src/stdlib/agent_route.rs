use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::agent_route_table::{
    ROUTE_TABLE, RoutingEntry, agent_key, find_candidates, select_agent, send_to_agent,
    str_list_from, with_load_tracking,
};

pub fn mk_register() -> Value {
    mk("agent.register", 2, bi_register)
}

pub fn mk_unregister() -> Value {
    mk("agent.unregister", 1, bi_unregister)
}

pub fn mk_registered() -> Value {
    mk("agent.registered", 1, bi_registered)
}

pub fn mk_route() -> Value {
    mk("agent.route", 2, bi_route)
}

pub fn mk_route_multi() -> Value {
    mk("agent.route_multi", 2, bi_route_multi)
}

fn bi_register(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = &args[0];
    let Value::Record(opts) = &args[1] else {
        return Err(LxError::type_err(
            "agent.register: second arg must be Record",
            span,
        ));
    };
    let key = agent_key(agent);
    let max_concurrent = opts
        .get("max_concurrent")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
        .unwrap_or(usize::MAX);
    ROUTE_TABLE.insert(
        key,
        RoutingEntry::new(
            agent.clone(),
            str_list_from(opts.get("traits")),
            str_list_from(opts.get("domains")),
            max_concurrent,
        ),
    );
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_unregister(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let key = agent_key(&args[0]);
    match ROUTE_TABLE.remove(&key) {
        Some(_) => Ok(Value::Ok(Box::new(Value::Unit))),
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
            "agent not found in routing table: {key}"
        )))))),
    }
}

fn bi_registered(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let (trait_f, domain_f) = match &args[0] {
        Value::Record(filter) => (
            filter
                .get("trait")
                .and_then(|v| v.as_str())
                .map(String::from),
            filter
                .get("domain")
                .and_then(|v| v.as_str())
                .map(String::from),
        ),
        _ => (None, None),
    };
    let matched: Vec<Value> = ROUTE_TABLE
        .iter()
        .filter(|e| {
            let entry = e.value();
            let t_ok = trait_f.as_deref().is_none_or(|t| entry.has_trait(t));
            let d_ok = domain_f.as_deref().is_none_or(|d| entry.has_domain(d));
            t_ok && d_ok
        })
        .map(|e| e.value().agent.clone())
        .collect();
    Ok(Value::Ok(Box::new(Value::List(Arc::new(matched)))))
}

fn bi_route(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let msg = &args[0];
    let Value::Record(opts) = &args[1] else {
        return Err(LxError::type_err(
            "agent.route: second arg must be Record",
            span,
        ));
    };
    let exclude: Vec<String> = match opts.get("exclude") {
        Some(Value::List(agents)) => agents.iter().map(agent_key).collect(),
        _ => Vec::new(),
    };
    let candidates = find_candidates(opts, &exclude, true);
    if candidates.is_empty() {
        if let Some(fb) = opts.get("fallback") {
            return send_to_agent(fb, msg, span, ctx);
        }
        return Ok(Value::Err(Box::new(super::agent_errors::unavailable(
            "route",
            "no matching agents registered",
        ))));
    }
    let idx = select_agent(&candidates, opts, span, ctx)?;
    let selected = &candidates[idx];
    with_load_tracking(&selected.key, || {
        send_to_agent(&selected.agent, msg, span, ctx)
    })
}

fn bi_route_multi(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let msg = &args[0];
    let Value::Record(opts) = &args[1] else {
        return Err(LxError::type_err(
            "agent.route_multi: second arg must be Record",
            span,
        ));
    };
    let candidates = find_candidates(opts, &[], false);
    if candidates.is_empty() {
        return Ok(Value::Err(Box::new(super::agent_errors::unavailable(
            "route_multi",
            "no matching agents registered",
        ))));
    }
    let mut results = Vec::new();
    for c in &candidates {
        let result = with_load_tracking(&c.key, || send_to_agent(&c.agent, msg, span, ctx))?;
        match result {
            Value::Ok(v) => results.push(*v),
            other => results.push(other),
        }
    }
    if let Some(reconcile_cfg) = opts.get("reconcile") {
        let reconcile_fn = super::agent_reconcile::mk_reconcile();
        let partial = call_value_sync(&reconcile_fn, Value::List(Arc::new(results)), span, ctx)?;
        let reconciled = call_value_sync(&partial, reconcile_cfg.clone(), span, ctx)?;
        return Ok(Value::Ok(Box::new(reconciled)));
    }
    Ok(Value::Ok(Box::new(Value::List(Arc::new(results)))))
}
