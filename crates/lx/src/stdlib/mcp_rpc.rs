use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

struct McpProcess {
    _child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

fn registry() -> &'static Mutex<HashMap<u64, McpProcess>> {
    static REG: OnceLock<Mutex<HashMap<u64, McpProcess>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn get_id(client: &Value, span: Span) -> Result<u64, LxError> {
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
    proc: &mut McpProcess,
    method: &str,
    params: &serde_json::Value,
    span: Span,
) -> Result<serde_json::Value, LxError> {
    proc.next_id += 1;
    let req = serde_json::json!({
        "jsonrpc": "2.0", "id": proc.next_id,
        "method": method, "params": params
    });
    let s = serde_json::to_string(&req)
        .map_err(|e| LxError::runtime(format!("mcp: encode: {e}"), span))?;
    writeln!(proc.stdin, "{s}")
        .map_err(|e| LxError::runtime(format!("mcp: write: {e}"), span))?;
    proc.stdin
        .flush()
        .map_err(|e| LxError::runtime(format!("mcp: flush: {e}"), span))?;
    loop {
        let mut line = String::new();
        proc.stdout
            .read_line(&mut line)
            .map_err(|e| LxError::runtime(format!("mcp: read: {e}"), span))?;
        if line.is_empty() {
            return Err(LxError::runtime("mcp: server disconnected", span));
        }
        let jv: serde_json::Value = serde_json::from_str(line.trim())
            .map_err(|e| LxError::runtime(format!("mcp: decode: {e}"), span))?;
        if jv.get("id").is_none() {
            continue;
        }
        if let Some(err) = jv.get("error") {
            let msg = err
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            return Err(LxError::runtime(
                format!("mcp: server error: {msg}"),
                span,
            ));
        }
        return jv
            .get("result")
            .cloned()
            .ok_or_else(|| LxError::runtime("mcp: no result in response", span));
    }
}

fn notify(proc: &mut McpProcess, method: &str, span: Span) -> Result<(), LxError> {
    let s =
        serde_json::to_string(&serde_json::json!({"jsonrpc": "2.0", "method": method}))
            .map_err(|e| LxError::runtime(format!("mcp: encode: {e}"), span))?;
    writeln!(proc.stdin, "{s}")
        .map_err(|e| LxError::runtime(format!("mcp: write: {e}"), span))?;
    proc.stdin
        .flush()
        .map_err(|e| LxError::runtime(format!("mcp: flush: {e}"), span))
}

fn parse_config(val: &Value, span: Span) -> Result<(String, Vec<String>), LxError> {
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

pub(super) fn connect(args: &[Value], span: Span) -> Result<Value, LxError> {
    let (cmd, cmd_args) = parse_config(&args[0], span)?;
    let mut child = Command::new(&cmd)
        .args(&cmd_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| LxError::runtime(format!("mcp.connect: spawn: {e}"), span))?;
    let stdin = BufWriter::new(
        child
            .stdin
            .take()
            .ok_or_else(|| LxError::runtime("mcp.connect: no stdin", span))?,
    );
    let stdout = BufReader::new(
        child
            .stdout
            .take()
            .ok_or_else(|| LxError::runtime("mcp.connect: no stdout", span))?,
    );
    let mut proc = McpProcess {
        _child: child,
        stdin,
        stdout,
        next_id: 0,
    };
    let init_params = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "lx", "version": "0.1.0"}
    });
    rpc(&mut proc, "initialize", &init_params, span)?;
    notify(&mut proc, "notifications/initialized", span)?;
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    registry()
        .lock()
        .map_err(|e| LxError::runtime(format!("mcp registry lock: {e}"), span))?
        .insert(client_id, proc);
    let mut rec = IndexMap::new();
    rec.insert("__mcp_id".into(), Value::Int(BigInt::from(client_id)));
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(rec)))))
}

pub(super) fn close(args: &[Value], span: Span) -> Result<Value, LxError> {
    let id = get_id(&args[0], span)?;
    let mut reg = registry()
        .lock()
        .map_err(|e| LxError::runtime(format!("mcp registry lock: {e}"), span))?;
    match reg.remove(&id) {
        Some(mut proc) => {
            drop(proc.stdin);
            drop(proc.stdout);
            match proc._child.try_wait() {
                Ok(Some(_)) => {}
                _ => {
                    proc._child
                        .kill()
                        .map_err(|e| LxError::runtime(format!("mcp.close: {e}"), span))?;
                    proc._child
                        .wait()
                        .map_err(|e| LxError::runtime(format!("mcp.close: {e}"), span))?;
                }
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
    let mut reg = registry()
        .lock()
        .map_err(|e| LxError::runtime(format!("mcp registry lock: {e}"), span))?;
    let proc = reg
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("mcp: client not found", span))?;
    rpc(proc, method, params, span)
}
