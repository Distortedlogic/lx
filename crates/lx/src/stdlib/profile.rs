use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::profile_io;

pub(crate) struct Profile {
    pub name: String,
    pub created: String,
    pub updated: String,
    pub knowledge: IndexMap<String, KnowledgeEntry>,
    pub preferences: IndexMap<String, Value>,
}

#[derive(Clone)]
pub(crate) struct KnowledgeEntry {
    pub data: Value,
    pub learned_at: String,
}

pub(crate) static PROFILES: LazyLock<DashMap<u64, Profile>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("load".into(), mk("profile.load", 1, bi_load));
    m.insert("save".into(), mk("profile.save", 1, bi_save));
    m.insert("learn".into(), mk("profile.learn", 3, bi_learn));
    m.insert("recall".into(), mk("profile.recall", 2, bi_recall));
    m.insert(
        "recall_prefix".into(),
        mk("profile.recall_prefix", 2, bi_recall_prefix),
    );
    m.insert("forget".into(), mk("profile.forget", 2, bi_forget));
    m.insert(
        "preference".into(),
        mk("profile.preference", 3, bi_preference),
    );
    m.insert(
        "get_preference".into(),
        mk("profile.get_preference", 2, bi_get_preference),
    );
    m.insert(
        "history".into(),
        mk("profile.history", 1, profile_io::bi_history),
    );
    m.insert(
        "merge".into(),
        mk("profile.merge", 2, profile_io::bi_merge),
    );
    m.insert("age".into(), mk("profile.age", 2, profile_io::bi_age));
    m.insert(
        "decay".into(),
        mk("profile.decay", 2, profile_io::bi_decay),
    );
    super::profile_strategy::register(&mut m);
    m
}

pub(crate) fn profile_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__profile_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("profile: expected Profile handle", span)),
        _ => Err(LxError::type_err("profile: expected Profile Record", span)),
    }
}

fn make_handle(id: u64, name: &str) -> Value {
    let mut rec = IndexMap::new();
    rec.insert("__profile_id".into(), Value::Int(BigInt::from(id)));
    rec.insert("name".into(), Value::Str(Arc::from(name)));
    Value::Record(Arc::new(rec))
}

pub(crate) fn now_str() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn bi_load(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let (name, create) = match &args[0] {
        Value::Str(s) => (s.to_string(), false),
        Value::Record(r) if r.contains_key("name") => {
            let nm = r
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let cr = r
                .get("create")
                .map(|v| matches!(v, Value::Bool(true)))
                .unwrap_or(false);
            (nm, cr)
        }
        _ => return Err(LxError::type_err("profile.load: expected Str name", span)),
    };
    match profile_io::load_from_disk(&name, span)? {
        Some(p) => {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let handle = make_handle(id, &name);
            PROFILES.insert(id, p);
            Ok(Value::Ok(Box::new(handle)))
        }
        None if create => {
            let now = now_str();
            let p = Profile {
                name: name.clone(),
                created: now.clone(),
                updated: now,
                knowledge: IndexMap::new(),
                preferences: IndexMap::new(),
            };
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let handle = make_handle(id, &name);
            PROFILES.insert(id, p);
            Ok(Value::Ok(Box::new(handle)))
        }
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("profile '{name}' not found").as_str(),
        ))))),
    }
}

fn bi_save(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let mut p = PROFILES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    p.updated = now_str();
    profile_io::persist(&p, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_learn(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let domain = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("profile.learn: domain must be Str", span))?;
    let data = &args[2];
    let now = now_str();
    let entry_data = add_learned_at(data, &now);
    let mut p = PROFILES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    p.knowledge.insert(
        domain.to_string(),
        KnowledgeEntry {
            data: entry_data,
            learned_at: now,
        },
    );
    Ok(Value::Unit)
}

fn add_learned_at(data: &Value, timestamp: &str) -> Value {
    match data {
        Value::Record(fields) => {
            let mut f = (**fields).clone();
            f.insert("learned_at".into(), Value::Str(Arc::from(timestamp)));
            Value::Record(Arc::new(f))
        }
        _ => data.clone(),
    }
}

fn bi_recall(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let domain = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("profile.recall: domain must be Str", span))?;
    let p = PROFILES
        .get(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    match p.knowledge.get(domain) {
        Some(entry) => Ok(Value::Ok(Box::new(entry.data.clone()))),
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("domain '{domain}' not found").as_str(),
        ))))),
    }
}

fn bi_recall_prefix(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let prefix = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("profile.recall_prefix: prefix must be Str", span))?;
    let p = PROFILES
        .get(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    let results: Vec<Value> = p
        .knowledge
        .iter()
        .filter(|(k, _)| k.starts_with(prefix))
        .map(|(_, e)| e.data.clone())
        .collect();
    Ok(Value::List(Arc::new(results)))
}

fn bi_forget(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let domain = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("profile.forget: domain must be Str", span))?;
    let mut p = PROFILES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    p.knowledge.shift_remove(domain);
    Ok(Value::Unit)
}

fn bi_preference(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("profile.preference: key must be Str", span))?;
    let val = args[2].clone();
    let mut p = PROFILES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    p.preferences.insert(key.to_string(), val);
    Ok(Value::Unit)
}

fn bi_get_preference(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("profile.get_preference: key must be Str", span))?;
    let p = PROFILES
        .get(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    match p.preferences.get(key) {
        Some(v) => Ok(Value::Ok(Box::new(v.clone()))),
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("preference '{key}' not found").as_str(),
        ))))),
    }
}
