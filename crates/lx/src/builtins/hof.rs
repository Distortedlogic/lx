use std::sync::Arc;

use crate::env::Env;
use crate::error::LxError;
use crate::iterator::{self, IterSource, LiveIter};
use crate::span::Span;
use crate::value::Value;

use super::mk;

pub(super) fn register(env: &mut Env) {
  env.bind("map".into(), mk("map", 2, bi_map));
  env.bind("filter".into(), mk("filter", 2, bi_filter));
  env.bind("fold".into(), mk("fold", 3, bi_fold));
  env.bind("flat_map".into(), mk("flat_map", 2, bi_flat_map));
  env.bind("each".into(), mk("each", 2, bi_each));
  env.bind("take".into(), mk("take", 2, bi_take));
  env.bind("drop".into(), mk("drop", 2, bi_drop));
  env.bind("zip".into(), mk("zip", 2, bi_zip));
  env.bind("enumerate".into(), mk("enumerate", 1, bi_enumerate));
  env.bind("find".into(), mk("find", 2, bi_find));
  env.bind("any?".into(), mk("any?", 2, bi_any));
  env.bind("all?".into(), mk("all?", 2, bi_all));
  env.bind("none?".into(), mk("none?", 2, bi_none_q));
  env.bind("count".into(), mk("count", 2, bi_count));
  env.bind("take_while".into(), mk("take_while", 2, bi_take_while));
  env.bind("drop_while".into(), mk("drop_while", 2, bi_drop_while));
  env.bind("sort_by".into(), mk("sort_by", 2, bi_sort_by));
  env.bind("min_by".into(), mk("min_by", 2, bi_min_by));
  env.bind("max_by".into(), mk("max_by", 2, bi_max_by));
  env.bind("partition".into(), mk("partition", 2, bi_partition));
  env.bind("group_by".into(), mk("group_by", 2, bi_group_by));
  env.bind("chunks".into(), mk("chunks", 2, bi_chunks));
  env.bind("windows".into(), mk("windows", 2, bi_windows));
  env.bind("intersperse".into(), mk("intersperse", 2, bi_intersperse));
  env.bind("scan".into(), mk("scan", 3, bi_scan));
  env.bind("tap".into(), mk("tap", 2, bi_tap));
  env.bind("find_index".into(), mk("find_index", 2, bi_find_index));
  env.bind("pmap".into(), mk("pmap", 2, bi_pmap));
  env.bind("pmap_n".into(), mk("pmap_n", 3, bi_pmap_n));
}

fn call(f: &Value, arg: Value, span: Span) -> Result<Value, LxError> {
  crate::builtins::call_value(f, arg, span)
}

fn range_to_list(start: i64, end: i64, inclusive: bool) -> Vec<Value> {
  if inclusive {
    (start..=end).map(|i| Value::Int(i.into())).collect()
  } else {
    (start..end).map(|i| Value::Int(i.into())).collect()
  }
}

enum ListRef<'a> {
  Borrowed(&'a [Value]),
  Owned(Vec<Value>),
}

impl<'a> std::ops::Deref for ListRef<'a> {
  type Target = [Value];
  fn deref(&self) -> &[Value] {
    match self {
      ListRef::Borrowed(s) => s,
      ListRef::Owned(v) => v.as_slice(),
    }
  }
}

fn get_list<'a>(v: &'a Value, name: &str, sp: Span) -> Result<ListRef<'a>, LxError> {
  match v {
    Value::List(l) => Ok(ListRef::Borrowed(l.as_ref())),
    Value::Range { start, end, inclusive } => Ok(ListRef::Owned(range_to_list(*start, *end, *inclusive))),
    _ => Err(LxError::type_err(format!("{name} expects List or Range"), sp)),
  }
}

fn as_iter(v: &Value) -> Option<LiveIter> {
  match v {
    Value::Iterator(source) => Some(iterator::instantiate(source)),
    Value::Record(fields) => fields.get("next").map(|f| iterator::from_record_next(f.clone())),
    _ => None,
  }
}

