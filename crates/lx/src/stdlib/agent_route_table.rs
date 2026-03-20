use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub(super) struct RoutingEntry {
    pub(super) agent: Value,
    traits: Vec<String>,
    protocols: Vec<String>,
    domains: Vec<String>,
    pub(super) max_concurrent: usize,
    pub(super) load: AtomicU64,
}

impl RoutingEntry {
    pub(super) fn new(
        agent: Value,
        traits: Vec<String>,
        protocols: Vec<String>,
        domains: Vec<String>,
        max_concurrent: usize,
    ) -> Self {
        Self {
            agent,
            traits,
            protocols,
            domains,
            max_concurrent,
            load: AtomicU64::new(0),
        }
    }

    pub(super) fn has_trait(&self, t: &str) -> bool {
        self.traits.iter().any(|et| et == t)
    }

    pub(super) fn has_protocol(&self, p: &str) -> bool {
        self.protocols.iter().any(|ep| ep == p)
    }

    pub(super) fn has_domain(&self, d: &str) -> bool {
        self.domains.iter().any(|ed| ed == d)
    }
}

pub(super) static ROUTE_TABLE: std::sync::LazyLock<DashMap<String, RoutingEntry>> =
    std::sync::LazyLock::new(DashMap::new);
static NEXT_ROUTE_ID: AtomicU64 = AtomicU64::new(1);
static ROUND_ROBIN: AtomicU64 = AtomicU64::new(0);

pub(super) fn agent_key(agent: &Value) -> String {
    match agent {
        Value::Record(r) => {
            if let Some(Value::Str(name)) = r.get("name") {
                return name.to_string();
            }
            if let Some(pid) = r.get("__pid").and_then(|v| v.as_int()) {
                return format!("pid:{pid}");
            }
            if let Some(mid) = r.get("__mock_id").and_then(|v| v.as_int()) {
                return format!("mock:{mid}");
            }
            format!("anon:{}", NEXT_ROUTE_ID.fetch_add(1, Ordering::Relaxed))
        }
        Value::Class { name, .. } => name.to_string(),
        _ => format!("anon:{}", NEXT_ROUTE_ID.fetch_add(1, Ordering::Relaxed)),
    }
}

pub(super) fn str_list_from(val: Option<&Value>) -> Vec<String> {
    match val {
        Some(Value::List(list)) => list
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

fn matches_entry(
    entry: &RoutingEntry,
    trait_f: Option<&str>,
    proto_f: Option<&str>,
    domain_f: Option<&str>,
) -> bool {
    if let Some(t) = trait_f
        && !entry.traits.iter().any(|et| et == t)
    {
        return false;
    }
    if let Some(p) = proto_f
        && !entry.protocols.iter().any(|ep| ep == p)
    {
        return false;
    }
    if let Some(d) = domain_f
        && !entry.domains.iter().any(|ed| ed == d)
    {
        return false;
    }
    true
}

pub(super) struct Candidate {
    pub(super) key: String,
    pub(super) agent: Value,
    pub(super) load: u64,
}

pub(super) fn find_candidates(
    opts: &IndexMap<String, Value>,
    exclude: &[String],
    check_capacity: bool,
) -> Vec<Candidate> {
    let trait_f = opts.get("trait").and_then(|v| v.as_str());
    let proto_f = opts.get("protocol").and_then(|v| v.as_str());
    let domain_f = opts.get("domain").and_then(|v| v.as_str());
    ROUTE_TABLE
        .iter()
        .filter(|e| matches_entry(e.value(), trait_f, proto_f, domain_f))
        .filter(|e| !exclude.contains(e.key()))
        .filter(|e| {
            !check_capacity
                || e.value().load.load(Ordering::Relaxed) < e.value().max_concurrent as u64
        })
        .map(|e| Candidate {
            key: e.key().clone(),
            agent: e.value().agent.clone(),
            load: e.value().load.load(Ordering::Relaxed),
        })
        .collect()
}

pub(super) fn select_agent(
    candidates: &[Candidate],
    opts: &IndexMap<String, Value>,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<usize, LxError> {
    let prefer = opts.get("prefer");
    match prefer.and_then(|v| v.as_str()) {
        None | Some("least_busy") => Ok(candidates
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| c.load)
            .map(|(i, _)| i)
            .unwrap_or(0)),
        Some("round_robin") => {
            Ok(ROUND_ROBIN.fetch_add(1, Ordering::Relaxed) as usize % candidates.len())
        }
        Some("random") => {
            let ns = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as usize)
                .unwrap_or(0);
            Ok(ns % candidates.len())
        }
        _ => {
            if let Some(f @ (Value::Func(_) | Value::BuiltinFunc(_))) = prefer {
                let agents: Vec<Value> = candidates.iter().map(|c| c.agent.clone()).collect();
                let selected = call_value_sync(f, Value::List(Arc::new(agents)), span, ctx)?;
                let sel_key = agent_key(&selected);
                Ok(candidates
                    .iter()
                    .position(|c| c.key == sel_key)
                    .unwrap_or(0))
            } else {
                let s = prefer.map(|v| format!("{v}")).unwrap_or_default();
                Err(LxError::runtime(
                    format!("agent.route: unknown strategy '{s}'"),
                    span,
                ))
            }
        }
    }
}

pub(super) fn send_to_agent(
    agent: &Value,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    match agent {
        Value::Record(r) => {
            if let Some(handler) = r.get("handler") {
                let result = call_value_sync(handler, msg.clone(), span, ctx)?;
                return Ok(Value::Ok(Box::new(result)));
            }
            if let Some(pid_val) = r.get("__pid") {
                let pid: u32 = pid_val
                    .as_int()
                    .and_then(|n| n.try_into().ok())
                    .ok_or_else(|| LxError::type_err("agent.route: invalid __pid", span))?;
                return super::agent::ask_subprocess(pid, msg, span);
            }
            Err(LxError::runtime(
                "agent.route: agent has no handler or __pid",
                span,
            ))
        }
        _ => Err(LxError::type_err(
            "agent.route: expected agent Record",
            span,
        )),
    }
}

pub(super) fn with_load_tracking<F: FnOnce() -> Result<Value, LxError>>(
    key: &str,
    f: F,
) -> Result<Value, LxError> {
    if let Some(entry) = ROUTE_TABLE.get(key) {
        entry.load.fetch_add(1, Ordering::Relaxed);
    }
    let result = f();
    if let Some(entry) = ROUTE_TABLE.get(key) {
        entry.load.fetch_sub(1, Ordering::Relaxed);
    }
    result
}
