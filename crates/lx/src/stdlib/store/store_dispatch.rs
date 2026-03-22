use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::value::{BuiltinFunc, BuiltinKind, LxVal};
use miette::SourceSpan;

use super::store::{
  NEXT_ID, STORES, StoreState, bi_clear, bi_count, bi_create, bi_entries, bi_get, bi_keys, bi_load, bi_persist, bi_query, bi_remove, bi_set, bi_update,
  get_store, get_store_mut, persist, store_id,
};

pub fn store_method(name: &str, store_val: &LxVal) -> Option<LxVal> {
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
  method.map(|(mname, arity, func)| LxVal::BuiltinFunc(BuiltinFunc { name: mname, arity, kind: BuiltinKind::Sync(func), applied: vec![store_val.clone()] }))
}

pub fn object_insert(fields: indexmap::IndexMap<crate::sym::Sym, crate::value::LxVal>) -> u64 {
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  STORES.insert(id, StoreState { data: fields, path: None });
  id
}

pub fn object_get_field(id: u64, field: &str) -> Option<crate::value::LxVal> {
  STORES.get(&id).and_then(|s| s.data.get(&crate::sym::intern(field)).cloned())
}

pub fn object_update_nested(id: u64, path: &[crate::sym::Sym], value: crate::value::LxVal) -> Result<(), String> {
  let Some(mut s) = STORES.get_mut(&id) else {
    return Err("object not found".into());
  };
  match path {
    [field] => {
      s.data.insert(*field, value);
      Ok(())
    },
    [field, rest @ ..] => {
      let inner = s.data.get(field).ok_or_else(|| format!("field '{field}' not found"))?.clone();
      let updated = update_nested_record(&inner, rest, value)?;
      s.data.insert(*field, updated);
      Ok(())
    },
    [] => Err("empty field path".into()),
  }
}

fn update_nested_record(val: &crate::value::LxVal, path: &[crate::sym::Sym], new_val: crate::value::LxVal) -> Result<crate::value::LxVal, String> {
  let crate::value::LxVal::Record(rec) = val else {
    return Err(format!("field update requires Record, got {}", val.type_name()));
  };
  match path {
    [field] => {
      let mut new_rec = rec.as_ref().clone();
      new_rec.insert(*field, new_val);
      Ok(crate::value::LxVal::record(new_rec))
    },
    [field, rest @ ..] => {
      let inner = rec.get(field).ok_or_else(|| format!("field '{field}' not found"))?;
      let updated = update_nested_record(inner, rest, new_val)?;
      let mut new_rec = rec.as_ref().clone();
      new_rec.insert(*field, updated);
      Ok(crate::value::LxVal::record(new_rec))
    },
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

pub fn build_constructor() -> LxVal {
  mk("Store", 1, bi_create)
}

fn bi_values(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let s = get_store(id, span)?;
  let vals: Vec<LxVal> = s.data.values().cloned().collect();
  Ok(LxVal::list(vals))
}

fn bi_to_record(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let s = get_store(id, span)?;
  Ok(LxVal::record(s.data.clone()))
}

fn bi_has(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let key = args[1].require_str("store.has", span)?;
  let s = get_store(id, span)?;
  Ok(LxVal::Bool(s.data.contains_key(&crate::sym::intern(key))))
}

fn bi_save_to(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let path = args[1].require_str("store.save", span)?;
  let s = get_store(id, span)?;
  let record = LxVal::record(s.data.clone());
  let json_val = serde_json::Value::from(&record);
  let pretty = serde_json::to_string_pretty(&json_val).unwrap_or_default();
  std::fs::write(path, pretty).map_err(|e| LxError::runtime(format!("store.save: {e}"), span))?;
  Ok(LxVal::Unit)
}

fn bi_load_from(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let path = args[1].require_str("store.load", span)?;
  let content = std::fs::read_to_string(path).map_err(|e| LxError::runtime(format!("store.load: {e}"), span))?;
  let json_val: serde_json::Value = serde_json::from_str(&content).map_err(|e| LxError::runtime(format!("store.load: {e}"), span))?;
  let val = LxVal::from(json_val);
  let data = match val {
    LxVal::Record(r) => r.as_ref().clone(),
    _ => return Err(LxError::runtime("store.load: expected JSON object", span)),
  };
  let mut s = get_store_mut(id, span)?;
  s.data = data;
  Ok(LxVal::Unit)
}

fn bi_merge(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let source_data: indexmap::IndexMap<crate::sym::Sym, LxVal> = match &args[1] {
    LxVal::Store { id: src_id } => {
      let src = get_store(*src_id, span)?;
      src.data.clone()
    },
    LxVal::Record(r) => r.as_ref().clone(),
    other => {
      return Err(LxError::type_err(format!("store.merge: expected Store or Record, got {}", other.type_name()), span));
    },
  };
  let mut s = get_store_mut(id, span)?;
  s.data.extend(source_data);
  persist(&s);
  Ok(LxVal::Unit)
}

fn bi_map(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let f = &args[1];
  let s = get_store(id, span)?;
  let snapshot = s.data.clone();
  drop(s);
  let mut results = Vec::new();
  for (k, v) in snapshot {
    let entry = record! {
        "key" => LxVal::str(k.as_str()),
        "value" => v,
    };
    results.push(call_value_sync(f, entry, span, ctx)?);
  }
  Ok(LxVal::list(results))
}
