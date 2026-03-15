use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{Value, ValueKey};

use super::mk;

pub(crate) fn cmp_values(a: &Value, b: &Value) -> std::cmp::Ordering {
  match (a, b) {
    (Value::Int(x), Value::Int(y)) => x.cmp(y),
    (Value::Float(x), Value::Float(y)) => x.total_cmp(y),
    (Value::Int(x), Value::Float(y)) => x.to_f64().map_or(std::cmp::Ordering::Greater, |xf| xf.total_cmp(y)),
    (Value::Float(x), Value::Int(y)) => y.to_f64().map_or(std::cmp::Ordering::Less, |yf| x.total_cmp(&yf)),
    (Value::Str(x), Value::Str(y)) => x.cmp(y),
    (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
    _ => std::cmp::Ordering::Equal,
  }
}

fn maybe(v: Option<&Value>) -> Value {
  v.map_or(Value::None, |v| Value::Some(Box::new(v.clone())))
}

fn bi_first(args: &[Value], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  Ok(maybe(args[0].as_list().ok_or_else(|| LxError::type_err("first expects List", sp))?.first()))
}
fn bi_last(args: &[Value], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  Ok(maybe(args[0].as_list().ok_or_else(|| LxError::type_err("last expects List", sp))?.last()))
}
fn bi_contains(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  match &args[1] {
    Value::Str(s) => {
      let needle = args[0].as_str().ok_or_else(|| LxError::type_err("contains?: needle must be Str for Str haystack", span))?;
      Ok(Value::Bool(s.contains(needle)))
    },
    Value::List(l) => Ok(Value::Bool(l.iter().any(|v| v == &args[0]))),
    other => Err(LxError::type_err(format!("contains? expects Str/List, got {}", other.type_name()), span)),
  }
}
fn bi_get(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  match &args[1] {
    Value::List(l) => {
      let n = args[0]
        .as_int()
        .ok_or_else(|| LxError::type_err("get: index must be Int for List", span))?;
      let idx = n.to_i64().ok_or_else(|| LxError::runtime("get: index out of range", span))?;
      let idx = if idx < 0 { l.len() as i64 + idx } else { idx };
      if idx < 0 { return Ok(Value::None); }
      Ok(maybe(l.get(idx as usize)))
    },
    Value::Record(r) => {
      let key = args[0].as_str().ok_or_else(|| LxError::type_err("get: key must be Str for Record", span))?;
      Ok(maybe(r.get(key)))
    },
    Value::Map(m) => Ok(maybe(m.get(&ValueKey(args[0].clone())))),
    other => Err(LxError::type_err(format!("get expects List/Record/Map, got {}", other.type_name()), span)),
  }
}
fn bi_sort(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let l = args[0].as_list().ok_or_else(|| LxError::type_err("sort expects List", span))?;
  let mut items = l.as_ref().clone();
  items.sort_by(cmp_values);
  Ok(Value::List(Arc::new(items)))
}
fn bi_sorted_q(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let l = args[0].as_list().ok_or_else(|| LxError::type_err("sorted? expects List", span))?;
  Ok(Value::Bool(l.windows(2).all(|w| cmp_values(&w[0], &w[1]).is_le())))
}
fn bi_rev(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let l = args[0].as_list().ok_or_else(|| LxError::type_err("rev expects List", span))?;
  let mut items = l.as_ref().clone();
  items.reverse();
  Ok(Value::List(Arc::new(items)))
}
fn num_fold(
  name: &str,
  list: &[Value],
  init_int: BigInt,
  init_float: f64,
  op_int: fn(&BigInt, &BigInt) -> BigInt,
  op_float: fn(f64, f64) -> f64,
  span: Span,
) -> Result<Value, LxError> {
  let mut has_float = false;
  let (mut ia, mut fa) = (init_int, init_float);
  for v in list {
    match v {
      Value::Int(n) if has_float => {
        fa = op_float(fa, n.to_f64().ok_or_else(|| LxError::runtime(format!("{name}: int too large"), span))?);
      },
      Value::Int(n) => ia = op_int(&ia, n),
      Value::Float(f) => {
        if !has_float {
          has_float = true;
          fa = ia.to_f64().ok_or_else(|| LxError::runtime(format!("{name}: int too large"), span))?;
        }
        fa = op_float(fa, *f);
      },
      other => return Err(LxError::type_err(format!("{name}: non-number {}", other.type_name()), span)),
    }
  }
  if has_float { Ok(Value::Float(fa)) } else { Ok(Value::Int(ia)) }
}
fn bi_sum(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let l = args[0].as_list().ok_or_else(|| LxError::type_err("sum expects List", span))?;
  num_fold("sum", l, BigInt::from(0), 0.0, |a, b| a + b, |a, b| a + b, span)
}
fn bi_product(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let l = args[0].as_list().ok_or_else(|| LxError::type_err("product expects List", span))?;
  num_fold("product", l, BigInt::from(1), 1.0, |a, b| a * b, |a, b| a * b, span)
}
fn bi_min(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let l = args[0].as_list().ok_or_else(|| LxError::type_err("min expects List", span))?;
  l.iter().min_by(|a, b| cmp_values(a, b)).cloned().ok_or_else(|| LxError::runtime("min: empty list", span))
}
fn bi_max(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let l = args[0].as_list().ok_or_else(|| LxError::type_err("max expects List", span))?;
  l.iter().max_by(|a, b| cmp_values(a, b)).cloned().ok_or_else(|| LxError::runtime("max: empty list", span))
}
fn bi_uniq(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let l = args[0].as_list().ok_or_else(|| LxError::type_err("uniq expects List", span))?;
  let mut out: Vec<Value> = Vec::with_capacity(l.len());
  for v in l.iter() {
    if out.last() != Some(v) {
      out.push(v.clone());
    }
  }
  Ok(Value::List(Arc::new(out)))
}
fn bi_flatten(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let l = args[0].as_list().ok_or_else(|| LxError::type_err("flatten expects List", span))?;
  let mut out = Vec::new();
  for v in l.iter() {
    match v {
      Value::List(i) => out.extend(i.iter().cloned()),
      o => out.push(o.clone()),
    }
  }
  Ok(Value::List(Arc::new(out)))
}
fn kv_tuple(k: Value, v: Value) -> Value {
  Value::Tuple(Arc::new(vec![k, v]))
}
fn bi_to_list(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  match &args[0] {
    Value::Map(m) => Ok(Value::List(Arc::new(m.iter().map(|(k, v)| kv_tuple(k.0.clone(), v.clone())).collect()))),
    other => Err(LxError::type_err(format!("to_list expects Map, got {}", other.type_name()), span)),
  }
}
fn bi_to_map(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  match &args[0] {
    Value::Record(r) => Ok(Value::Map(Arc::new(r.iter().map(|(k, v)| (ValueKey(Value::Str(Arc::from(k.as_str()))), v.clone())).collect()))),
    Value::List(l) => {
      let mut m = IndexMap::new();
      for v in l.iter() {
        match v {
          Value::Tuple(t) if t.len() == 2 => {
            m.insert(ValueKey(t[0].clone()), t[1].clone());
          },
          other => return Err(LxError::type_err(format!("to_map: element must be 2-tuple, got {}", other.type_name()), span)),
        }
      }
      Ok(Value::Map(Arc::new(m)))
    },
    other => Err(LxError::type_err(format!("to_map expects Record/List, got {}", other.type_name()), span)),
  }
}
fn bi_to_record(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let m = match &args[0] {
    Value::Map(m) => m,
    other => return Err(LxError::type_err(format!("to_record expects Map, got {}", other.type_name()), span)),
  };
  let mut r = IndexMap::new();
  for (k, v) in m.iter() {
    let key = k.0.as_str().ok_or_else(|| LxError::type_err("to_record: map key must be Str", span))?;
    r.insert(key.to_string(), v.clone());
  }
  Ok(Value::Record(Arc::new(r)))
}
fn bi_keys(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  match &args[0] {
    Value::Map(m) => Ok(Value::List(Arc::new(m.keys().map(|k| k.0.clone()).collect()))),
    Value::Record(r) => Ok(Value::List(Arc::new(r.keys().map(|k| Value::Str(Arc::from(k.as_str()))).collect()))),
    other => Err(LxError::type_err(format!("keys expects Map/Record, got {}", other.type_name()), span)),
  }
}
fn bi_values(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  match &args[0] {
    Value::Map(m) => Ok(Value::List(Arc::new(m.values().cloned().collect()))),
    Value::Record(r) => Ok(Value::List(Arc::new(r.values().cloned().collect()))),
    other => Err(LxError::type_err(format!("values expects Map/Record, got {}", other.type_name()), span)),
  }
}
fn bi_entries(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  match &args[0] {
    Value::Map(m) => Ok(Value::List(Arc::new(m.iter().map(|(k, v)| kv_tuple(k.0.clone(), v.clone())).collect()))),
    Value::Record(r) => Ok(Value::List(Arc::new(r.iter().map(|(k, v)| kv_tuple(Value::Str(Arc::from(k.as_str())), v.clone())).collect()))),
    other => Err(LxError::type_err(format!("entries expects Map/Record, got {}", other.type_name()), span)),
  }
}
fn bi_has_key(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  match &args[1] {
    Value::Map(m) => Ok(Value::Bool(m.contains_key(&ValueKey(args[0].clone())))),
    Value::Record(r) => {
      let key = args[0].as_str().ok_or_else(|| LxError::type_err("has_key?: key must be Str for Record", span))?;
      Ok(Value::Bool(r.contains_key(key)))
    },
    other => Err(LxError::type_err(format!("has_key? expects Map/Record, got {}", other.type_name()), span)),
  }
}
fn bi_remove(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  let m = match &args[1] {
    Value::Map(m) => m,
    other => return Err(LxError::type_err(format!("remove expects Map, got {}", other.type_name()), span)),
  };
  let mut out = m.as_ref().clone();
  out.shift_remove(&ValueKey(args[0].clone()));
  Ok(Value::Map(Arc::new(out)))
}
fn bi_merge(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
  match (&args[0], &args[1]) {
    (Value::Map(m1), Value::Map(m2)) => {
      let mut merged = m1.as_ref().clone();
      for (k, v) in m2.iter() {
        merged.insert(k.clone(), v.clone());
      }
      Ok(Value::Map(Arc::new(merged)))
    },
    _ => Err(LxError::type_err("merge expects two Maps", span)),
  }
}
pub(super) fn register(env: &mut Env) {
  env.bind("first".into(), mk("first", 1, bi_first));
  env.bind("last".into(), mk("last", 1, bi_last));
  env.bind("contains?".into(), mk("contains?", 2, bi_contains));
  env.bind("get".into(), mk("get", 2, bi_get));
  env.bind("sort".into(), mk("sort", 1, bi_sort));
  env.bind("sorted?".into(), mk("sorted?", 1, bi_sorted_q));
  env.bind("rev".into(), mk("rev", 1, bi_rev));
  env.bind("sum".into(), mk("sum", 1, bi_sum));
  env.bind("product".into(), mk("product", 1, bi_product));
  env.bind("min".into(), mk("min", 1, bi_min));
  env.bind("max".into(), mk("max", 1, bi_max));
  env.bind("uniq".into(), mk("uniq", 1, bi_uniq));
  env.bind("flatten".into(), mk("flatten", 1, bi_flatten));
  env.bind("to_list".into(), mk("to_list", 1, bi_to_list));
  env.bind("to_map".into(), mk("to_map", 1, bi_to_map));
  env.bind("to_record".into(), mk("to_record", 1, bi_to_record));
  env.bind("keys".into(), mk("keys", 1, bi_keys));
  env.bind("values".into(), mk("values", 1, bi_values));
  env.bind("entries".into(), mk("entries", 1, bi_entries));
  env.bind("has_key?".into(), mk("has_key?", 2, bi_has_key));
  env.bind("remove".into(), mk("remove", 2, bi_remove));
  env.bind("merge".into(), mk("merge", 2, bi_merge));
}
