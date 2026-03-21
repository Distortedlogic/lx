use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub(super) struct StoreState {
    pub(super) data: IndexMap<String, Value>,
    pub(super) path: Option<PathBuf>,
}

pub(super) static STORES: LazyLock<DashMap<u64, StoreState>> = LazyLock::new(DashMap::new);
pub(super) static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("store.create", 1, bi_create));
    m.insert("set".into(), mk("store.set", 3, bi_set));
    m.insert("get".into(), mk("store.get", 2, bi_get));
    m.insert("update".into(), mk("store.update", 3, bi_update));
    m.insert("remove".into(), mk("store.remove", 2, bi_remove));
    m.insert("keys".into(), mk("store.keys", 1, bi_keys));
    m.insert("entries".into(), mk("store.entries", 1, bi_entries));
    m.insert("query".into(), mk("store.query", 2, bi_query));
    m.insert("count".into(), mk("store.count", 1, bi_count));
    m.insert("clear".into(), mk("store.clear", 1, bi_clear));
    m.insert("persist".into(), mk("store.persist", 1, bi_persist));
    m.insert("load".into(), mk("store.load", 1, bi_load));
    m
}

pub(super) fn store_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Store { id } => Ok(*id),
        _ => Err(LxError::type_err("store: expected Store", span)),
    }
}

pub(super) fn persist(state: &StoreState) {
    let Some(ref path) = state.path else { return };
    let dummy_span = Span::default();
    let Ok(json_val) =
        super::json_conv::lx_to_json(&Value::Record(Arc::new(state.data.clone())), dummy_span)
    else {
        return;
    };
    let pretty = serde_json::to_string_pretty(&json_val).unwrap_or_default();
    let _ = std::fs::write(path, pretty);
}

fn load_from_disk(path: &std::path::Path) -> IndexMap<String, Value> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return IndexMap::new();
    };
    let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return IndexMap::new();
    };
    let val = super::json_conv::json_to_lx(json_val);
    match val {
        Value::Record(r) => r.as_ref().clone(),
        _ => IndexMap::new(),
    }
}

pub(super) fn bi_create(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let path = match &args[0] {
        Value::Record(r) => r.get("persist").and_then(|v| v.as_str()).map(PathBuf::from),
        Value::Unit => None,
        _ => {
            return Err(LxError::type_err(
                "store.create: opts must be Record or ()",
                span,
            ));
        }
    };
    let data = path.as_deref().map(load_from_disk).unwrap_or_default();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    STORES.insert(id, StoreState { data, path });
    Ok(Value::Store { id })
}

pub(super) fn bi_set(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("store.set: key must be Str", span))?;
    let mut s = STORES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    s.data.insert(key.to_string(), args[2].clone());
    persist(&s);
    Ok(Value::Unit)
}

pub(super) fn bi_get(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("store.get: key must be Str", span))?;
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    Ok(s.data.get(key).cloned().unwrap_or(Value::None))
}

pub(super) fn bi_update(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("store.update: key must be Str", span))?;
    let f = &args[2];
    let mut s = STORES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    let old = s.data.get(key).cloned().unwrap_or(Value::None);
    let new_val = call_value_sync(f, old, span, ctx)?;
    if let Value::Err(_) = &new_val {
        return Ok(new_val);
    }
    s.data.insert(key.to_string(), new_val.clone());
    persist(&s);
    Ok(new_val)
}

pub(super) fn bi_remove(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("store.remove: key must be Str", span))?;
    let mut s = STORES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    let removed = s.data.shift_remove(key).unwrap_or(Value::None);
    persist(&s);
    Ok(removed)
}

pub(super) fn bi_keys(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    let keys: Vec<Value> = s
        .data
        .keys()
        .map(|k| Value::Str(Arc::from(k.as_str())))
        .collect();
    Ok(Value::List(Arc::new(keys)))
}

pub(super) fn bi_entries(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    let entries: Vec<Value> = s
        .data
        .iter()
        .map(|(k, v)| {
            record! {
                "key" => Value::Str(Arc::from(k.as_str())),
                "value" => v.clone(),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(entries)))
}

pub(super) fn bi_query(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let pred = &args[1];
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    let snapshot: Vec<_> = s.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    drop(s);
    let mut matched = Vec::new();
    for (k, v) in snapshot {
        let entry = record! {
            "key" => Value::Str(Arc::from(k.as_str())),
            "value" => v,
        };
        let result = call_value_sync(pred, entry.clone(), span, ctx)?;
        if matches!(result, Value::Bool(true)) {
            matched.push(entry);
        }
    }
    Ok(Value::List(Arc::new(matched)))
}

pub(super) fn bi_count(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    Ok(Value::Int(BigInt::from(s.data.len())))
}

pub(super) fn bi_clear(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let mut s = STORES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    s.data.clear();
    persist(&s);
    Ok(Value::Unit)
}

pub(super) fn bi_persist(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    persist(&s);
    Ok(Value::Unit)
}

pub(super) fn bi_load(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let mut s = STORES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    if let Some(ref path) = s.path {
        s.data = load_from_disk(path);
    }
    Ok(Value::Unit)
}
