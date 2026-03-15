use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

struct KnowledgeBase {
    entries: IndexMap<String, KBEntry>,
    path: PathBuf,
}

struct KBEntry {
    val: Value,
    meta: Value,
    stored_at: String,
}

static KBS: LazyLock<DashMap<u64, KnowledgeBase>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("knowledge.create", 1, bi_create));
    m.insert("store".into(), mk("knowledge.store", 4, bi_store));
    m.insert("get".into(), mk("knowledge.get", 2, bi_get));
    m.insert("query".into(), mk("knowledge.query", 2, bi_query));
    m.insert("keys".into(), mk("knowledge.keys", 1, bi_keys));
    m.insert("remove".into(), mk("knowledge.remove", 2, bi_remove));
    m.insert("merge".into(), mk("knowledge.merge", 2, bi_merge));
    m.insert("expire".into(), mk("knowledge.expire", 2, bi_expire));
    m
}

fn kb_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r.get("__kb_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("knowledge: expected KB record", span)),
        _ => Err(LxError::type_err("knowledge: expected KB Record", span)),
    }
}

fn make_handle(id: u64) -> Value {
    let mut rec = IndexMap::new();
    rec.insert("__kb_id".into(), Value::Int(BigInt::from(id)));
    Value::Ok(Box::new(Value::Record(Arc::new(rec))))
}

fn now_str() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn entry_to_value(key: &str, e: &KBEntry) -> Value {
    let mut fields = IndexMap::new();
    fields.insert("key".into(), Value::Str(Arc::from(key)));
    fields.insert("val".into(), e.val.clone());
    fields.insert("meta".into(), e.meta.clone());
    fields.insert("stored_at".into(), Value::Str(Arc::from(e.stored_at.as_str())));
    Value::Record(Arc::new(fields))
}

fn persist(kb: &KnowledgeBase, span: Span) -> Result<(), LxError> {
    let items: Vec<Value> = kb.entries.iter()
        .map(|(k, e)| entry_to_value(k, e))
        .collect();
    let list = Value::List(Arc::new(items));
    let json = json_conv::lx_to_json(&list, span)?;
    let s = serde_json::to_string_pretty(&json)
        .map_err(|e| LxError::runtime(format!("knowledge: serialize: {e}"), span))?;
    std::fs::write(&kb.path, s)
        .map_err(|e| LxError::runtime(format!("knowledge: write: {e}"), span))
}

fn load_entries(path: &str, span: Span) -> Result<IndexMap<String, KBEntry>, LxError> {
    let mut entries = IndexMap::new();
    if !std::path::Path::new(path).exists() {
        return Ok(entries);
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| LxError::runtime(format!("knowledge: read: {e}"), span))?;
    let jv: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| LxError::runtime(format!("knowledge: JSON: {e}"), span))?;
    let Value::List(items) = json_conv::json_to_lx(jv) else {
        return Err(LxError::runtime("knowledge: expected JSON array", span));
    };
    for item in items.iter() {
        if let Value::Record(r) = item {
            let key = r.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let val = r.get("val").cloned().unwrap_or(Value::Unit);
            let meta = r.get("meta").cloned().unwrap_or(Value::Record(Arc::new(IndexMap::new())));
            let stored_at = r.get("stored_at").and_then(|v| v.as_str())
                .unwrap_or("").to_string();
            entries.insert(key, KBEntry { val, meta, stored_at });
        }
    }
    Ok(entries)
}

fn bi_create(args: &[Value], span: Span) -> Result<Value, LxError> {
    let path = args[0].as_str()
        .ok_or_else(|| LxError::type_err("knowledge.create expects Str path", span))?;
    let entries = load_entries(path, span)?;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    KBS.insert(id, KnowledgeBase { entries, path: PathBuf::from(path) });
    Ok(make_handle(id))
}

