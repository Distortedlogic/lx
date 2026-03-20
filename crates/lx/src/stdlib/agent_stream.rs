use std::io::{BufRead, Write};
use std::sync::Arc;
use std::sync::mpsc;

use parking_lot::Mutex;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

use super::agent::REGISTRY;

pub fn stream_ask_subprocess(pid: u32, msg: &Value, span: Span) -> Result<Value, LxError> {
    let json = json_conv::lx_to_json(msg, span)?;
    let json_str = serde_json::to_string(&json)
        .map_err(|e| LxError::runtime(format!("~>>?: JSON encode: {e}"), span))?;
    let mut agent = REGISTRY
        .get_mut(&pid)
        .ok_or_else(|| LxError::runtime(format!("~>>?: agent {pid} not found"), span))?;
    writeln!(agent.stdin, "{json_str}")
        .map_err(|e| LxError::runtime(format!("~>>?: write error: {e}"), span))?;
    agent
        .stdin
        .flush()
        .map_err(|e| LxError::runtime(format!("~>>?: flush error: {e}"), span))?;
    drop(agent);

    let (tx, rx) = mpsc::channel();
    let (cancel_tx, cancel_rx) = mpsc::channel::<()>();
    std::thread::spawn(move || {
        loop {
            if cancel_rx.try_recv().is_ok() {
                break;
            }
            let Some(mut agent) = REGISTRY.get_mut(&pid) else {
                break;
            };
            let mut line = String::new();
            match agent.stdout.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    drop(agent);
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let Ok(jv) = serde_json::from_str::<serde_json::Value>(trimmed) else {
                        break;
                    };
                    if let Some(typ) = jv.get("type").and_then(|v| v.as_str()) {
                        match typ {
                            "stream_end" => break,
                            "stream_error" => {
                                let err_msg = jv
                                    .get("error")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("stream error");
                                let _ =
                                    tx.send(Value::Err(Box::new(Value::Str(Arc::from(err_msg)))));
                            }
                            "stream" => {
                                if let Some(val) = jv.get("value") {
                                    let lx_val = json_conv::json_to_lx(val.clone());
                                    if tx.send(Value::Ok(Box::new(lx_val))).is_err() {
                                        break;
                                    }
                                }
                            }
                            _ => {
                                let lx_val = json_conv::json_to_lx(jv);
                                if tx.send(Value::Ok(Box::new(lx_val))).is_err() {
                                    break;
                                }
                            }
                        }
                    } else {
                        let lx_val = json_conv::json_to_lx(jv);
                        if tx.send(Value::Ok(Box::new(lx_val))).is_err() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });
    Ok(Value::Stream {
        rx: Arc::new(Mutex::new(rx)),
        cancel_tx: Arc::new(Mutex::new(Some(cancel_tx))),
    })
}

pub fn builtins() -> Vec<(&'static str, Value)> {
    vec![
        ("emit_stream", mk("agent.emit_stream", 1, bi_emit_stream)),
        ("end_stream", mk("agent.end_stream", 0, bi_end_stream)),
    ]
}

fn bi_emit_stream(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let json = json_conv::lx_to_json(&args[0], span)?;
    let envelope = serde_json::json!({
        "type": "stream",
        "value": json,
    });
    let line = serde_json::to_string(&envelope)
        .map_err(|e| LxError::runtime(format!("emit_stream: JSON encode: {e}"), span))?;
    ctx.emit.emit(&Value::Str(Arc::from(line.as_str())), span)?;
    Ok(Value::Unit)
}

fn bi_end_stream(_args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let line = r#"{"type":"stream_end"}"#;
    ctx.emit.emit(&Value::Str(Arc::from(line)), span)?;
    Ok(Value::Unit)
}
