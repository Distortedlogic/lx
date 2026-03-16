use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

use super::{MemEntry, MemoryStore, STORES, store_id};

pub(super) fn entry_to_value(id: &str, e: &MemEntry) -> Value {
    let tags: Vec<Value> = e
        .tags
        .iter()
        .map(|t| Value::Str(Arc::from(t.as_str())))
        .collect();
    crate::record! {
        "id" => Value::Str(Arc::from(id)),
        "content" => Value::Str(Arc::from(e.content.as_str())),
        "tier" => Value::Int(num_bigint::BigInt::from(e.tier)),
        "confidence" => Value::Float(e.confidence),
        "tags" => Value::List(Arc::new(tags)),
        "created_at" => Value::Str(Arc::from(e.created_at.as_str())),
        "confirmed" => Value::Int(num_bigint::BigInt::from(e.confirmed)),
        "contradicted" => Value::Int(num_bigint::BigInt::from(e.contradicted)),
    }
}

pub(super) fn persist(store: &MemoryStore, span: Span) -> Result<(), LxError> {
    let items: Vec<Value> = store
        .entries
        .iter()
        .map(|(id, e)| entry_to_value(id, e))
        .collect();
    let list = Value::List(Arc::new(items));
    let json = json_conv::lx_to_json(&list, span)?;
    let s = serde_json::to_string_pretty(&json)
        .map_err(|e| LxError::runtime(format!("memory: serialize: {e}"), span))?;
    std::fs::write(&store.path, s)
        .map_err(|e| LxError::runtime(format!("memory: write: {e}"), span))
}

pub(super) fn load_entries(
    path: &str,
    span: Span,
) -> Result<(IndexMap<String, MemEntry>, u64), LxError> {
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
        let id = r
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();
        if let Ok(n) = id.parse::<u64>()
            && n >= max_id
        {
            max_id = n + 1;
        }
        let tags = r
            .get("tags")
            .and_then(|v| v.as_list())
            .map(|l| {
                l.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        entries.insert(
            id,
            MemEntry {
                content: r
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                tier: r
                    .get("tier")
                    .and_then(|v| v.as_int())
                    .and_then(|n| n.try_into().ok())
                    .unwrap_or(0),
                confidence: r
                    .get("confidence")
                    .and_then(|v| v.as_float())
                    .unwrap_or(0.0),
                tags,
                created_at: r
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                confirmed: r
                    .get("confirmed")
                    .and_then(|v| v.as_int())
                    .and_then(|n| n.try_into().ok())
                    .unwrap_or(0),
                contradicted: r
                    .get("contradicted")
                    .and_then(|v| v.as_int())
                    .and_then(|n| n.try_into().ok())
                    .unwrap_or(0),
            },
        );
    }
    Ok((entries, max_id))
}

pub(super) fn bi_tier(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let level = args[0]
        .as_int()
        .and_then(|n| -> Option<i64> { n.try_into().ok() })
        .ok_or_else(|| LxError::type_err("memory.tier: level must be Int", span))?;
    let sid = store_id(&args[1], span)?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let results: Vec<Value> = store
        .entries
        .iter()
        .filter(|(_, e)| e.tier == level)
        .map(|(id, e)| entry_to_value(id, e))
        .collect();
    Ok(Value::List(Arc::new(results)))
}

pub(super) fn bi_all(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("memory: store not found", span))?;
    let results: Vec<Value> = store
        .entries
        .iter()
        .map(|(id, e)| entry_to_value(id, e))
        .collect();
    Ok(Value::List(Arc::new(results)))
}
