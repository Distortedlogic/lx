use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

use super::BoxFut;

pub(super) fn register(env: &crate::env::Env) {
  super::register_builtins!(env, {
    "take"/2 => bi_take, "drop"/2 => bi_drop, "zip"/2 => bi_zip,
    "enumerate"/1 => bi_enumerate,
    "chunks"/2 => super::hof_extra::bi_chunks,
    "windows"/2 => super::hof_extra::bi_windows,
    "intersperse"/2 => super::hof_extra::bi_intersperse,
  });
  super::register_builtins!(env, async {
    "map"/2 => bi_map, "filter"/2 => bi_filter, "fold"/3 => bi_fold,
    "flat_map"/2 => bi_flat_map, "each"/2 => bi_each,
    "find"/2 => bi_find, "any?"/2 => bi_any, "all?"/2 => bi_all,
    "none?"/2 => bi_none_q,
    "count"/2 => super::hof_extra::bi_count,
    "take_while"/2 => super::hof_extra::bi_take_while,
    "drop_while"/2 => super::hof_extra::bi_drop_while,
    "sort_by"/2 => super::hof_extra::bi_sort_by,
    "min_by"/2 => super::hof_extra::bi_min_by,
    "max_by"/2 => super::hof_extra::bi_max_by,
    "partition"/2 => super::hof_extra::bi_partition,
    "group_by"/2 => super::hof_extra::bi_group_by,
    "scan"/3 => super::hof_extra::bi_scan,
    "tap"/2 => super::hof_extra::bi_tap,
    "find_index"/2 => super::hof_extra::bi_find_index,
    "pmap"/2 => super::hof_parallel::bi_pmap,
    "pmap_n"/3 => super::hof_parallel::bi_pmap_n,
  });
}

pub(super) async fn call(f: &LxVal, arg: LxVal, span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  crate::builtins::call_value(f, arg, span, ctx).await
}

pub(super) async fn call_predicate(f: &LxVal, arg: LxVal, sp: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<bool, LxError> {
  Ok(call(f, arg, sp, ctx).await?.as_bool() == Some(true))
}

fn range_to_list(start: i64, end: i64, inclusive: bool) -> Vec<LxVal> {
  if inclusive { (start..=end).map(LxVal::int).collect() } else { (start..end).map(LxVal::int).collect() }
}

pub(super) enum ListRef<'a> {
  Borrowed(&'a [LxVal]),
  Owned(Vec<LxVal>),
}

impl<'a> std::ops::Deref for ListRef<'a> {
  type Target = [LxVal];
  fn deref(&self) -> &[LxVal] {
    match self {
      ListRef::Borrowed(s) => s,
      ListRef::Owned(v) => v.as_slice(),
    }
  }
}

pub(super) fn get_list<'a>(v: &'a LxVal, name: &str, sp: SourceSpan) -> Result<ListRef<'a>, LxError> {
  match v {
    LxVal::List(l) => Ok(ListRef::Borrowed(l.as_ref())),
    LxVal::Range { start, end, inclusive } => Ok(ListRef::Owned(range_to_list(*start, *end, *inclusive))),
    other => Err(LxError::type_err(format!("{name} expects List or Range, got {}", other.type_name()), sp, None)),
  }
}

fn bi_map(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "map", sp)?;
    let mut out = Vec::with_capacity(items.len());
    for v in items.iter() {
      out.push(call(&args[0], v.clone(), sp, &ctx).await?);
    }
    Ok(LxVal::list(out))
  })
}

fn bi_filter(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "filter", sp)?;
    let mut out = Vec::new();
    for v in items.iter() {
      let result = call(&args[0], v.clone(), sp, &ctx).await?;
      match result.as_bool() {
        Some(true) => out.push(v.clone()),
        Some(false) => {},
        _ => {
          return Err(LxError::type_err(format!("filter predicate must return Bool, got {}", result.type_name()), sp, None));
        },
      }
    }
    Ok(LxVal::list(out))
  })
}

fn bi_fold(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[2], "fold", sp)?;
    let mut acc = args[0].clone();
    let f = &args[1];
    for v in items.iter() {
      let partial = call(f, acc, sp, &ctx).await?;
      acc = call(&partial, v.clone(), sp, &ctx).await?;
    }
    Ok(acc)
  })
}

fn bi_flat_map(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "flat_map", sp)?;
    let mut out = Vec::new();
    for v in items.iter() {
      let result = call(&args[0], v.clone(), sp, &ctx).await?;
      match result {
        LxVal::List(l) => out.extend(l.as_ref().iter().cloned()),
        other => out.push(other),
      }
    }
    Ok(LxVal::list(out))
  })
}

fn bi_each(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "each", sp)?;
    for v in items.iter() {
      call(&args[0], v.clone(), sp, &ctx).await?;
    }
    Ok(LxVal::Unit)
  })
}

fn bi_take(args: &[LxVal], sp: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let n = args[0].require_int("take", sp)?;
  let n = n.to_usize().ok_or_else(|| LxError::runtime("take: count out of range", sp))?;
  let items = get_list(&args[1], "take", sp)?;
  Ok(LxVal::list(items.iter().take(n).cloned().collect()))
}

fn bi_drop(args: &[LxVal], sp: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let n = args[0].require_int("drop", sp)?;
  let n = n.to_usize().ok_or_else(|| LxError::runtime("drop: count out of range", sp))?;
  let items = get_list(&args[1], "drop", sp)?;
  Ok(LxVal::list(items.iter().skip(n).cloned().collect()))
}

fn bi_zip(args: &[LxVal], sp: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let a = get_list(&args[0], "zip", sp)?;
  let b = get_list(&args[1], "zip", sp)?;
  let out: Vec<LxVal> = a.iter().zip(b.iter()).map(|(x, y)| LxVal::tuple(vec![y.clone(), x.clone()])).collect();
  Ok(LxVal::list(out))
}

fn bi_enumerate(args: &[LxVal], sp: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let items = get_list(&args[0], "enumerate", sp)?;
  let out: Vec<LxVal> = items.iter().enumerate().map(|(i, v)| LxVal::tuple(vec![LxVal::int(i), v.clone()])).collect();
  Ok(LxVal::list(out))
}

fn bi_find(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "find", sp)?;
    for v in items.iter() {
      if call_predicate(&args[0], v.clone(), sp, &ctx).await? {
        return Ok(v.clone());
      }
    }
    Ok(LxVal::None)
  })
}

fn bi_any(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "any?", sp)?;
    for v in items.iter() {
      if call_predicate(&args[0], v.clone(), sp, &ctx).await? {
        return Ok(LxVal::Bool(true));
      }
    }
    Ok(LxVal::Bool(false))
  })
}

fn bi_all(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "all?", sp)?;
    for v in items.iter() {
      if !call_predicate(&args[0], v.clone(), sp, &ctx).await? {
        return Ok(LxVal::Bool(false));
      }
    }
    Ok(LxVal::Bool(true))
  })
}

fn bi_none_q(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "none?", sp)?;
    for v in items.iter() {
      if call_predicate(&args[0], v.clone(), sp, &ctx).await? {
        return Ok(LxVal::Bool(false));
      }
    }
    Ok(LxVal::Bool(true))
  })
}
