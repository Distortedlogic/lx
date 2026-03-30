use std::cmp::Ordering;
use std::sync::Arc;

use indexmap::IndexMap;
use itertools::Itertools;

use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::{LxVal, ValueKey};
use miette::SourceSpan;

use super::BoxFut;
use super::hof::{call, call_predicate, get_list};

pub(super) fn bi_take_while(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "take_while", sp)?;
    let mut out = Vec::new();
    for v in items.iter() {
      if !call_predicate(&args[0], v.clone(), sp, &ctx).await? {
        break;
      }
      out.push(v.clone());
    }
    Ok(LxVal::list(out))
  })
}

pub(super) fn bi_drop_while(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "drop_while", sp)?;
    let mut dropping = true;
    let mut out = Vec::new();
    for v in items.iter() {
      if dropping && call_predicate(&args[0], v.clone(), sp, &ctx).await? {
        continue;
      }
      dropping = false;
      out.push(v.clone());
    }
    Ok(LxVal::list(out))
  })
}

pub(super) fn bi_sort_by(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "sort_by", sp)?;
    let mut keyed: Vec<(LxVal, LxVal)> = Vec::with_capacity(items.len());
    for v in items.iter() {
      let k = call(&args[0], v.clone(), sp, &ctx).await?;
      keyed.push((k, v.clone()));
    }
    keyed.sort_by(|(a, _), (b, _)| super::coll::cmp_values(a, b));
    Ok(LxVal::list(keyed.into_iter().map(|(_, v)| v).collect()))
  })
}

async fn extremum_by(
  items: &[LxVal],
  f: &LxVal,
  pick_first: fn(Ordering) -> bool,
  name: &str,
  sp: SourceSpan,
  ctx: &Arc<dyn BuiltinCtx>,
) -> Result<LxVal, LxError> {
  if items.is_empty() {
    return Err(LxError::runtime(format!("{name}: empty list"), sp));
  }
  let mut best = items[0].clone();
  let mut best_key = call(f, best.clone(), sp, ctx).await?;
  for v in &items[1..] {
    let k = call(f, v.clone(), sp, ctx).await?;
    if pick_first(super::coll::cmp_values(&k, &best_key)) {
      best = v.clone();
      best_key = k;
    }
  }
  Ok(best)
}

pub(super) fn bi_min_by(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "min_by", sp)?;
    extremum_by(&items, &args[0], Ordering::is_lt, "min_by", sp, &ctx).await
  })
}

pub(super) fn bi_max_by(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "max_by", sp)?;
    extremum_by(&items, &args[0], Ordering::is_gt, "max_by", sp, &ctx).await
  })
}

pub(super) fn bi_partition(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "partition", sp)?;
    let (mut yes, mut no) = (Vec::new(), Vec::new());
    for v in items.iter() {
      if call_predicate(&args[0], v.clone(), sp, &ctx).await? {
        yes.push(v.clone());
      } else {
        no.push(v.clone());
      }
    }
    Ok(LxVal::tuple(vec![LxVal::list(yes), LxVal::list(no)]))
  })
}

pub(super) fn bi_group_by(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "group_by", sp)?;
    let mut groups = IndexMap::new();
    for v in items.iter() {
      let key = call(&args[0], v.clone(), sp, &ctx).await?;
      groups.entry(ValueKey(key)).or_insert_with(Vec::new).push(v.clone());
    }
    let map: indexmap::IndexMap<ValueKey, LxVal> = groups.into_iter().map(|(k, vs)| (k, LxVal::list(vs))).collect();
    Ok(LxVal::Map(Arc::new(map)))
  })
}

pub(super) fn bi_chunks(args: &[LxVal], sp: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let n = args[0].require_int("chunks", sp)?;
  let items = get_list(&args[1], "chunks", sp)?;
  let n = usize::try_from(n.clone()).map_err(|_| LxError::runtime("chunks: invalid size", sp))?;
  if n == 0 {
    return Err(LxError::runtime("chunks: size must be > 0", sp));
  }
  let out: Vec<LxVal> = items.chunks(n).map(|chunk| LxVal::list(chunk.to_vec())).collect();
  Ok(LxVal::list(out))
}

pub(super) fn bi_windows(args: &[LxVal], sp: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let n = args[0].require_int("windows", sp)?;
  let items = get_list(&args[1], "windows", sp)?;
  let n = usize::try_from(n.clone()).map_err(|_| LxError::runtime("windows: invalid size", sp))?;
  if n == 0 || items.len() < n {
    return Ok(LxVal::list(vec![]));
  }
  let out: Vec<LxVal> = items.windows(n).map(|w| LxVal::list(w.to_vec())).collect();
  Ok(LxVal::list(out))
}

pub(super) fn bi_intersperse(args: &[LxVal], sp: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let items = get_list(&args[1], "intersperse", sp)?;
  let out: Vec<LxVal> = Itertools::intersperse(items.iter().cloned(), args[0].clone()).collect();
  Ok(LxVal::list(out))
}

pub(super) fn bi_scan(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[2], "scan", sp)?;
    let mut acc = args[0].clone();
    let f = &args[1];
    let mut out = Vec::with_capacity(items.len() + 1);
    out.push(acc.clone());
    for v in items.iter() {
      let partial = call(f, acc, sp, &ctx).await?;
      acc = call(&partial, v.clone(), sp, &ctx).await?;
      out.push(acc.clone());
    }
    Ok(LxVal::list(out))
  })
}

pub(super) fn bi_tap(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let val = args[1].clone();
    call(&args[0], val.clone(), sp, &ctx).await?;
    Ok(val)
  })
}

pub(super) fn bi_find_index(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "find_index", sp)?;
    for (i, v) in items.iter().enumerate() {
      if call_predicate(&args[0], v.clone(), sp, &ctx).await? {
        return Ok(LxVal::some(LxVal::int(i)));
      }
    }
    Ok(LxVal::None)
  })
}

pub(super) fn bi_count(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "count", sp)?;
    let mut n = 0usize;
    for v in items.iter() {
      if call_predicate(&args[0], v.clone(), sp, &ctx).await? {
        n += 1;
      }
    }
    Ok(LxVal::int(n))
  })
}
