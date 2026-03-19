use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

static NEXT_SUP_ID: AtomicU64 = AtomicU64::new(1);
pub(super) static SUPERVISORS: std::sync::LazyLock<DashMap<u64, Supervisor>> =
    std::sync::LazyLock::new(DashMap::new);

pub(super) struct Supervisor {
    pub(super) strategy: String,
    max_restarts: usize,
    _window: u64,
    pub(super) children: Vec<ChildSpec>,
    pub(super) restart_counts: Vec<usize>,
}

pub(super) struct ChildSpec {
    pub(super) id: String,
    spawn_fn: Value,
    restart: String,
    current: Value,
}

fn sup_id_from(val: &Value, span: Span) -> Result<u64, LxError> {
    match val {
        Value::Record(r) => r
            .get("__sup_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.to_u64())
            .ok_or_else(|| {
                LxError::type_err("agent.supervise: expected supervisor with __sup_id", span)
            }),
        _ => Err(LxError::type_err(
            "agent.supervise: expected supervisor Record",
            span,
        )),
    }
}

pub fn mk_supervise() -> Value {
    mk("agent.supervise", 1, bi_supervise)
}

pub fn mk_child() -> Value {
    mk("agent.child", 2, bi_child)
}

pub fn mk_supervise_stop() -> Value {
    mk("agent.supervise_stop", 1, bi_supervise_stop)
}

fn bi_supervise(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(config) = &args[0] else {
        return Err(LxError::type_err(
            "agent.supervise: config must be a Record",
            span,
        ));
    };
    let strategy = config
        .get("strategy")
        .and_then(|v| v.as_str())
        .map(|s| s.trim_start_matches(':').to_string())
        .unwrap_or_else(|| "one_for_one".into());
    let max_restarts = config
        .get("max_restarts")
        .and_then(|v| v.as_int())
        .and_then(|n| n.to_usize())
        .unwrap_or(5);
    let window = config
        .get("window")
        .and_then(|v| v.as_int())
        .and_then(|n| n.to_u64())
        .unwrap_or(60);
    let children_val = config
        .get("children")
        .ok_or_else(|| LxError::runtime("agent.supervise: config missing 'children'", span))?;
    let children_list = children_val
        .as_list()
        .ok_or_else(|| LxError::type_err("agent.supervise: 'children' must be a List", span))?;
    let mut children = Vec::new();
    for child_val in children_list.as_ref() {
        let child_spec = parse_child_spec(child_val, span, ctx)?;
        children.push(child_spec);
    }
    let restart_counts = vec![0; children.len()];
    let sup_id = NEXT_SUP_ID.fetch_add(1, Ordering::Relaxed);
    SUPERVISORS.insert(
        sup_id,
        Supervisor {
            strategy,
            max_restarts,
            _window: window,
            children,
            restart_counts,
        },
    );
    Ok(Value::Ok(Box::new(record! {
        "__sup_id" => Value::Int(BigInt::from(sup_id)),
    })))
}

fn parse_child_spec(val: &Value, span: Span, ctx: &Arc<RuntimeCtx>) -> Result<ChildSpec, LxError> {
    let Value::Record(r) = val else {
        return Err(LxError::type_err(
            "agent.supervise: child spec must be a Record",
            span,
        ));
    };
    let id = r
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("agent.supervise: child spec missing 'id' (Str)", span))?
        .to_string();
    let spawn_fn = r
        .get("spawn")
        .ok_or_else(|| LxError::runtime("agent.supervise: child spec missing 'spawn' (Fn)", span))?
        .clone();
    let restart = r
        .get("restart")
        .and_then(|v| v.as_str())
        .map(|s| s.trim_start_matches(':').to_string())
        .unwrap_or_else(|| "permanent".into());
    let current = call_value(&spawn_fn, Value::Unit, span, ctx)?;
    Ok(ChildSpec {
        id,
        spawn_fn,
        restart,
        current,
    })
}

fn bi_child(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sup_id = sup_id_from(&args[0], span)?;
    let child_id = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("agent.child: second arg must be child id (Str)", span))?;
    let mut sup = SUPERVISORS
        .get_mut(&sup_id)
        .ok_or_else(|| LxError::runtime("agent.child: supervisor not found", span))?;
    let idx = sup
        .children
        .iter()
        .position(|c| c.id == child_id)
        .ok_or_else(|| {
            LxError::runtime(format!("agent.child: child '{child_id}' not found"), span)
        })?;
    if needs_restart(&sup.children[idx]) {
        if sup.restart_counts[idx] >= sup.max_restarts {
            return Ok(Value::Err(Box::new(record! {
                "type" => Value::Str(Arc::from("supervisor_exhausted")),
                "id" => Value::Str(Arc::from(child_id)),
                "restarts" => Value::Int(BigInt::from(sup.restart_counts[idx])),
            })));
        }
        let strategy = sup.strategy.clone();
        let len = sup.children.len();
        match strategy.as_str() {
            "one_for_all" => restart_range(&mut sup, 0..len, span, ctx)?,
            "rest_for_one" => restart_range(&mut sup, idx..len, span, ctx)?,
            _ => restart_range(&mut sup, idx..idx + 1, span, ctx)?,
        }
    }
    Ok(sup.children[idx].current.clone())
}

fn needs_restart(child: &ChildSpec) -> bool {
    if child.restart == "temporary" {
        return false;
    }
    match &child.current {
        Value::Err(_) => true,
        Value::Record(r) => {
            if let Some(pid_val) = r.get("__pid")
                && let Some(pid) = pid_val.as_int().and_then(|n| n.to_u32())
            {
                return !super::agent::REGISTRY.contains_key(&pid);
            }
            false
        }
        _ => false,
    }
}

fn restart_range(
    sup: &mut Supervisor,
    range: std::ops::Range<usize>,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<(), LxError> {
    for i in range {
        let spawn_fn = sup.children[i].spawn_fn.clone();
        let new_agent = call_value(&spawn_fn, Value::Unit, span, ctx)?;
        sup.children[i].current = new_agent;
        sup.restart_counts[i] += 1;
    }
    Ok(())
}

fn bi_supervise_stop(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sup_id = sup_id_from(&args[0], span)?;
    if let Some((_, sup)) = SUPERVISORS.remove(&sup_id) {
        for child in sup.children.iter().rev() {
            if let Value::Record(r) = &child.current
                && let Some(pid_val) = r.get("__pid")
                && let Some(pid) = pid_val.as_int().and_then(|n| n.to_u32())
                && let Some((_, mut agent)) = super::agent::REGISTRY.remove(&pid)
            {
                if let Err(e) = agent._child.kill() {
                    eprintln!("supervise_stop: kill failed for pid {pid}: {e}");
                }
                if let Err(e) = agent._child.wait() {
                    eprintln!("supervise_stop: wait failed for pid {pid}: {e}");
                }
            }
        }
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}