fn bi_map(args: &[Value], sp: Span) -> Result<Value, LxError> {
  if let Some(live) = as_iter(&args[1]) {
    let mapped = iterator::make_live(iterator::MappedIter::new(live, args[0].clone()));
    return Ok(Value::Iterator(IterSource::Live(mapped)));
  }
  let items = get_list(&args[1], "map", sp)?;
  let mut out = Vec::with_capacity(items.len());
  for v in items.iter() {
    out.push(call(&args[0], v.clone(), sp)?);
  }
  Ok(Value::List(Arc::new(out)))
}

fn bi_filter(args: &[Value], sp: Span) -> Result<Value, LxError> {
  if let Some(live) = as_iter(&args[1]) {
    let filtered = iterator::make_live(iterator::FilteredIter::new(live, args[0].clone()));
    return Ok(Value::Iterator(IterSource::Live(filtered)));
  }
  let items = get_list(&args[1], "filter", sp)?;
  let mut out = Vec::new();
  for v in items.iter() {
    let result = call(&args[0], v.clone(), sp)?;
    match result.as_bool() {
      Some(true) => out.push(v.clone()),
      Some(false) => {},
      _ => return Err(LxError::type_err("filter predicate must return Bool", sp)),
    }
  }
  Ok(Value::List(Arc::new(out)))
}

fn bi_fold(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[2], "fold", sp)?;
  let mut acc = args[0].clone();
  let f = &args[1];
  for v in items.iter() {
    let partial = call(f, acc, sp)?;
    acc = call(&partial, v.clone(), sp)?;
  }
  Ok(acc)
}

fn bi_flat_map(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "flat_map", sp)?;
  let mut out = Vec::new();
  for v in items.iter() {
    let result = call(&args[0], v.clone(), sp)?;
    match result {
      Value::List(l) => out.extend(l.as_ref().iter().cloned()),
      other => out.push(other),
    }
  }
  Ok(Value::List(Arc::new(out)))
}

fn bi_each(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "each", sp)?;
  for v in items.iter() {
    call(&args[0], v.clone(), sp)?;
  }
  Ok(Value::Unit)
}

fn bi_take(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let n = args[0].as_int().ok_or_else(|| LxError::type_err("take: first arg must be Int", sp))?;
  let n = usize::try_from(n.clone()).unwrap_or(0);
  if let Some(live) = as_iter(&args[1]) {
    let mut items = Vec::with_capacity(n);
    for _ in 0..n {
      match iterator::pull_next(&live, sp)? {
        Some(v) => items.push(v),
        None => break,
      }
    }
    return Ok(Value::List(Arc::new(items)));
  }
  let items = get_list(&args[1], "take", sp)?;
  Ok(Value::List(Arc::new(items.iter().take(n).cloned().collect())))
}

fn bi_drop(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let n = args[0].as_int().ok_or_else(|| LxError::type_err("drop: first arg must be Int", sp))?;
  let n = usize::try_from(n.clone()).unwrap_or(0);
  if let Some(live) = as_iter(&args[1]) {
    for _ in 0..n {
      if iterator::pull_next(&live, sp)?.is_none() {
        break;
      }
    }
    return Ok(Value::Iterator(IterSource::Live(live)));
  }
  let items = get_list(&args[1], "drop", sp)?;
  Ok(Value::List(Arc::new(items.iter().skip(n).cloned().collect())))
}

fn bi_zip(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let a = get_list(&args[0], "zip", sp)?;
  let b = get_list(&args[1], "zip", sp)?;
  let out: Vec<Value> = a.iter().zip(b.iter()).map(|(x, y)| Value::Tuple(Arc::new(vec![y.clone(), x.clone()]))).collect();
  Ok(Value::List(Arc::new(out)))
}

