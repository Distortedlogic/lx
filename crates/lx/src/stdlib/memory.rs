#[path = "memory_store.rs"]
mod memory_store;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use memory_store::{entry_to_value, load_entries, persist};

pub(super) struct MemoryStore {
    pub entries: IndexMap<String, MemEntry>,
    pub path: PathBuf,
    pub next_entry_id: u64,
}

pub(super) struct MemEntry {
    pub content: String,
    pub tier: i64,
    pub confidence: f64,
    pub tags: Vec<String>,
    pub created_at: String,
    pub confirmed: i64,
    pub contradicted: i64,
}

pub(super) static STORES: LazyLock<DashMap<u64, MemoryStore>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("memory.create", 1, bi_create));
    m.insert("store".into(), mk("memory.store", 2, bi_store));
    m.insert("recall".into(), mk("memory.recall", 2, bi_recall));
    m.insert("promote".into(), mk("memory.promote", 2, bi_promote));
    m.insert("demote".into(), mk("memory.demote", 2, bi_demote));
    m.insert("forget".into(), mk("memory.forget", 2, bi_forget));
    m.insert(
        "consolidate".into(),
        mk("memory.consolidate", 1, bi_consolidate),
    );
    m.insert("tier".into(), mk("memory.tier", 2, memory_store::bi_tier));
    m.insert("all".into(), mk("memory.all", 1, memory_store::bi_all));
    m
}

pub(super) fn store_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__mem_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("memory: expected memory store record", span)),
        _ => Err(LxError::type_err("memory: expected Record", span)),
    }
}

fn make_handle(id: u64) -> Value {
    Value::Ok(Box::new(record! {
        "__mem_id" => Value::Int(BigInt::from(id)),
    }))
}

fn now_str() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("memory.create expects Str path", span))?;
    let (entries, next_entry_id) = load_entries(path, span)?;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    STORES.insert(
        id,
        MemoryStore {
            entries,
            path: PathBuf::from(path),
            next_entry_id,
        },
    );
    Ok(make_handle(id))
}

fn bi_store(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err(
            "memory.store: first arg must be Record",
            span,
        ));
    };
    let sid = store_id(&args[1], span)?;
    let content = fields
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("memory.store: missing 'content'", span))?
        .to_string();
    let tier = fields
        .get("tier")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
        .unwrap_or(0i64);
    let confidence = fields
        .get("confidence")
        .and_then(|v| v.as_float())
        .unwrap_or(0.3);
    let tags = fields
        .get("tags")
        .and_then(|v| v.as_list())
        .map(|l| {
            l.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let entry_id = store.next_entry_id.to_string();
    store.next_entry_id += 1;
    store.entries.insert(
        entry_id.clone(),
        MemEntry {
            content,
            tier,
            confidence,
            tags,
            created_at: now_str(),
            confirmed: 0,
            contradicted: 0,
        },
    );
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Str(Arc::from(
        entry_id.as_str(),
    )))))
}

fn bi_recall(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let query = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("memory.recall: query must be Str", span))?;
    let sid = store_id(&args[1], span)?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let query_lower = query.to_lowercase();
    let keywords: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .collect();
    let mut scored: Vec<(f64, &str, &MemEntry)> = store
        .entries
        .iter()
        .filter_map(|(id, e)| {
            let content_lower = e.content.to_lowercase();
            let hits = keywords
                .iter()
                .filter(|kw| content_lower.contains(**kw))
                .count();
            if hits == 0 {
                return None;
            }
            let keyword_score = hits as f64 / keywords.len().max(1) as f64;
            let tier_boost = e.tier as f64 * 0.1;
            let score = keyword_score + tier_boost + e.confidence * 0.1;
            Some((score, id.as_str(), e))
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let results: Vec<Value> = scored
        .iter()
        .map(|(_, id, e)| entry_to_value(id, e))
        .collect();
    Ok(Value::List(Arc::new(results)))
}

fn bi_promote(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let entry_id = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("memory.promote: id must be Str", span))?;
    let sid = store_id(&args[1], span)?;
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let entry = store
        .entries
        .get_mut(entry_id)
        .ok_or_else(|| LxError::runtime(format!("memory: entry '{entry_id}' not found"), span))?;
    entry.confirmed += 1;
    entry.confidence = (entry.confidence + 0.15).min(1.0);
    if entry.confidence >= 0.7 && entry.tier < 2 {
        entry.tier += 1;
    } else if entry.confidence >= 0.95 && entry.tier < 3 {
        entry.tier = 3;
    }
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_demote(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let entry_id = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("memory.demote: id must be Str", span))?;
    let sid = store_id(&args[1], span)?;
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let entry = store
        .entries
        .get_mut(entry_id)
        .ok_or_else(|| LxError::runtime(format!("memory: entry '{entry_id}' not found"), span))?;
    entry.contradicted += 1;
    entry.confidence = (entry.confidence - 0.2).max(0.0);
    if entry.confidence < 0.3 && entry.tier > 0 {
        entry.tier -= 1;
    }
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_forget(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let entry_id = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("memory.forget: id must be Str", span))?;
    let sid = store_id(&args[1], span)?;
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    store.entries.shift_remove(entry_id);
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_consolidate(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let mut promoted = 0i64;
    let mut demoted = 0i64;
    let mut removed = Vec::new();
    for (id, entry) in store.entries.iter_mut() {
        if entry.confirmed >= 3 && entry.tier < 3 {
            entry.tier += 1;
            entry.confidence = (entry.confidence + 0.1).min(1.0);
            promoted += 1;
        }
        if entry.contradicted >= 2 && entry.tier > 0 {
            entry.tier -= 1;
            entry.confidence = (entry.confidence - 0.1).max(0.0);
            demoted += 1;
        }
        if entry.confidence <= 0.0 && entry.contradicted > entry.confirmed {
            removed.push(id.clone());
        }
    }
    for id in &removed {
        store.entries.shift_remove(id);
    }
    persist(&store, span)?;
    Ok(record! {
        "promoted" => Value::Int(BigInt::from(promoted)),
        "demoted" => Value::Int(BigInt::from(demoted)),
        "removed" => Value::Int(BigInt::from(removed.len() as i64)),
    })
}