fn bi_store(args: &[Value], span: Span) -> Result<Value, LxError> {
    let key = args[0].as_str()
        .ok_or_else(|| LxError::type_err("knowledge.store: key must be Str", span))?;
    let val = args[1].clone();
    let meta = args[2].clone();
    let id = kb_id(&args[3], span)?;
    let mut kb = KBS.get_mut(&id)
        .ok_or_else(|| LxError::runtime("knowledge: KB not found", span))?;
    kb.entries.insert(key.to_string(), KBEntry {
        val,
        meta,
        stored_at: now_str(),
    });
    persist(&kb, span)?;
    Ok(make_handle(id))
}

fn bi_get(args: &[Value], span: Span) -> Result<Value, LxError> {
    let key = args[0].as_str()
        .ok_or_else(|| LxError::type_err("knowledge.get: key must be Str", span))?;
    let id = kb_id(&args[1], span)?;
    let kb = KBS.get(&id)
        .ok_or_else(|| LxError::runtime("knowledge: KB not found", span))?;
    match kb.entries.get(key) {
        Some(e) => Ok(Value::Some(Box::new(entry_to_value(key, e)))),
        None => Ok(Value::None),
    }
}

fn bi_query(args: &[Value], span: Span) -> Result<Value, LxError> {
    let filter_fn = &args[0];
    let id = kb_id(&args[1], span)?;
    let kb = KBS.get(&id)
        .ok_or_else(|| LxError::runtime("knowledge: KB not found", span))?;
    let mut results = Vec::new();
    for (key, entry) in kb.entries.iter() {
        let entry_val = entry_to_value(key, entry);
        let result = call_value(filter_fn, entry_val.clone(), span)?;
        if matches!(result, Value::Bool(true)) {
            results.push(entry_val);
        }
    }
    Ok(Value::List(Arc::new(results)))
}

fn bi_keys(args: &[Value], span: Span) -> Result<Value, LxError> {
    let id = kb_id(&args[0], span)?;
    let kb = KBS.get(&id)
        .ok_or_else(|| LxError::runtime("knowledge: KB not found", span))?;
    let keys: Vec<Value> = kb.entries.keys()
        .map(|k| Value::Str(Arc::from(k.as_str())))
        .collect();
    Ok(Value::List(Arc::new(keys)))
}

fn bi_remove(args: &[Value], span: Span) -> Result<Value, LxError> {
    let key = args[0].as_str()
        .ok_or_else(|| LxError::type_err("knowledge.remove: key must be Str", span))?;
    let id = kb_id(&args[1], span)?;
    let mut kb = KBS.get_mut(&id)
        .ok_or_else(|| LxError::runtime("knowledge: KB not found", span))?;
    kb.entries.shift_remove(key);
    persist(&kb, span)?;
    Ok(make_handle(id))
}

fn bi_merge(args: &[Value], span: Span) -> Result<Value, LxError> {
    let id1 = kb_id(&args[0], span)?;
    let id2 = kb_id(&args[1], span)?;
    let other_entries: IndexMap<String, KBEntry> = {
        let kb2 = KBS.get(&id2)
            .ok_or_else(|| LxError::runtime("knowledge: KB2 not found", span))?;
        kb2.entries.iter().map(|(k, e)| (k.clone(), KBEntry {
            val: e.val.clone(),
            meta: e.meta.clone(),
            stored_at: e.stored_at.clone(),
        })).collect()
    };
    let mut kb1 = KBS.get_mut(&id1)
        .ok_or_else(|| LxError::runtime("knowledge: KB1 not found", span))?;
    for (k, e) in other_entries {
        kb1.entries.insert(k, e);
    }
    persist(&kb1, span)?;
    Ok(make_handle(id1))
}

fn bi_expire(args: &[Value], span: Span) -> Result<Value, LxError> {
    let before = args[0].as_str()
        .ok_or_else(|| LxError::type_err("knowledge.expire: timestamp must be Str", span))?;
    let id = kb_id(&args[1], span)?;
    let mut kb = KBS.get_mut(&id)
        .ok_or_else(|| LxError::runtime("knowledge: KB not found", span))?;
    kb.entries.retain(|_, e| e.stored_at.as_str() >= before);
    persist(&kb, span)?;
    Ok(make_handle(id))
}
