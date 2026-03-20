use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use indexmap::IndexMap;
use num_bigint::BigInt;
use tokio::sync::Mutex as TokioMutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, connect_async};

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

type WsSink = Arc<
    TokioMutex<
        futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
            Message,
        >,
    >,
>;

type WsStream = Arc<
    TokioMutex<
        futures::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
        >,
    >,
>;

struct WsConn {
    sink: WsSink,
    stream: WsStream,
}

static WS_CONNS: LazyLock<DashMap<u64, WsConn>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn conn_id(v: &Value, span: Span) -> Result<u64, LxError> {
    let Value::Record(fields) = v else {
        return Err(LxError::type_err(
            "ws: expected connection handle Record",
            span,
        ));
    };
    match fields.get("__ws_id") {
        Some(Value::Int(n)) => {
            u64::try_from(n).map_err(|_| LxError::type_err("ws: invalid connection id", span))
        }
        _ => Err(LxError::type_err("ws: missing __ws_id in handle", span)),
    }
}

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("connect".into(), mk("ws.connect", 1, bi_connect));
    m.insert("send".into(), mk("ws.send", 2, bi_send));
    m.insert("recv".into(), mk("ws.recv", 1, bi_recv));
    m.insert("recv_json".into(), mk("ws.recv_json", 1, bi_recv_json));
    m.insert("close".into(), mk("ws.close", 1, bi_close));
    m
}

fn bi_connect(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let url = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("ws.connect expects Str url", span))?
        .to_string();

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match connect_async(&url).await {
                Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
                    format!("ws.connect: {e}").as_str(),
                ))))),
                Ok((ws_stream, _response)) => {
                    let (sink, stream) = ws_stream.split();
                    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                    WS_CONNS.insert(
                        id,
                        WsConn {
                            sink: Arc::new(TokioMutex::new(sink)),
                            stream: Arc::new(TokioMutex::new(stream)),
                        },
                    );
                    Ok(Value::Ok(Box::new(record! {
                        "__ws_id" => Value::Int(BigInt::from(id)),
                        "url" => Value::Str(Arc::from(url.as_str()))
                    })))
                }
            }
        })
    })
}

fn bi_close(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = conn_id(&args[0], span)?;
    match WS_CONNS.remove(&id) {
        Some((_, conn)) => {
            let val = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    match conn.sink.lock().await.close().await {
                        Ok(()) => Value::Ok(Box::new(Value::Unit)),
                        Err(e) => Value::Err(Box::new(Value::Str(Arc::from(
                            format!("ws.close: {e}").as_str(),
                        )))),
                    }
                })
            });
            Ok(val)
        }
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "ws.close: connection not found",
        ))))),
    }
}

fn bi_send(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = conn_id(&args[0], span)?;
    let msg = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("ws.send expects Str message", span))?
        .to_string();

    let sink = match WS_CONNS.get(&id) {
        Some(conn) => conn.sink.clone(),
        None => {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                "ws.send: connection not found",
            )))));
        }
    };

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match sink.lock().await.send(Message::Text(msg)).await {
                Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
                Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
                    format!("ws.send: {e}").as_str(),
                ))))),
            }
        })
    })
}

fn bi_recv(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = conn_id(&args[0], span)?;
    let (sink, stream) = match WS_CONNS.get(&id) {
        Some(conn) => (conn.sink.clone(), conn.stream.clone()),
        None => {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                "ws.recv: connection not found",
            )))));
        }
    };

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            loop {
                match stream.lock().await.next().await {
                    None => {
                        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                            "connection closed",
                        )))));
                    }
                    Some(Err(e)) => {
                        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                            format!("ws.recv: {e}").as_str(),
                        )))));
                    }
                    Some(Ok(Message::Text(t))) => {
                        return Ok(Value::Ok(Box::new(Value::Str(Arc::from(
                            t.to_string().as_str(),
                        )))));
                    }
                    Some(Ok(Message::Binary(b))) => {
                        let hex: String = b.iter().map(|byte| format!("{byte:02x}")).collect();
                        return Ok(Value::Ok(Box::new(Value::Str(Arc::from(hex.as_str())))));
                    }
                    Some(Ok(Message::Close(_))) => {
                        WS_CONNS.remove(&id);
                        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                            "connection closed",
                        )))));
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        let _ = sink.lock().await.send(Message::Pong(payload)).await;
                    }
                    Some(Ok(_)) => {}
                }
            }
        })
    })
}

fn bi_recv_json(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let recv_result = bi_recv(args, span, _ctx)?;
    match recv_result {
        Value::Ok(inner) => {
            if let Value::Str(s) = *inner {
                match serde_json::from_str::<serde_json::Value>(&s) {
                    Ok(json_val) => Ok(Value::Ok(Box::new(crate::stdlib::json_conv::json_to_lx(
                        json_val,
                    )))),
                    Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
                        format!("ws.recv_json: parse error: {e}").as_str(),
                    ))))),
                }
            } else {
                Ok(Value::Err(Box::new(Value::Str(Arc::from(
                    "ws.recv_json: expected Str from recv",
                )))))
            }
        }
        other => Ok(other),
    }
}
