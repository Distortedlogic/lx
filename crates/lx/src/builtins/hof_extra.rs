use std::pin::Pin;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::LxVal;

use super::hof::{call, get_list};

type BoxFut = Pin<Box<dyn std::future::Future<Output = Result<LxVal, LxError>>>>;

pub(super) fn bi_take_while(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "take_while", sp)?;
    let mut out = Vec::new();
    for v in items.iter() {
      if call(&args[0], v.clone(), sp, &ctx).await?.as_bool() != Some(true) {
        break;
      }
      out.push(v.clone());
    }
    Ok(LxVal::list(out))
  })
}

pub(super) fn bi_drop_while(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "drop_while", sp)?;
    let mut dropping = true;
    let mut out = Vec::new();
    for v in items.iter() {
      if dropping && call(&args[0], v.clone(), sp, &ctx).await?.as_bool() == Some(true) {
        continue;
      }
      dropping = false;
      out.push(v.clone());
    }
    Ok(LxVal::list(out))
  })
}

pub(super) fn bi_sort_by(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
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

pub(super) fn bi_min_by(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "min_by", sp)?;
    if items.is_empty() {
      return Err(LxError::runtime("min_by: empty list", sp));
    }
    let mut best = items[0].clone();
    let mut best_key = call(&args[0], best.clone(), sp, &ctx).await?;
    for v in &items[1..] {
      let k = call(&args[0], v.clone(), sp, &ctx).await?;
      if super::coll::cmp_values(&k, &best_key).is_lt() {
        best = v.clone();
        best_key = k;
      }
    }
    Ok(best)
  })
}

pub(super) fn bi_max_by(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "max_by", sp)?;
    if items.is_empty() {
      return Err(LxError::runtime("max_by: empty list", sp));
    }
    let mut best = items[0].clone();
    let mut best_key = call(&args[0], best.clone(), sp, &ctx).await?;
    for v in &items[1..] {
      let k = call(&args[0], v.clone(), sp, &ctx).await?;
      if super::coll::cmp_values(&k, &best_key).is_gt() {
        best = v.clone();
        best_key = k;
      }
    }
    Ok(best)
  })
}

pub(super) fn bi_partition(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "partition", sp)?;
    let (mut yes, mut no) = (Vec::new(), Vec::new());
    for v in items.iter() {
      if call(&args[0], v.clone(), sp, &ctx).await?.as_bool() == Some(true) {
        yes.push(v.clone());
      } else {
        no.push(v.clone());
      }
    }
    Ok(LxVal::tuple(vec![LxVal::list(yes), LxVal::list(no)]))
  })
}

pub(super) fn bi_group_by(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "group_by", sp)?;
    let mut groups = IndexMap::new();
    for v in items.iter() {
      let key = call(&args[0], v.clone(), sp, &ctx).await?;
      groups.entry(crate::value::ValueKey(key)).or_insert_with(Vec::new).push(v.clone());
    }
    let map: indexmap::IndexMap<crate::value::ValueKey, LxVal> = groups.into_iter().map(|(k, vs)| (k, LxVal::list(vs))).collect();
    Ok(LxVal::Map(Arc::new(map)))
  })
}

pub(super) fn bi_chunks(args: &[LxVal], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let n = args[0].as_int().ok_or_else(|| LxError::type_err(format!("chunks: size must be Int, got {}", args[0].type_name()), sp))?;
  let items = get_list(&args[1], "chunks", sp)?;
  let n = usize::try_from(n.clone()).map_err(|_| LxError::runtime("chunks: invalid size", sp))?;
  if n == 0 {
    return Err(LxError::runtime("chunks: size must be > 0", sp));
  }
  let out: Vec<LxVal> = items.chunks(n).map(|chunk| LxVal::list(chunk.to_vec())).collect();
  Ok(LxVal::list(out))
}

pub(super) fn bi_windows(args: &[LxVal], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let n = args[0].as_int().ok_or_else(|| LxError::type_err(format!("windows: size must be Int, got {}", args[0].type_name()), sp))?;
  let items = get_list(&args[1], "windows", sp)?;
  let n = usize::try_from(n.clone()).map_err(|_| LxError::runtime("windows: invalid size", sp))?;
  if n == 0 || items.len() < n {
    return Ok(LxVal::list(vec![]));
  }
  let out: Vec<LxVal> = items.windows(n).map(|w| LxVal::list(w.to_vec())).collect();
  Ok(LxVal::list(out))
}

pub(super) fn bi_intersperse(args: &[LxVal], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let items = get_list(&args[1], "intersperse", sp)?;
  let sep = &args[0];
  let mut out = Vec::with_capacity(items.len() * 2);
  for (i, v) in items.iter().enumerate() {
    if i > 0 {
      out.push(sep.clone());
    }
    out.push(v.clone());
  }
  Ok(LxVal::list(out))
}

pub(super) fn bi_scan(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
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

pub(super) fn bi_tap(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let val = args[1].clone();
    call(&args[0], val.clone(), sp, &ctx).await?;
    Ok(val)
  })
}

pub(super) fn bi_find_index(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "find_index", sp)?;
    for (i, v) in items.iter().enumerate() {
      let result = call(&args[0], v.clone(), sp, &ctx).await?;
      if result.as_bool() == Some(true) {
        return Ok(LxVal::Some(Box::new(LxVal::int(i))));
      }
    }
    Ok(LxVal::None)
  })
}

pub(super) fn bi_count(args: Vec<LxVal>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "count", sp)?;
    let mut n = 0usize;
    for v in items.iter() {
      if call(&args[0], v.clone(), sp, &ctx).await?.as_bool() == Some(true) {
        n += 1;
      }
    }
    Ok(LxVal::int(n))
  })
}
