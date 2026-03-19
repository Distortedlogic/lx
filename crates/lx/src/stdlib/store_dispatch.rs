use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::{BuiltinFunc, Value};

use super::store::{
    NEXT_ID, STORES, StoreState, bi_clear, bi_count, bi_create, bi_entries, bi_get, bi_keys,
    bi_load, bi_persist, bi_query, bi_remove, bi_set, bi_update, store_id,
};

pub fn store_method(name: &str, store_val: &Value) -> Option<Value> {
    let method: Option<(&'static str, usize, crate::value::BuiltinFn)> = match name {
        "set" => Some(("store.set", 3, bi_set)),
        "get" => Some(("store.get", 2, bi_get)),
        "keys" => Some(("store.keys", 1, bi_keys)),
        "values" => Some(("store.values", 1, bi_values)),
        "entries" => Some(("store.entries", 1, bi_entries)),
        "remove" => Some(("store.remove", 2, bi_remove)),
        "len" | "count" => Some(("store.len", 1, bi_count)),
        "has" => Some(("store.has", 2, bi_has)),
        "clear" => Some(("store.clear", 1, bi_clear)),
        "filter" | "query" => Some(("store.query", 2, bi_query)),
        "map" => Some(("store.map", 2, bi_map)),
        "update" => Some(("store.update", 3, bi_update)),
        "save" => Some(("store.save", 2, bi_save_to)),
        "load" => Some(("store.load", 2, bi_load_from)),
        "persist" => Some(("store.persist", 1, bi_persist)),
        "reload" => Some(("store.reload", 1, bi_load)),
        _ => None,
    };
    method.map(|(mname, arity, func)| {
        Value::BuiltinFunc(BuiltinFunc {
            name: mname,
            arity,
            func,
            applied: vec![store_val.clone()],
        })
    })
}

pub fn store_len(id: u64) -> usize {
    STORES.get(&id).map(|s| s.data.len()).unwrap_or(0)
}

pub fn store_clone(id: u64) -> u64 {
    let data = STORES.get(&id).map(|s| s.data.clone()).unwrap_or_default();
    let new_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    STORES.insert(new_id, StoreState { data, path: None });
    new_id
}

pub fn build_constructor() -> Value {
    mk("Store", 1, bi_create)
}

fn bi_values(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    let vals: Vec<Value> = s.data.values().cloned().collect();
    Ok(Value::List(Arc::new(vals)))
}

fn bi_has(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("store.has: key must be Str", span))?;
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    Ok(Value::Bool(s.data.contains_key(key)))
}

fn bi_save_to(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let path = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("store.save: path must be Str", span))?;
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    let json_val =
        super::json_conv::lx_to_json(&Value::Record(Arc::new(s.data.clone())), span)?;
    let pretty = serde_json::to_string_pretty(&json_val).unwrap_or_default();
    std::fs::write(path, pretty)
        .map_err(|e| LxError::runtime(format!("store.save: {e}"), span))?;
    Ok(Value::Unit)
}

fn bi_load_from(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let path = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("store.load: path must be Str", span))?;
    let content = std::fs::read_to_string(path)
        .map_err(|e| LxError::runtime(format!("store.load: {e}"), span))?;
    let json_val: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| LxError::runtime(format!("store.load: {e}"), span))?;
    let val = super::json_conv::json_to_lx(json_val);
    let data = match val {
        Value::Record(r) => r.as_ref().clone(),
        _ => return Err(LxError::runtime("store.load: expected JSON object", span)),
    };
    let mut s = STORES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    s.data = data;
    Ok(Value::Unit)
}

fn bi_map(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let f = &args[1];
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    let snapshot: Vec<_> = s.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    drop(s);
    let mut results = Vec::new();
    for (k, v) in snapshot {
        let entry = record! {
            "key" => Value::Str(Arc::from(k.as_str())),
            "value" => v,
        };
        results.push(call_value(f, entry, span, ctx)?);
    }
    Ok(Value::List(Arc::new(results)))
}
