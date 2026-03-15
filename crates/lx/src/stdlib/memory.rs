use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

struct MemoryStore {
    entries: IndexMap<String, MemEntry>,
    path: PathBuf,
    next_entry_id: u64,
}

struct MemEntry {
    content: String,
    tier: i64,
    confidence: f64,
    tags: Vec<String>,
    created_at: String,
    confirmed: i64,
    contradicted: i64,
}

static STORES: LazyLock<DashMap<u64, MemoryStore>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("memory.create", 1, bi_create));
    m.insert("store".into(), mk("memory.store", 2, bi_store));
    m.insert("recall".into(), mk("memory.recall", 2, bi_recall));
    m.insert("promote".into(), mk("memory.promote", 2, bi_promote));
    m.insert("demote".into(), mk("memory.demote", 2, bi_demote));
    m.insert("forget".into(), mk("memory.forget", 2, bi_forget));
    m.insert("consolidate".into(), mk("memory.consolidate", 1, bi_consolidate));
    m.insert("tier".into(), mk("memory.tier", 2, bi_tier));
    m.insert("all".into(), mk("memory.all", 1, bi_all));
    m
}

fn store_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r.get("__mem_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("memory: expected memory store record", span)),
        _ => Err(LxError::type_err("memory: expected Record", span)),
    }
}

fn make_handle(id: u64) -> Value {
    let mut rec = IndexMap::new();
    rec.insert("__mem_id".into(), Value::Int(BigInt::from(id)));
    Value::Ok(Box::new(Value::Record(Arc::new(rec))))
}

fn now_str() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn entry_to_value(id: &str, e: &MemEntry) -> Value {
    let mut f = IndexMap::new();
    f.insert("id".into(), Value::Str(Arc::from(id)));
    f.insert("content".into(), Value::Str(Arc::from(e.content.as_str())));
    f.insert("tier".into(), Value::Int(BigInt::from(e.tier)));
    f.insert("confidence".into(), Value::Float(e.confidence));
    let tags: Vec<Value> = e.tags.iter()
        .map(|t| Value::Str(Arc::from(t.as_str()))).collect();
    f.insert("tags".into(), Value::List(Arc::new(tags)));
    f.insert("created_at".into(), Value::Str(Arc::from(e.created_at.as_str())));
    f.insert("confirmed".into(), Value::Int(BigInt::from(e.confirmed)));
    f.insert("contradicted".into(), Value::Int(BigInt::from(e.contradicted)));
    Value::Record(Arc::new(f))
}

fn persist(store: &MemoryStore, span: Span) -> Result<(), LxError> {
    let items: Vec<Value> = store.entries.iter()
        .map(|(id, e)| entry_to_value(id, e)).collect();
    let list = Value::List(Arc::new(items));
    let json = json_conv::lx_to_json(&list, span)?;
    let s = serde_json::to_string_pretty(&json)
        .map_err(|e| LxError::runtime(format!("memory: serialize: {e}"), span))?;
    std::fs::write(&store.path, s)
        .map_err(|e| LxError::runtime(format!("memory: write: {e}"), span))
}

fn load_entries(path: &str, span: Span) -> Result<(IndexMap<String, MemEntry>, u64), LxError> {
    let mut entries = IndexMap::new();
    let mut max_id: u64 = 0;
    if !std::path::Path::new(path).exists() {
        return Ok((entries, 1));
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| LxError::runtime(format!("memory: read: {e}"), span))?;
    let jv: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| LxError::runtime(format!("memory: JSON: {e}"), span))?;
    let Value::List(items) = json_conv::json_to_lx(jv) else {
        return Err(LxError::runtime("memory: expected JSON array", span));
    };
    for item in items.iter() {
        let Value::Record(r) = item else { continue };
        let id = r.get("id").and_then(|v| v.as_str()).unwrap_or("0").to_string();
        if let Ok(n) = id.parse::<u64>()
            && n >= max_id { max_id = n + 1; }
        let tags = r.get("tags").and_then(|v| v.as_list())
            .map(|l| l.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        entries.insert(id, MemEntry {
            content: r.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            tier: r.get("tier").and_then(|v| v.as_int())
                .and_then(|n| n.try_into().ok()).unwrap_or(0),
            confidence: r.get("confidence").and_then(|v| v.as_float()).unwrap_or(0.0),
            tags,
            created_at: r.get("created_at").and_then(|v| v.as_str())
                .unwrap_or("").to_string(),
            confirmed: r.get("confirmed").and_then(|v| v.as_int())
                .and_then(|n| n.try_into().ok()).unwrap_or(0),
            contradicted: r.get("contradicted").and_then(|v| v.as_int())
                .and_then(|n| n.try_into().ok()).unwrap_or(0),
        });
    }
    Ok((entries, max_id))
}