fn bi_enumerate(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[0], "enumerate", sp)?;
  let out: Vec<Value> =
    items.iter().enumerate().map(|(i, v)| Value::Tuple(Arc::new(vec![Value::Int(i.into()), v.clone()]))).collect();
  Ok(Value::List(Arc::new(out)))
}

fn bi_find(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "find", sp)?;
  for v in items.iter() {
    let result = call(&args[0], v.clone(), sp)?;
    if result.as_bool() == Some(true) {
      return Ok(Value::Some(Box::new(v.clone())));
    }
  }
  Ok(Value::None)
}

fn bi_any(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "any?", sp)?;
  for v in items.iter() {
    if call(&args[0], v.clone(), sp)?.as_bool() == Some(true) {
      return Ok(Value::Bool(true));
    }
  }
  Ok(Value::Bool(false))
}

fn bi_all(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "all?", sp)?;
  for v in items.iter() {
    if call(&args[0], v.clone(), sp)?.as_bool() != Some(true) {
      return Ok(Value::Bool(false));
    }
  }
  Ok(Value::Bool(true))
}

fn bi_none_q(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "none?", sp)?;
  for v in items.iter() {
    if call(&args[0], v.clone(), sp)?.as_bool() == Some(true) {
      return Ok(Value::Bool(false));
    }
  }
  Ok(Value::Bool(true))
}

fn bi_count(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "count", sp)?;
  let mut n = 0usize;
  for v in items.iter() {
    if call(&args[0], v.clone(), sp)?.as_bool() == Some(true) {
      n += 1;
    }
  }
  Ok(Value::Int(n.into()))
}

fn bi_take_while(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "take_while", sp)?;
  let mut out = Vec::new();
  for v in items.iter() {
    if call(&args[0], v.clone(), sp)?.as_bool() != Some(true) {
      break;
    }
    out.push(v.clone());
  }
  Ok(Value::List(Arc::new(out)))
}

fn bi_drop_while(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "drop_while", sp)?;
  let mut dropping = true;
  let mut out = Vec::new();
  for v in items.iter() {
    if dropping && call(&args[0], v.clone(), sp)?.as_bool() == Some(true) {
      continue;
    }
    dropping = false;
    out.push(v.clone());
  }
  Ok(Value::List(Arc::new(out)))
}

fn bi_sort_by(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "sort_by", sp)?;
  let mut keyed: Vec<(Value, Value)> = Vec::with_capacity(items.len());
  for v in items.iter() {
    let k = call(&args[0], v.clone(), sp)?;
    keyed.push((k, v.clone()));
  }
  keyed.sort_by(|(a, _), (b, _)| super::coll::cmp_values(a, b));
  Ok(Value::List(Arc::new(keyed.into_iter().map(|(_, v)| v).collect())))
}

fn bi_min_by(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "min_by", sp)?;
  if items.is_empty() {
    return Err(LxError::runtime("min_by: empty list", sp));
  }
  let mut best = &items[0];
  let mut best_key = call(&args[0], best.clone(), sp)?;
  for v in &items[1..] {
    let k = call(&args[0], v.clone(), sp)?;
    if super::coll::cmp_values(&k, &best_key).is_lt() {
      best = v;
      best_key = k;
    }
  }
  Ok(best.clone())
}

fn bi_max_by(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "max_by", sp)?;
  if items.is_empty() {
    return Err(LxError::runtime("max_by: empty list", sp));
  }
  let mut best = &items[0];
  let mut best_key = call(&args[0], best.clone(), sp)?;
  for v in &items[1..] {
    let k = call(&args[0], v.clone(), sp)?;
    if super::coll::cmp_values(&k, &best_key).is_gt() {
      best = v;
      best_key = k;
    }
  }
  Ok(best.clone())
}

