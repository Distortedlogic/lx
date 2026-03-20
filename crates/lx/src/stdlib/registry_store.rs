use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Instant;

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub struct AgentEntry {
    pub name: String,
    pub traits: Vec<String>,
    pub protocols: Vec<String>,
    pub domains: Vec<String>,
    pub capacity: u64,
    pub metadata: Value,
    pub load: AtomicU64,
    pub registered_at: Instant,
    pub last_heartbeat: Instant,
    pub healthy: bool,
}

pub struct Watcher {
    pub query: WatchQuery,
    pub callback: Value,
}

pub struct WatchQuery {
    pub trait_filter: Option<String>,
    pub protocol_filter: Option<String>,
    pub domain_filter: Option<String>,
}

pub struct Registry {
    pub agents: DashMap<String, AgentEntry>,
    pub watchers: Vec<Watcher>,
    pub round_robin: AtomicU64,
}

pub static REGISTRIES: LazyLock<DashMap<u64, Registry>> = LazyLock::new(DashMap::new);
static NEXT_REG_ID: AtomicU64 = AtomicU64::new(1);

pub fn create_registry() -> u64 {
    let id = NEXT_REG_ID.fetch_add(1, Ordering::Relaxed);
    REGISTRIES.insert(
        id,
        Registry {
            agents: DashMap::new(),
            watchers: Vec::new(),
            round_robin: AtomicU64::new(0),
        },
    );
    id
}

pub fn reg_handle(id: u64) -> Value {
    record! {
        "__registry_id" => Value::Int(BigInt::from(id)),
    }
}

pub fn conn_handle(id: u64) -> Value {
    record! {
        "__registry_id" => Value::Int(BigInt::from(id)),
    }
}

pub fn get_reg_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__registry_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| {
                LxError::type_err("registry: expected registry/connection handle", span)
            }),
        _ => Err(LxError::type_err("registry: expected Record handle", span)),
    }
}

pub fn entry_matches(entry: &AgentEntry, query: &Value) -> bool {
    let Value::Record(q) = query else {
        return true;
    };
    if q.get("trait")
        .and_then(|v| v.as_str())
        .is_some_and(|t| !entry.traits.iter().any(|et| et == t))
    {
        return false;
    }
    if q.get("protocol")
        .and_then(|v| v.as_str())
        .is_some_and(|p| !entry.protocols.iter().any(|ep| ep == p))
    {
        return false;
    }
    if q.get("domain")
        .and_then(|v| v.as_str())
        .is_some_and(|d| !entry.domains.iter().any(|ed| ed == d))
    {
        return false;
    }
    true
}

pub fn entry_to_agent_ref(entry: &AgentEntry) -> Value {
    let traits: Vec<Value> = entry
        .traits
        .iter()
        .map(|t| Value::Str(Arc::from(t.as_str())))
        .collect();
    let protocols: Vec<Value> = entry
        .protocols
        .iter()
        .map(|p| Value::Str(Arc::from(p.as_str())))
        .collect();
    let domains: Vec<Value> = entry
        .domains
        .iter()
        .map(|d| Value::Str(Arc::from(d.as_str())))
        .collect();
    let load = entry.load.load(Ordering::Relaxed);
    let mut rec = IndexMap::new();
    rec.insert("name".into(), Value::Str(Arc::from(entry.name.as_str())));
    rec.insert(
        "address".into(),
        Value::Str(Arc::from(format!("local://{}", entry.name).as_str())),
    );
    rec.insert("traits".into(), Value::List(Arc::new(traits)));
    rec.insert("protocols".into(), Value::List(Arc::new(protocols)));
    rec.insert("domains".into(), Value::List(Arc::new(domains)));
    rec.insert("capacity".into(), Value::Int(BigInt::from(entry.capacity)));
    rec.insert("load".into(), Value::Int(BigInt::from(load)));
    rec.insert("healthy".into(), Value::Bool(entry.healthy));
    rec.insert("metadata".into(), entry.metadata.clone());
    Value::Record(Arc::new(rec))
}

pub fn watcher_matches(wq: &WatchQuery, entry: &AgentEntry) -> bool {
    if wq
        .trait_filter
        .as_ref()
        .is_some_and(|t| !entry.traits.iter().any(|et| et == t))
    {
        return false;
    }
    if wq
        .protocol_filter
        .as_ref()
        .is_some_and(|p| !entry.protocols.iter().any(|ep| ep == p))
    {
        return false;
    }
    if wq
        .domain_filter
        .as_ref()
        .is_some_and(|d| !entry.domains.iter().any(|ed| ed == d))
    {
        return false;
    }
    true
}

pub fn parse_watch_query(query: &Value) -> WatchQuery {
    let mut wq = WatchQuery {
        trait_filter: None,
        protocol_filter: None,
        domain_filter: None,
    };
    if let Value::Record(q) = query {
        if let Some(Value::Str(t)) = q.get("trait") {
            wq.trait_filter = Some(t.to_string());
        }
        if let Some(Value::Str(p)) = q.get("protocol") {
            wq.protocol_filter = Some(p.to_string());
        }
        if let Some(Value::Str(d)) = q.get("domain") {
            wq.domain_filter = Some(d.to_string());
        }
    }
    wq
}
