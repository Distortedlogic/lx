use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use indexmap::IndexMap;
use tokio::sync::Mutex as TokioMutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, connect_async};

use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::LxVal;

type WsSink = Arc<TokioMutex<futures::stream::SplitSink<tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>>;

type WsStream = Arc<TokioMutex<futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>>;

struct WsConn {
  sink: WsSink,
  stream: WsStream,
}

static WS_CONNS: LazyLock<DashMap<u64, WsConn>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn conn_id(v: &LxVal, span: Span) -> Result<u64, LxError> {
  let LxVal::Record(fields) = v else {
    return Err(LxError::type_err("ws: expected connection handle Record", span));
  };
  match fields.get("__ws_id") {
    Some(LxVal::Int(n)) => u64::try_from(n).map_err(|_| LxError::type_err("ws: invalid connection id", span)),
    _ => Err(LxError::type_err("ws: missing __ws_id in handle", span)),
  }
}

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("connect".into(), mk("ws.connect", 1, bi_connect));
  m.insert("send".into(), mk("ws.send", 2, bi_send));
  m.insert("recv".into(), mk("ws.recv", 1, bi_recv));
  m.insert("recv_json".into(), mk("ws.recv_json", 1, bi_recv_json));
  m.insert("close".into(), mk("ws.close", 1, bi_close));
  m
}

fn bi_connect(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let url = args[0].require_str("ws.connect", span)?.to_string();

  tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
      match connect_async(&url).await {
        Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(format!("ws.connect: {e}"))))),
        Ok((ws_stream, _response)) => {
          let (sink, stream) = ws_stream.split();
          let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
          WS_CONNS.insert(id, WsConn { sink: Arc::new(TokioMutex::new(sink)), stream: Arc::new(TokioMutex::new(stream)) });
          Ok(LxVal::Ok(Box::new(record! {
              "__ws_id" => LxVal::int(id),
              "url" => LxVal::str(url)
          })))
        },
      }
    })
  })
}

fn bi_close(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = conn_id(&args[0], span)?;
  match WS_CONNS.remove(&id) {
    Some((_, conn)) => {
      let val = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
          match conn.sink.lock().await.close().await {
            Ok(()) => LxVal::Ok(Box::new(LxVal::Unit)),
            Err(e) => LxVal::Err(Box::new(LxVal::str(format!("ws.close: {e}")))),
          }
        })
      });
      Ok(val)
    },
    None => Ok(LxVal::Err(Box::new(LxVal::str("ws.close: connection not found")))),
  }
}

fn bi_send(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = conn_id(&args[0], span)?;
  let msg = args[1].require_str("ws.send", span)?.to_string();

  let sink = match WS_CONNS.get(&id) {
    Some(conn) => conn.sink.clone(),
    None => {
      return Ok(LxVal::Err(Box::new(LxVal::str("ws.send: connection not found"))));
    },
  };

  tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
      match sink.lock().await.send(Message::Text(msg)).await {
        Ok(()) => Ok(LxVal::Ok(Box::new(LxVal::Unit))),
        Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(format!("ws.send: {e}"))))),
      }
    })
  })
}

fn bi_recv(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = conn_id(&args[0], span)?;
  let (sink, stream) = match WS_CONNS.get(&id) {
    Some(conn) => (conn.sink.clone(), conn.stream.clone()),
    None => {
      return Ok(LxVal::Err(Box::new(LxVal::str("ws.recv: connection not found"))));
    },
  };

  tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
      loop {
        match stream.lock().await.next().await {
          None => {
            return Ok(LxVal::Err(Box::new(LxVal::str("connection closed"))));
          },
          Some(Err(e)) => {
            return Ok(LxVal::Err(Box::new(LxVal::str(format!("ws.recv: {e}")))));
          },
          Some(Ok(Message::Text(t))) => {
            return Ok(LxVal::Ok(Box::new(LxVal::str(&t))));
          },
          Some(Ok(Message::Binary(b))) => {
            let hex: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
            return Ok(LxVal::Ok(Box::new(LxVal::str(hex))));
          },
          Some(Ok(Message::Close(_))) => {
            WS_CONNS.remove(&id);
            return Ok(LxVal::Err(Box::new(LxVal::str("connection closed"))));
          },
          Some(Ok(Message::Ping(payload))) => {
            let _ = sink.lock().await.send(Message::Pong(payload)).await;
          },
          Some(Ok(_)) => {},
        }
      }
    })
  })
}

fn bi_recv_json(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let recv_result = bi_recv(args, span, _ctx)?;
  match recv_result {
    LxVal::Ok(inner) => {
      if let LxVal::Str(s) = *inner {
        match serde_json::from_str::<serde_json::Value>(&s) {
          Ok(json_val) => Ok(LxVal::Ok(Box::new(LxVal::from(json_val)))),
          Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(format!("ws.recv_json: parse error: {e}"))))),
        }
      } else {
        Ok(LxVal::Err(Box::new(LxVal::str("ws.recv_json: expected Str from recv"))))
      }
    },
    other => Ok(other),
  }
}