fn bi_partition(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "partition", sp)?;
  let (mut yes, mut no) = (Vec::new(), Vec::new());
  for v in items.iter() {
    if call(&args[0], v.clone(), sp)?.as_bool() == Some(true) {
      yes.push(v.clone());
    } else {
      no.push(v.clone());
    }
  }
  Ok(Value::Tuple(Arc::new(vec![Value::List(Arc::new(yes)), Value::List(Arc::new(no))])))
}

fn bi_group_by(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "group_by", sp)?;
  let mut groups = indexmap::IndexMap::new();
  for v in items.iter() {
    let key = call(&args[0], v.clone(), sp)?;
    groups.entry(crate::value::ValueKey(key)).or_insert_with(Vec::new).push(v.clone());
  }
  let map: indexmap::IndexMap<crate::value::ValueKey, Value> =
    groups.into_iter().map(|(k, vs)| (k, Value::List(Arc::new(vs)))).collect();
  Ok(Value::Map(Arc::new(map)))
}

fn bi_chunks(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let n = args[0].as_int().ok_or_else(|| LxError::type_err("chunks: size must be Int", sp))?;
  let items = get_list(&args[1], "chunks", sp)?;
  let n = usize::try_from(n.clone()).map_err(|_| LxError::runtime("chunks: invalid size", sp))?;
  if n == 0 {
    return Err(LxError::runtime("chunks: size must be > 0", sp));
  }
  let out: Vec<Value> = items.chunks(n).map(|chunk| Value::List(Arc::new(chunk.to_vec()))).collect();
  Ok(Value::List(Arc::new(out)))
}

fn bi_windows(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let n = args[0].as_int().ok_or_else(|| LxError::type_err("windows: size must be Int", sp))?;
  let items = get_list(&args[1], "windows", sp)?;
  let n = usize::try_from(n.clone()).map_err(|_| LxError::runtime("windows: invalid size", sp))?;
  if n == 0 || items.len() < n {
    return Ok(Value::List(Arc::new(vec![])));
  }
  let out: Vec<Value> = items.windows(n).map(|w| Value::List(Arc::new(w.to_vec()))).collect();
  Ok(Value::List(Arc::new(out)))
}

fn bi_intersperse(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "intersperse", sp)?;
  let sep = &args[0];
  let mut out = Vec::with_capacity(items.len() * 2);
  for (i, v) in items.iter().enumerate() {
    if i > 0 {
      out.push(sep.clone());
    }
    out.push(v.clone());
  }
  Ok(Value::List(Arc::new(out)))
}

fn bi_scan(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[2], "scan", sp)?;
  let mut acc = args[0].clone();
  let f = &args[1];
  let mut out = Vec::with_capacity(items.len() + 1);
  out.push(acc.clone());
  for v in items.iter() {
    let partial = call(f, acc, sp)?;
    acc = call(&partial, v.clone(), sp)?;
    out.push(acc.clone());
  }
  Ok(Value::List(Arc::new(out)))
}

fn bi_tap(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let val = &args[1];
  call(&args[0], val.clone(), sp)?;
  Ok(val.clone())
}

fn bi_pmap(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "pmap", sp)?;
  let mut out = Vec::with_capacity(items.len());
  for v in items.iter() {
    out.push(call(&args[0], v.clone(), sp)?);
  }
  Ok(Value::List(Arc::new(out)))
}

fn bi_pmap_n(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[2], "pmap_n", sp)?;
  let mut out = Vec::with_capacity(items.len());
  for v in items.iter() {
    out.push(call(&args[1], v.clone(), sp)?);
  }
  Ok(Value::List(Arc::new(out)))
}

fn bi_find_index(args: &[Value], sp: Span) -> Result<Value, LxError> {
  let items = get_list(&args[1], "find_index", sp)?;
  for (i, v) in items.iter().enumerate() {
    let result = call(&args[0], v.clone(), sp)?;
    if result.as_bool() == Some(true) {
      return Ok(Value::Some(Box::new(Value::Int(i.into()))));
    }
  }
  Ok(Value::None)
}
