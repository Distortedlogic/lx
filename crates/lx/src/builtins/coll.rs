use std::sync::Arc;

use indexmap::IndexMap;
use num_traits::ToPrimitive;

use crate::env::Env;
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::{LxVal, ValueKey};

use super::mk;

pub(crate) fn cmp_values(a: &LxVal, b: &LxVal) -> std::cmp::Ordering {
  match (a, b) {
    (LxVal::Int(x), LxVal::Int(y)) => x.cmp(y),
    (LxVal::Float(x), LxVal::Float(y)) => x.total_cmp(y),
    (LxVal::Int(x), LxVal::Float(y)) => x.to_f64().map_or(std::cmp::Ordering::Greater, |xf| xf.total_cmp(y)),
    (LxVal::Float(x), LxVal::Int(y)) => y.to_f64().map_or(std::cmp::Ordering::Less, |yf| x.total_cmp(&yf)),
    (LxVal::Str(x), LxVal::Str(y)) => x.cmp(y),
    (LxVal::Bool(x), LxVal::Bool(y)) => x.cmp(y),
    _ => std::cmp::Ordering::Equal,
  }
}

fn maybe(v: Option<&LxVal>) -> LxVal {
  v.map_or(LxVal::None, |v| LxVal::Some(Box::new(v.clone())))
}

fn bi_first(args: &[LxVal], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(maybe(args[0].require_list("first", sp)?.first()))
}

fn bi_last(args: &[LxVal], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(maybe(args[0].require_list("last", sp)?.last()))
}

fn bi_contains(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[1] {
    LxVal::Str(s) => {
      let needle = args[0].require_str("contains?", span)?;
      Ok(LxVal::Bool(s.contains(needle)))
    },
    LxVal::List(l) => Ok(LxVal::Bool(l.iter().any(|v| v == &args[0]))),
    other => Err(LxError::type_err(format!("contains? expects Str/List, got {}", other.type_name()), span)),
  }
}

fn bi_get(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[1] {
    LxVal::List(l) => {
      let n = args[0].require_int("get", span)?;
      let idx = n.to_i64().ok_or_else(|| LxError::runtime("get: index out of range", span))?;
      let idx = if idx < 0 { l.len() as i64 + idx } else { idx };
      if idx < 0 {
        return Ok(LxVal::None);
      }
      Ok(maybe(l.get(idx as usize)))
    },
    LxVal::Record(r) => {
      let key = args[0].require_str("get", span)?;
      Ok(maybe(r.get(key)))
    },
    LxVal::Map(m) => Ok(maybe(m.get(&ValueKey(args[0].clone())))),
    other => Err(LxError::type_err(format!("get expects List/Record/Map, got {}", other.type_name()), span)),
  }
}

fn kv_tuple(k: LxVal, v: LxVal) -> LxVal {
  LxVal::tuple(vec![k, v])
}

fn bi_to_list(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Map(m) => Ok(LxVal::list(m.iter().map(|(k, v)| kv_tuple(k.0.clone(), v.clone())).collect())),
    other => Err(LxError::type_err(format!("to_list expects Map, got {}", other.type_name()), span)),
  }
}

fn bi_to_map(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Record(r) => Ok(LxVal::Map(Arc::new(r.iter().map(|(k, v)| (ValueKey(LxVal::str(k)), v.clone())).collect()))),
    LxVal::List(l) => {
      let mut m = IndexMap::new();
      for v in l.iter() {
        match v {
          LxVal::Tuple(t) if t.len() == 2 => {
            m.insert(ValueKey(t[0].clone()), t[1].clone());
          },
          other => {
            return Err(LxError::type_err(format!("to_map: element must be 2-tuple, got {}", other.type_name()), span));
          },
        }
      }
      Ok(LxVal::Map(Arc::new(m)))
    },
    other => Err(LxError::type_err(format!("to_map expects Record/List, got {}", other.type_name()), span)),
  }
}

fn bi_to_record(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let m = match &args[0] {
    LxVal::Map(m) => m,
    other => {
      return Err(LxError::type_err(format!("to_record expects Map, got {}", other.type_name()), span));
    },
  };
  let mut r = IndexMap::new();
  for (k, v) in m.iter() {
    let key = k.0.require_str("to_record", span)?;
    r.insert(key.to_string(), v.clone());
  }
  Ok(LxVal::record(r))
}

fn bi_keys(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Map(m) => Ok(LxVal::list(m.keys().map(|k| k.0.clone()).collect())),
    LxVal::Record(r) => Ok(LxVal::list(r.keys().map(LxVal::str).collect())),
    other => Err(LxError::type_err(format!("keys expects Map/Record, got {}", other.type_name()), span)),
  }
}

fn bi_values(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Map(m) => Ok(LxVal::list(m.values().cloned().collect())),
    LxVal::Record(r) => Ok(LxVal::list(r.values().cloned().collect())),
    other => Err(LxError::type_err(format!("values expects Map/Record, got {}", other.type_name()), span)),
  }
}

fn bi_entries(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Map(m) => Ok(LxVal::list(m.iter().map(|(k, v)| kv_tuple(k.0.clone(), v.clone())).collect())),
    LxVal::Record(r) => Ok(LxVal::list(r.iter().map(|(k, v)| kv_tuple(LxVal::str(k), v.clone())).collect())),
    other => Err(LxError::type_err(format!("entries expects Map/Record, got {}", other.type_name()), span)),
  }
}

pub(super) fn register(env: &mut Env) {
  env.bind("first".into(), mk("first", 1, bi_first));
  env.bind("last".into(), mk("last", 1, bi_last));
  env.bind("contains?".into(), mk("contains?", 2, bi_contains));
  env.bind("get".into(), mk("get", 2, bi_get));
  env.bind("to_list".into(), mk("to_list", 1, bi_to_list));
  env.bind("to_map".into(), mk("to_map", 1, bi_to_map));
  env.bind("to_record".into(), mk("to_record", 1, bi_to_record));
  env.bind("keys".into(), mk("keys", 1, bi_keys));
  env.bind("values".into(), mk("values", 1, bi_values));
  env.bind("entries".into(), mk("entries", 1, bi_entries));
  super::coll_transform::register(env);
}
