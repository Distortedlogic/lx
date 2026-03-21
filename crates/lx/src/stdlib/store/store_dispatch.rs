use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::{BuiltinFunc, BuiltinKind, Value};

use super::store::{
    NEXT_ID, STORES, StoreState, bi_clear, bi_count, bi_create, bi_entries, bi_get, bi_keys,
    bi_load, bi_persist, bi_query, bi_remove, bi_set, bi_update, persist, store_id,
};

pub fn store_method(name: &str, store_val: &Value) -> Option<Value> {
    let method: Option<(&'static str, usize, crate::value::SyncBuiltinFn)> = match name {
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
        "merge" => Some(("store.merge", 2, bi_merge)),
        "update" => Some(("store.update", 3, bi_update)),
        "save" => Some(("store.save", 2, bi_save_to)),
        "load" => Some(("store.load", 2, bi_load_from)),
        "persist" => Some(("store.persist", 1, bi_persist)),
        "reload" => Some(("store.reload", 1, bi_load)),
        "to_record" => Some(("store.to_record", 1, bi_to_record)),
        _ => None,
    };
    method.map(|(mname, arity, func)| {
        Value::BuiltinFunc(BuiltinFunc {
            name: mname,
            arity,
            kind: BuiltinKind::Sync(func),
            applied: vec![store_val.clone()],
        })
    })
}

pub fn object_insert(fields: indexmap::IndexMap<String, crate::value::Value>) -> u64 {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    STORES.insert(
        id,
        StoreState {
            data: fields,
            path: None,
        },
    );
    id
}

pub fn object_get_field(id: u64, field: &str) -> Option<crate::value::Value> {
    STORES.get(&id).and_then(|s| s.data.get(field).cloned())
}

pub fn object_update_nested(
    id: u64,
    path: &[String],
    value: crate::value::Value,
) -> Result<(), String> {
    let Some(mut s) = STORES.get_mut(&id) else {
        return Err("object not found".into());
    };
    match path {
        [field] => {
            s.data.insert(field.clone(), value);
            Ok(())
        }
        [field, rest @ ..] => {
            let inner = s
                .data
                .get(field)
                .ok_or_else(|| format!("field '{field}' not found"))?
                .clone();
            let updated = update_nested_record(&inner, rest, value)?;
            s.data.insert(field.clone(), updated);
            Ok(())
        }
        [] => Err("empty field path".into()),
    }
}

fn update_nested_record(
    val: &crate::value::Value,
    path: &[String],
    new_val: crate::value::Value,
) -> Result<crate::value::Value, String> {
    let crate::value::Value::Record(rec) = val else {
        return Err(format!(
            "field update requires Record, got {}",
            val.type_name()
        ));
    };
    match path {
        [field] => {
            let mut new_rec = rec.as_ref().clone();
            new_rec.insert(field.clone(), new_val);
            Ok(crate::value::Value::Record(Arc::new(new_rec)))
        }
        [field, rest @ ..] => {
            let inner = rec
                .get(field)
                .ok_or_else(|| format!("field '{field}' not found"))?;
            let updated = update_nested_record(inner, rest, new_val)?;
            let mut new_rec = rec.as_ref().clone();
            new_rec.insert(field.clone(), updated);
            Ok(crate::value::Value::Record(Arc::new(new_rec)))
        }
        [] => Err("empty field path".into()),
    }
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

fn bi_to_record(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let s = STORES
        .get(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    let fields: indexmap::IndexMap<String, Value> =
        s.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    Ok(Value::Record(Arc::new(fields)))
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
    let json_val = super::json_conv::lx_to_json(&Value::Record(Arc::new(s.data.clone())), span)?;
    let pretty = serde_json::to_string_pretty(&json_val).unwrap_or_default();
    std::fs::write(path, pretty).map_err(|e| LxError::runtime(format!("store.save: {e}"), span))?;
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

fn bi_merge(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = store_id(&args[0], span)?;
    let entries: Vec<(String, Value)> = match &args[1] {
        Value::Store { id: src_id } => {
            let src = STORES
                .get(src_id)
                .ok_or_else(|| LxError::runtime("store.merge: source store not found", span))?;
            src.data
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }
        Value::Record(r) => r.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        other => {
            return Err(LxError::type_err(
                format!(
                    "store.merge: expected Store or Record, got {}",
                    other.type_name()
                ),
                span,
            ));
        }
    };
    let mut s = STORES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("store: not found", span))?;
    for (k, v) in entries {
        s.data.insert(k, v);
    }
    persist(&s);
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
        results.push(call_value_sync(f, entry, span, ctx)?);
    }
    Ok(Value::List(Arc::new(results)))
}
