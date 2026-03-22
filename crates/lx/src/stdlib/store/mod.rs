use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;

use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

pub(super) struct StoreState {
  pub(super) data: IndexMap<String, LxVal>,
  pub(super) path: Option<PathBuf>,
}

pub(super) static STORES: LazyLock<DashMap<u64, StoreState>> = LazyLock::new(DashMap::new);
pub(super) static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, LxVal> {
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

pub(super) fn store_id(v: &LxVal, span: SourceSpan) -> Result<u64, LxError> {
  match v {
    LxVal::Store { id } => Ok(*id),
    _ => Err(LxError::type_err("store: expected Store", span)),
  }
}

pub(super) fn get_store(id: u64, span: SourceSpan) -> Result<dashmap::mapref::one::Ref<'static, u64, StoreState>, LxError> {
  STORES.get(&id).ok_or_else(|| LxError::runtime("store: not found", span))
}

pub(super) fn get_store_mut(id: u64, span: SourceSpan) -> Result<dashmap::mapref::one::RefMut<'static, u64, StoreState>, LxError> {
  STORES.get_mut(&id).ok_or_else(|| LxError::runtime("store: not found", span))
}

pub(super) fn persist(state: &StoreState) {
  let Some(ref path) = state.path else { return };
  let record = LxVal::record(state.data.clone());
  let json_val = serde_json::Value::from(&record);
  let pretty = serde_json::to_string_pretty(&json_val).unwrap_or_default();
  let _ = std::fs::write(path, pretty);
}

fn load_from_disk(path: &std::path::Path) -> IndexMap<String, LxVal> {
  let Ok(content) = std::fs::read_to_string(path) else {
    return IndexMap::new();
  };
  let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) else {
    return IndexMap::new();
  };
  let val = LxVal::from(json_val);
  match val {
    LxVal::Record(r) => r.as_ref().clone(),
    _ => IndexMap::new(),
  }
}

pub(super) fn bi_create(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path = match &args[0] {
    LxVal::Record(r) => r.get("persist").and_then(|v| v.as_str()).map(PathBuf::from),
    LxVal::Unit => None,
    _ => {
      return Err(LxError::type_err("store.create: opts must be Record or ()", span));
    },
  };
  let data = path.as_deref().map(load_from_disk).unwrap_or_default();
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  STORES.insert(id, StoreState { data, path });
  Ok(LxVal::Store { id })
}

pub(super) fn bi_set(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let key = args[1].require_str("store.set", span)?;
  let mut s = get_store_mut(id, span)?;
  s.data.insert(key.to_string(), args[2].clone());
  persist(&s);
  Ok(LxVal::Unit)
}

pub(super) fn bi_get(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let key = args[1].require_str("store.get", span)?;
  let s = get_store(id, span)?;
  Ok(s.data.get(key).cloned().unwrap_or(LxVal::None))
}

pub(super) fn bi_update(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let key = args[1].require_str("store.update", span)?;
  let f = &args[2];
  let mut s = get_store_mut(id, span)?;
  let old = s.data.get(key).cloned().unwrap_or(LxVal::None);
  let new_val = call_value_sync(f, old, span, ctx)?;
  if matches!(new_val, LxVal::Err(_)) {
    return Ok(new_val);
  }
  s.data.insert(key.to_string(), new_val.clone());
  persist(&s);
  Ok(new_val)
}

pub(super) fn bi_remove(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let key = args[1].require_str("store.remove", span)?;
  let mut s = get_store_mut(id, span)?;
  let removed = s.data.shift_remove(key).unwrap_or(LxVal::None);
  persist(&s);
  Ok(removed)
}

pub(super) fn bi_keys(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let s = get_store(id, span)?;
  let keys: Vec<LxVal> = s.data.keys().map(LxVal::str).collect();
  Ok(LxVal::list(keys))
}

pub(super) fn bi_entries(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let s = get_store(id, span)?;
  let entries: Vec<LxVal> = s
    .data
    .iter()
    .map(|(k, v)| {
      record! {
          "key" => LxVal::str(k),
          "value" => v.clone(),
      }
    })
    .collect();
  Ok(LxVal::list(entries))
}

pub(super) fn bi_query(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let pred = &args[1];
  let s = get_store(id, span)?;
  let snapshot = s.data.clone();
  drop(s);
  let mut matched = Vec::new();
  for (k, v) in snapshot {
    let entry = record! {
        "key" => LxVal::str(k),
        "value" => v,
    };
    let result = call_value_sync(pred, entry.clone(), span, ctx)?;
    if matches!(result, LxVal::Bool(true)) {
      matched.push(entry);
    }
  }
  Ok(LxVal::list(matched))
}

pub(super) fn bi_count(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let s = get_store(id, span)?;
  Ok(LxVal::int(s.data.len()))
}

pub(super) fn bi_clear(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let mut s = get_store_mut(id, span)?;
  s.data.clear();
  persist(&s);
  Ok(LxVal::Unit)
}

pub(super) fn bi_persist(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let s = get_store(id, span)?;
  persist(&s);
  Ok(LxVal::Unit)
}

pub(super) fn bi_load(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = store_id(&args[0], span)?;
  let mut s = get_store_mut(id, span)?;
  if let Some(ref path) = s.path {
    s.data = load_from_disk(path);
  }
  Ok(LxVal::Unit)
}