fn bi_create(args: &[Value], span: Span) -> Result<Value, LxError> {
    let path = args[0].as_str()
        .ok_or_else(|| LxError::type_err("memory.create expects Str path", span))?;
    let (entries, next_entry_id) = load_entries(path, span)?;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    STORES.insert(id, MemoryStore { entries, path: PathBuf::from(path), next_entry_id });
    Ok(make_handle(id))
}

fn bi_store(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("memory.store: first arg must be Record", span));
    };
    let sid = store_id(&args[1], span)?;
    let content = fields.get("content").and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("memory.store: missing 'content'", span))?
        .to_string();
    let tier = fields.get("tier").and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok()).unwrap_or(0i64);
    let confidence = fields.get("confidence").and_then(|v| v.as_float()).unwrap_or(0.3);
    let tags = fields.get("tags").and_then(|v| v.as_list())
        .map(|l| l.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let mut store = STORES.get_mut(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let entry_id = store.next_entry_id.to_string();
    store.next_entry_id += 1;
    store.entries.insert(entry_id.clone(), MemEntry {
        content, tier, confidence, tags,
        created_at: now_str(), confirmed: 0, contradicted: 0,
    });
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Str(Arc::from(entry_id.as_str())))))
}

fn bi_recall(args: &[Value], span: Span) -> Result<Value, LxError> {
    let query = args[0].as_str()
        .ok_or_else(|| LxError::type_err("memory.recall: query must be Str", span))?;
    let sid = store_id(&args[1], span)?;
    let store = STORES.get(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let query_lower = query.to_lowercase();
    let keywords: Vec<&str> = query_lower.split_whitespace()
        .filter(|w| w.len() > 2).collect();
    let mut scored: Vec<(f64, &str, &MemEntry)> = store.entries.iter()
        .filter_map(|(id, e)| {
            let content_lower = e.content.to_lowercase();
            let hits = keywords.iter()
                .filter(|kw| content_lower.contains(**kw)).count();
            if hits == 0 { return None; }
            let keyword_score = hits as f64 / keywords.len().max(1) as f64;
            let tier_boost = e.tier as f64 * 0.1;
            let score = keyword_score + tier_boost + e.confidence * 0.1;
            Some((score, id.as_str(), e))
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let results: Vec<Value> = scored.iter()
        .map(|(_, id, e)| entry_to_value(id, e)).collect();
    Ok(Value::List(Arc::new(results)))
}

fn bi_promote(args: &[Value], span: Span) -> Result<Value, LxError> {
    let entry_id = args[0].as_str()
        .ok_or_else(|| LxError::type_err("memory.promote: id must be Str", span))?;
    let sid = store_id(&args[1], span)?;
    let mut store = STORES.get_mut(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let entry = store.entries.get_mut(entry_id)
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

fn bi_demote(args: &[Value], span: Span) -> Result<Value, LxError> {
    let entry_id = args[0].as_str()
        .ok_or_else(|| LxError::type_err("memory.demote: id must be Str", span))?;
    let sid = store_id(&args[1], span)?;
    let mut store = STORES.get_mut(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let entry = store.entries.get_mut(entry_id)
        .ok_or_else(|| LxError::runtime(format!("memory: entry '{entry_id}' not found"), span))?;
    entry.contradicted += 1;
    entry.confidence = (entry.confidence - 0.2).max(0.0);
    if entry.confidence < 0.3 && entry.tier > 0 {
        entry.tier -= 1;
    }
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_forget(args: &[Value], span: Span) -> Result<Value, LxError> {
    let entry_id = args[0].as_str()
        .ok_or_else(|| LxError::type_err("memory.forget: id must be Str", span))?;
    let sid = store_id(&args[1], span)?;
    let mut store = STORES.get_mut(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    store.entries.shift_remove(entry_id);
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_consolidate(args: &[Value], span: Span) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let mut store = STORES.get_mut(&sid)
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
    let mut r = IndexMap::new();
    r.insert("promoted".into(), Value::Int(BigInt::from(promoted)));
    r.insert("demoted".into(), Value::Int(BigInt::from(demoted)));
    r.insert("removed".into(), Value::Int(BigInt::from(removed.len() as i64)));
    Ok(Value::Record(Arc::new(r)))
}

fn bi_tier(args: &[Value], span: Span) -> Result<Value, LxError> {
    let level = args[0].as_int()
        .and_then(|n| -> Option<i64> { n.try_into().ok() })
        .ok_or_else(|| LxError::type_err("memory.tier: level must be Int", span))?;
    let sid = store_id(&args[1], span)?;
    let store = STORES.get(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let results: Vec<Value> = store.entries.iter()
        .filter(|(_, e)| e.tier == level)
        .map(|(id, e)| entry_to_value(id, e))
        .collect();
    Ok(Value::List(Arc::new(results)))
}

fn bi_all(args: &[Value], span: Span) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let store = STORES.get(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let results: Vec<Value> = store.entries.iter()
        .map(|(id, e)| entry_to_value(id, e)).collect();
    Ok(Value::List(Arc::new(results)))
}
