use std::sync::Arc;
use std::sync::atomic::Ordering;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::LxVal;

use super::agent::REGISTRY;
use super::agent_dialogue::SESSIONS;
use super::agent_pubsub::TOPICS;
use super::agent_route_table::ROUTE_TABLE;
use super::agent_supervise::SUPERVISORS;

pub fn build() -> IndexMap<String, LxVal> {
    let mut m = IndexMap::new();
    m.insert("system".into(), mk("introspect.system", 1, bi_system));
    m.insert("agents".into(), mk("introspect.agents", 1, bi_agents));
    m.insert("agent".into(), mk("introspect.agent", 1, bi_agent));
    m.insert("messages".into(), mk("introspect.messages", 1, bi_messages));
    m.insert(
        "bottleneck".into(),
        mk("introspect.bottleneck", 1, bi_bottleneck),
    );
    m
}

fn agent_info(pid: u32) -> LxVal {
    let Some(ap) = REGISTRY.get(&pid) else {
        return LxVal::None;
    };
    let uptime_ms = ap.spawned_at.elapsed().as_millis() as i64;
    let in_flight = ap.in_flight.load(Ordering::Relaxed);
    let completed = ap.completed.load(Ordering::Relaxed);
    let errors = ap.errors.load(Ordering::Relaxed);
    let traits: Vec<LxVal> = ap
        .traits
        .iter()
        .map(|t| LxVal::Str(Arc::from(t.as_str())))
        .collect();
    let mut dialogues = Vec::new();
    for entry in SESSIONS.iter() {
        let session = entry.value();
        let is_match = match &session.agent {
            LxVal::Record(r) => r
                .get("__pid")
                .and_then(|v| v.as_int())
                .and_then(|n| n.try_into().ok())
                .is_some_and(|p: u32| p == pid),
            _ => false,
        };
        if is_match {
            let mut d = IndexMap::new();
            d.insert("id".into(), LxVal::Int(BigInt::from(*entry.key())));
            d.insert(
                "turns".into(),
                LxVal::Int(BigInt::from(session.history.len() / 2)),
            );
            if let Some(ref role) = session.role {
                d.insert("role".into(), LxVal::Str(Arc::from(role.as_str())));
            }
            dialogues.push(LxVal::Record(Arc::new(d)));
        }
    }
    let mut route_load = 0u64;
    for entry in ROUTE_TABLE.iter() {
        if let LxVal::Record(r) = &entry.value().agent
            && r.get("__pid")
                .and_then(|v| v.as_int())
                .and_then(|n| n.try_into().ok())
                == Some(pid)
        {
            route_load = entry.value().load.load(Ordering::Relaxed);
            break;
        }
    }
    let status = if in_flight > 0 { "busy" } else { "idle" };
    let mut rec = IndexMap::new();
    rec.insert("name".into(), LxVal::Str(Arc::from(ap.name.as_str())));
    rec.insert("status".into(), LxVal::Str(Arc::from(status)));
    rec.insert("pid".into(), LxVal::Int(BigInt::from(pid)));
    rec.insert("uptime_ms".into(), LxVal::Int(BigInt::from(uptime_ms)));
    rec.insert("in_flight".into(), LxVal::Int(BigInt::from(in_flight)));
    rec.insert("completed".into(), LxVal::Int(BigInt::from(completed)));
    rec.insert("errors".into(), LxVal::Int(BigInt::from(errors)));
    rec.insert("traits".into(), LxVal::List(Arc::new(traits)));
    rec.insert("dialogues".into(), LxVal::List(Arc::new(dialogues)));
    rec.insert("route_load".into(), LxVal::Int(BigInt::from(route_load)));
    LxVal::Record(Arc::new(rec))
}

fn collect_agents() -> Vec<LxVal> {
    REGISTRY
        .iter()
        .map(|entry| agent_info(*entry.key()))
        .collect()
}

fn collect_topics() -> Vec<LxVal> {
    TOPICS
        .iter()
        .map(|entry| {
            record! {
                "name" => LxVal::Str(Arc::from(entry.key().as_str())),
                "subscribers" => LxVal::Int(BigInt::from(entry.value().subscribers.len())),
            }
        })
        .collect()
}

fn collect_supervisors() -> Vec<LxVal> {
    SUPERVISORS
        .iter()
        .map(|entry| {
            let sup = entry.value();
            let total_restarts: usize = sup.restart_counts.iter().sum();
            record! {
                "id" => LxVal::Int(BigInt::from(*entry.key())),
                "strategy" => LxVal::Str(Arc::from(sup.strategy.as_str())),
                "children" => LxVal::Int(BigInt::from(sup.children.len())),
                "restarts" => LxVal::Int(BigInt::from(total_restarts)),
            }
        })
        .collect()
}

fn bi_system(_args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let agents = collect_agents();
    let total_in_flight: u64 = REGISTRY
        .iter()
        .map(|e| e.value().in_flight.load(Ordering::Relaxed))
        .sum();
    let topics = collect_topics();
    let supervisors = collect_supervisors();
    Ok(LxVal::Ok(Box::new(record! {
        "agents" => LxVal::List(Arc::new(agents)),
        "messages_in_flight" => LxVal::Int(BigInt::from(total_in_flight)),
        "topics" => LxVal::List(Arc::new(topics)),
        "supervisors" => LxVal::List(Arc::new(supervisors)),
    })))
}

fn bi_agents(_args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Ok(Box::new(LxVal::List(Arc::new(collect_agents())))))
}

fn bi_agent(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let pid = super::agent::get_pid(&args[0], span)?;
    let info = agent_info(pid);
    match info {
        LxVal::None => Ok(LxVal::Err(Box::new(super::agent_errors::unavailable(
            &format!("pid:{pid}"),
            "agent not found in registry",
        )))),
        other => Ok(LxVal::Ok(Box::new(other))),
    }
}

fn bi_messages(_args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let msgs: Vec<LxVal> = REGISTRY
        .iter()
        .filter_map(|entry| {
            let in_flight = entry.value().in_flight.load(Ordering::Relaxed);
            if in_flight > 0 {
                Some(record! {
                    "agent" => LxVal::Str(Arc::from(entry.value().name.as_str())),
                    "pid" => LxVal::Int(BigInt::from(*entry.key())),
                    "in_flight" => LxVal::Int(BigInt::from(in_flight)),
                })
            } else {
                None
            }
        })
        .collect();
    Ok(LxVal::Ok(Box::new(LxVal::List(Arc::new(msgs)))))
}

fn bi_bottleneck(_args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let mut max_load = 0u64;
    let mut max_pid = None;
    for entry in REGISTRY.iter() {
        let load = entry.value().in_flight.load(Ordering::Relaxed);
        if load > max_load {
            max_load = load;
            max_pid = Some(*entry.key());
        }
    }
    match max_pid {
        Some(pid) => {
            let ap = REGISTRY.get(&pid);
            match ap {
                Some(ap) => Ok(LxVal::Ok(Box::new(record! {
                    "agent" => LxVal::Str(Arc::from(ap.name.as_str())),
                    "pid" => LxVal::Int(BigInt::from(pid)),
                    "in_flight" => LxVal::Int(BigInt::from(max_load)),
                }))),
                None => Ok(LxVal::None),
            }
        }
        None => Ok(LxVal::None),
    }
}
