use std::sync::Arc;

use crate::BuiltinCtx;
use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

use super::BoxFut;
use super::hof::{call, get_list};

async fn parallel_map_batch(items: &[LxVal], func: &LxVal, sp: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<Vec<LxVal>, LxError> {
  let mut futures = Vec::with_capacity(items.len());
  for v in items.iter() {
    let f = func.clone();
    let v = v.clone();
    let ctx = Arc::clone(ctx);
    futures.push(async move { call(&f, v, sp, &ctx).await });
  }
  let results = futures::future::join_all(futures).await;
  let out: Vec<_> = results.into_iter().collect::<Result<_, _>>()?;
  Ok(out)
}

pub(super) fn bi_pmap(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[1], "pmap", sp)?;
    let out = parallel_map_batch(&items, &args[0], sp, &ctx).await?;
    Ok(LxVal::list(out))
  })
}

pub(super) fn bi_pmap_n(args: Vec<LxVal>, sp: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> BoxFut {
  Box::pin(async move {
    let items = get_list(&args[2], "pmap_n", sp)?;
    let n = args[0].as_int().and_then(|i| usize::try_from(i.clone()).ok()).ok_or_else(|| LxError::runtime("pmap_n: first arg must be a positive Int", sp))?;
    if n == 0 {
      return Err(LxError::runtime("pmap_n: concurrency limit must be > 0", sp));
    }
    let mut out = Vec::with_capacity(items.len());
    for chunk in items.chunks(n) {
      let batch = parallel_map_batch(chunk, &args[1], sp, &ctx).await?;
      out.extend(batch);
    }
    Ok(LxVal::list(out))
  })
}
