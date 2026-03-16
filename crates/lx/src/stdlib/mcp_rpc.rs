use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::mcp_http::HttpTransport;
use super::mcp_stdio::StdioTransport;

enum McpTransport {
    Stdio(StdioTransport),
    Http(HttpTransport),
}

struct McpConnection {
    transport: McpTransport,
    next_id: u64,
}

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);
static REGISTRY: LazyLock<DashMap<u64, McpConnection>> = LazyLock::new(DashMap::new);

fn get_id(client: &Value, span: Span) -> Result<u64, LxError> {
    match client {
        Value::Record(r) => r
            .get("__mcp_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("mcp: expected client record with __mcp_id", span)),
        _ => Err(LxError::type_err("mcp: expected client Record", span)),
    }
}

fn rpc(
    conn: &mut McpConnection,
    method: &str,
    params: &serde_json::Value,
    span: Span,
) -> Result<serde_json::Value, LxError> {
    conn.next_id += 1;
    let req = serde_json::json!({
        "jsonrpc": "2.0", "id": conn.next_id,
        "method": method, "params": params
    });
    let resp = match &mut conn.transport {
        McpTransport::Stdio(t) => t.send(&req, span)?,
        McpTransport::Http(t) => t.send(&req, span)?,
    };
    if let Some(err) = resp.get("error") {
        let msg = err
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Err(LxError::runtime(format!("mcp: server error: {msg}"), span));
    }
    resp.get("result")
        .cloned()
        .ok_or_else(|| LxError::runtime("mcp: no result in response", span))
}

fn notify(conn: &mut McpConnection, method: &str, span: Span) -> Result<(), LxError> {
    let req = serde_json::json!({"jsonrpc": "2.0", "method": method});
    match &mut conn.transport {
        McpTransport::Stdio(t) => t.send_notify(&req, span),
        McpTransport::Http(t) => t.send_notify(&req, span),
    }
}

fn parse_stdio_config(val: &Value, span: Span) -> Result<(String, Vec<String>), LxError> {
    match val {
        Value::Str(uri) => {
            let uri = uri.as_ref();
            let path = uri
                .strip_prefix("stdio://")
                .ok_or_else(|| {
                    LxError::runtime(format!("mcp.connect: unsupported URI: {uri}"), span)
                })?
                .trim_start_matches('/');
            let mut parts = path.split_whitespace();
            let cmd = parts
                .next()
                .ok_or_else(|| LxError::runtime("mcp.connect: empty stdio path", span))?;
            Ok((cmd.to_string(), parts.map(String::from).collect()))
        }
        Value::Record(r) => {
            let cmd = r
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| LxError::runtime("mcp.connect: needs 'command' field", span))?
                .to_string();
            let args = match r.get("args") {
                Some(Value::List(items)) => items
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .ok_or_else(|| {
                                LxError::runtime("mcp.connect: args must be [Str]", span)
                            })
                            .map(String::from)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                None => Vec::new(),
                _ => return Err(LxError::runtime("mcp.connect: args must be List", span)),
            };
            Ok((cmd, args))
        }
        _ => Err(LxError::type_err(
            "mcp.connect: expects URI Str or config Record",
            span,
        )),
    }
}

fn is_http(val: &Value) -> bool {
    match val {
        Value::Str(s) => s.starts_with("http://") || s.starts_with("https://"),
        Value::Record(r) => r.get("url").and_then(|v| v.as_str()).is_some(),
        _ => false,
    }
}

fn http_url(val: &Value, span: Span) -> Result<String, LxError> {
    match val {
        Value::Str(s) => Ok(s.to_string()),
        Value::Record(r) => r
            .get("url")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| LxError::runtime("mcp.connect: needs 'url' field", span)),
        _ => Err(LxError::type_err(
            "mcp.connect: expects URI or Record",
            span,
        )),
    }
}

fn init_handshake(conn: &mut McpConnection, span: Span) -> Result<(), LxError> {
    let init_params = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "lx", "version": "0.1.0"}
    });
    rpc(conn, "initialize", &init_params, span)?;
    notify(conn, "notifications/initialized", span)
}

pub(super) fn connect(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let transport = if is_http(&args[0]) {
        let url = http_url(&args[0], span)?;
        McpTransport::Http(HttpTransport::new(url, span)?)
    } else {
        let (cmd, cmd_args) = parse_stdio_config(&args[0], span)?;
        McpTransport::Stdio(StdioTransport::spawn(&cmd, &cmd_args, span)?)
    };
    let mut conn = McpConnection {
        transport,
        next_id: 0,
    };
    init_handshake(&mut conn, span)?;
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    REGISTRY.insert(client_id, conn);
    Ok(Value::Ok(Box::new(record! {
        "__mcp_id" => Value::Int(BigInt::from(client_id)),
    })))
}

pub(super) fn close(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = get_id(&args[0], span)?;
    match REGISTRY.remove(&id) {
        Some((_, conn)) => {
            match conn.transport {
                McpTransport::Stdio(t) => t.shutdown(span)?,
                McpTransport::Http(t) => t.shutdown(span)?,
            }
            Ok(Value::Unit)
        }
        None => Err(LxError::runtime("mcp.close: client not found", span)),
    }
}

pub(super) fn with_proc(
    client: &Value,
    method: &str,
    params: &serde_json::Value,
    span: Span,
) -> Result<serde_json::Value, LxError> {
    let id = get_id(client, span)?;
    let mut conn = REGISTRY
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("mcp: client not found", span))?;
    rpc(&mut conn, method, params, span)
}
