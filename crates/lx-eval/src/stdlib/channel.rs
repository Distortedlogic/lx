use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::mpsc;

use crate::stdlib::helpers::extract_handle_id;
use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::LxVal;
use lx_value::record;
use miette::SourceSpan;

struct ChannelEntry {
  sender: Option<Arc<mpsc::Sender<LxVal>>>,
  receiver: Arc<TokioMutex<mpsc::Receiver<LxVal>>>,
}

static CHANNELS: LazyLock<DashMap<u64, ChannelEntry>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<lx_span::sym::Sym, LxVal> {
  let mut m = IndexMap::new();
  m.insert(lx_span::sym::intern("create"), crate::builtins::mk("channel.create", 1, bi_create));
  m.insert(lx_span::sym::intern("send"), crate::builtins::mk_async("channel.send", 2, bi_send));
  m.insert(lx_span::sym::intern("recv"), crate::builtins::mk_async("channel.recv", 1, bi_recv));
  m.insert(lx_span::sym::intern("try_recv"), crate::builtins::mk("channel.try_recv", 1, bi_try_recv));
  m.insert(lx_span::sym::intern("close"), crate::builtins::mk("channel.close", 1, bi_close));
  m
}

fn chan_id(val: &LxVal, fn_name: &str, span: SourceSpan) -> Result<u64, LxError> {
  extract_handle_id(val, "__chan_id", fn_name, span)
}

fn bi_create(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let cap = args[0].require_usize("channel.create", span)?;
  let capacity = if cap == 0 { 1_000_000 } else { cap };
  let (tx, rx) = mpsc::channel(capacity);
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  CHANNELS.insert(id, ChannelEntry { sender: Some(Arc::new(tx)), receiver: Arc::new(TokioMutex::new(rx)) });
  let sender = record! {
      "__chan_id" => LxVal::int(id),
      "_role" => LxVal::str("sender")
  };
  let receiver = record! {
      "__chan_id" => LxVal::int(id),
      "_role" => LxVal::str("receiver")
  };
  Ok(LxVal::tuple(vec![sender, receiver]))
}

fn bi_send(args: Vec<LxVal>, span: SourceSpan, _ctx: Arc<dyn BuiltinCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> {
  Box::pin(async move {
    let id = chan_id(&args[0], "channel.send", span)?;
    let value = args[1].clone();
    let sender = {
      let entry = CHANNELS.get(&id).ok_or_else(|| LxError::runtime("channel.send: channel not found", span))?;
      match &entry.sender {
        Some(s) => Arc::clone(s),
        None => return Ok(LxVal::err_str("channel closed")),
      }
    };
    match sender.send(value).await {
      Ok(()) => Ok(LxVal::ok_unit()),
      Err(_) => Ok(LxVal::err_str("channel closed")),
    }
  })
}

fn bi_recv(args: Vec<LxVal>, span: SourceSpan, _ctx: Arc<dyn BuiltinCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> {
  Box::pin(async move {
    let id = chan_id(&args[0], "channel.recv", span)?;
    let receiver = {
      let entry = CHANNELS.get(&id).ok_or_else(|| LxError::runtime("channel.recv: channel not found", span))?;
      Arc::clone(&entry.receiver)
    };
    let mut guard = receiver.lock().await;
    match guard.recv().await {
      Some(value) => Ok(LxVal::ok(value)),
      None => Ok(LxVal::err(record! { "kind" => LxVal::str(":closed") })),
    }
  })
}

fn bi_try_recv(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let id = chan_id(&args[0], "channel.try_recv", span)?;
  let receiver = {
    let entry = CHANNELS.get(&id).ok_or_else(|| LxError::runtime("channel.try_recv: channel not found", span))?;
    Arc::clone(&entry.receiver)
  };
  let Ok(mut guard) = receiver.try_lock() else {
    return Ok(LxVal::None);
  };
  match guard.try_recv() {
    Ok(value) => Ok(LxVal::some(value)),
    Err(_) => Ok(LxVal::None),
  }
}

fn bi_close(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let id = chan_id(&args[0], "channel.close", span)?;
  if let Some(mut entry) = CHANNELS.get_mut(&id) {
    entry.sender = None;
  }
  Ok(LxVal::Unit)
}
