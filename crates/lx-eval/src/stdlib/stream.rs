use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;

use crate::builtins::call_value_sync;
use crate::std_module;
use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::LxVal;
use miette::SourceSpan;

enum StreamState {
  FromList { items: Arc<Vec<LxVal>>, index: AtomicUsize },
  FromFunc { func: LxVal },
  Map { inner_id: u64, func: LxVal },
  Filter { inner_id: u64, pred: LxVal },
  Take { inner_id: u64, remaining: AtomicUsize },
  Batch { inner_id: u64, size: usize },
}

static STREAMS: LazyLock<DashMap<u64, StreamState>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<lx_span::sym::Sym, LxVal> {
  std_module! {
      "from"    => "stream.from",    1, bi_from;
      "collect" => "stream.collect",  1, bi_collect;
      "map"     => "stream.map",     2, bi_map;
      "filter"  => "stream.filter",  2, bi_filter;
      "take"    => "stream.take",    2, bi_take;
      "batch"   => "stream.batch",   2, bi_batch;
      "each"    => "stream.each",    2, bi_each;
      "fold"    => "stream.fold",    3, bi_fold
  }
}

fn stream_id(v: &LxVal, span: SourceSpan) -> Result<u64, LxError> {
  match v {
    LxVal::Stream { id } => Ok(*id),
    _ => Err(LxError::type_err("stream: expected Stream", span, None)),
  }
}

fn alloc_stream(state: StreamState) -> u64 {
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  STREAMS.insert(id, state);
  id
}

fn pull_next(id: u64, span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let entry = STREAMS.get(&id).ok_or_else(|| LxError::runtime("stream: not found", span))?;
  match entry.value() {
    StreamState::FromList { items, index } => {
      let i = index.fetch_add(1, Ordering::Relaxed);
      if i < items.len() { Ok(LxVal::some(items[i].clone())) } else { Ok(LxVal::None) }
    },
    StreamState::FromFunc { func } => {
      let func = func.clone();
      drop(entry);
      let result = call_value_sync(&func, LxVal::Unit, span, ctx)?;
      match result {
        LxVal::None => Ok(LxVal::None),
        LxVal::Some(_) => Ok(result),
        other => Ok(LxVal::some(other)),
      }
    },
    StreamState::Map { inner_id, func } => {
      let inner_id = *inner_id;
      let func = func.clone();
      drop(entry);
      let upstream = pull_next(inner_id, span, ctx)?;
      match upstream {
        LxVal::Some(val) => {
          let mapped = call_value_sync(&func, *val, span, ctx)?;
          Ok(LxVal::some(mapped))
        },
        _ => Ok(LxVal::None),
      }
    },
    StreamState::Filter { inner_id, pred } => {
      let inner_id = *inner_id;
      let pred = pred.clone();
      drop(entry);
      loop {
        let upstream = pull_next(inner_id, span, ctx)?;
        match upstream {
          LxVal::Some(val) => {
            let keep = call_value_sync(&pred, (*val).clone(), span, ctx)?;
            if matches!(keep, LxVal::Bool(true)) {
              return Ok(LxVal::Some(val));
            }
          },
          _ => return Ok(LxVal::None),
        }
      }
    },
    StreamState::Take { inner_id, remaining } => {
      let r = remaining.load(Ordering::Relaxed);
      if r == 0 {
        return Ok(LxVal::None);
      }
      remaining.fetch_sub(1, Ordering::Relaxed);
      let inner_id = *inner_id;
      drop(entry);
      pull_next(inner_id, span, ctx)
    },
    StreamState::Batch { inner_id, size } => {
      let inner_id = *inner_id;
      let size = *size;
      drop(entry);
      let mut batch = Vec::with_capacity(size);
      for _ in 0..size {
        let upstream = pull_next(inner_id, span, ctx)?;
        match upstream {
          LxVal::Some(val) => batch.push(*val),
          _ => break,
        }
      }
      if batch.is_empty() { Ok(LxVal::None) } else { Ok(LxVal::some(LxVal::list(batch))) }
    },
  }
}

fn bi_from(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let source = &args[0];
  match source {
    LxVal::List(items) => {
      let id = alloc_stream(StreamState::FromList { items: Arc::clone(items), index: AtomicUsize::new(0) });
      Ok(LxVal::Stream { id })
    },
    LxVal::Func(_) | LxVal::BuiltinFunc(_) | LxVal::MultiFunc(_) => {
      let id = alloc_stream(StreamState::FromFunc { func: source.clone() });
      Ok(LxVal::Stream { id })
    },
    _ => Err(LxError::type_err("stream.from: expected List or Func", span, None)),
  }
}

fn bi_collect(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let id = stream_id(&args[0], span)?;
  let mut result = Vec::new();
  while let LxVal::Some(val) = pull_next(id, span, ctx)? {
    result.push(*val);
  }
  Ok(LxVal::list(result))
}

fn bi_map(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let inner_id = stream_id(&args[1], span)?;
  let id = alloc_stream(StreamState::Map { inner_id, func: args[0].clone() });
  Ok(LxVal::Stream { id })
}

fn bi_filter(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let inner_id = stream_id(&args[1], span)?;
  let id = alloc_stream(StreamState::Filter { inner_id, pred: args[0].clone() });
  Ok(LxVal::Stream { id })
}

fn bi_take(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let n = args[0].require_usize("stream.take", span)?;
  let inner_id = stream_id(&args[1], span)?;
  let id = alloc_stream(StreamState::Take { inner_id, remaining: AtomicUsize::new(n) });
  Ok(LxVal::Stream { id })
}

fn bi_batch(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let size = args[0].require_usize("stream.batch", span)?;
  let inner_id = stream_id(&args[1], span)?;
  let id = alloc_stream(StreamState::Batch { inner_id, size });
  Ok(LxVal::Stream { id })
}

fn bi_each(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let f = &args[0];
  let id = stream_id(&args[1], span)?;
  while let LxVal::Some(val) = pull_next(id, span, ctx)? {
    call_value_sync(f, *val, span, ctx)?;
  }
  Ok(LxVal::Unit)
}

fn bi_fold(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let mut acc = args[0].clone();
  let f = &args[1];
  let id = stream_id(&args[2], span)?;
  while let LxVal::Some(val) = pull_next(id, span, ctx)? {
    let partial = call_value_sync(f, acc, span, ctx)?;
    acc = call_value_sync(&partial, *val, span, ctx)?;
  }
  Ok(acc)
}
