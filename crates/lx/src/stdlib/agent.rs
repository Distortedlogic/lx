use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

struct AgentProcess {
    _child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

static REGISTRY: LazyLock<DashMap<u32, AgentProcess>> = LazyLock::new(DashMap::new);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("spawn".into(), mk("agent.spawn", 1, bi_spawn));
    m.insert("ask".into(), mk("agent.ask", 2, bi_ask));
    m.insert("send".into(), mk("agent.send", 2, bi_send));
    m.insert("kill".into(), mk("agent.kill", 1, bi_kill));
    m.insert("name".into(), mk("agent.name", 1, bi_name));
    m.insert("status".into(), mk("agent.status", 1, bi_status));
    m
}

fn get_pid(agent: &Value, span: Span) -> Result<u32, LxError> {
    match agent {
        Value::Record(r) => r.get("__pid")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("agent: expected agent record with __pid", span)),
        _ => Err(LxError::type_err("agent: expected agent Record", span)),
    }
}

fn bi_spawn(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::Record(config) = &args[0] else {
        return Err(LxError::type_err("agent.spawn expects Record config", span));
    };
    let script = config.get("script")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("agent.spawn: config must have 'script' field (Str)", span))?;
    let name = config.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed")
        .to_string();
    let lx_bin = std::env::current_exe()
        .map_err(|e| LxError::runtime(format!("agent.spawn: cannot find lx binary: {e}"), span))?;
    let mut child = Command::new(lx_bin)
        .arg("agent")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| LxError::runtime(format!("agent.spawn: failed: {e}"), span))?;
    let stdin = BufWriter::new(child.stdin.take()
        .ok_or_else(|| LxError::runtime("agent.spawn: no stdin pipe", span))?);
    let stdout = BufReader::new(child.stdout.take()
        .ok_or_else(|| LxError::runtime("agent.spawn: no stdout pipe", span))?);
    let pid = child.id();
    REGISTRY.insert(pid, AgentProcess { _child: child, stdin, stdout });
    let mut rec = IndexMap::new();
    rec.insert("__pid".into(), Value::Int(BigInt::from(pid)));
    rec.insert("name".into(), Value::Str(Arc::from(name.as_str())));
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(rec)))))
}

pub fn ask_subprocess(pid: u32, msg: &Value, span: Span) -> Result<Value, LxError> {
    let json = json_conv::lx_to_json(msg, span)?;
    let json_str = serde_json::to_string(&json)
        .map_err(|e| LxError::runtime(format!("agent.ask: JSON encode: {e}"), span))?;
    let mut agent = REGISTRY.get_mut(&pid)
        .ok_or_else(|| LxError::runtime(format!("agent.ask: agent {pid} not found"), span))?;
    writeln!(agent.stdin, "{json_str}")
        .map_err(|e| LxError::runtime(format!("agent.ask: write error: {e}"), span))?;
    agent.stdin.flush()
        .map_err(|e| LxError::runtime(format!("agent.ask: flush error: {e}"), span))?;
    let mut response = String::new();
    agent.stdout.read_line(&mut response)
        .map_err(|e| LxError::runtime(format!("agent.ask: read error: {e}"), span))?;
    if response.is_empty() {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from("agent disconnected")))));
    }
    let jv: serde_json::Value = serde_json::from_str(response.trim())
        .map_err(|e| LxError::runtime(format!("agent.ask: JSON decode: {e}"), span))?;
    if let Some(err_msg) = jv.get("__err").and_then(|v| v.as_str()) {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(err_msg)))));
    }
    Ok(Value::Ok(Box::new(json_conv::json_to_lx(jv))))
}

pub fn send_subprocess(pid: u32, msg: &Value, span: Span) -> Result<Value, LxError> {
    let json = json_conv::lx_to_json(msg, span)?;
    let json_str = serde_json::to_string(&json)
        .map_err(|e| LxError::runtime(format!("agent.send: JSON encode: {e}"), span))?;
    let mut agent = REGISTRY.get_mut(&pid)
        .ok_or_else(|| LxError::runtime(format!("agent.send: agent {pid} not found"), span))?;
    writeln!(agent.stdin, "{json_str}")
        .map_err(|e| LxError::runtime(format!("agent.send: write: {e}"), span))?;
    agent.stdin.flush()
        .map_err(|e| LxError::runtime(format!("agent.send: flush: {e}"), span))?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_ask(args: &[Value], span: Span) -> Result<Value, LxError> {
    let pid = get_pid(&args[0], span)?;
    ask_subprocess(pid, &args[1], span)
}

fn bi_send(args: &[Value], span: Span) -> Result<Value, LxError> {
    let pid = get_pid(&args[0], span)?;
    send_subprocess(pid, &args[1], span)
}

fn bi_kill(args: &[Value], span: Span) -> Result<Value, LxError> {
    let pid = get_pid(&args[0], span)?;
    match REGISTRY.remove(&pid) {
        Some((_, mut agent)) => {
            let _ = agent._child.kill();
            let _ = agent._child.wait();
            Ok(Value::Ok(Box::new(Value::Unit)))
        }
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from("agent not found"))))),
    }
}

fn bi_name(args: &[Value], span: Span) -> Result<Value, LxError> {
    match &args[0] {
        Value::Record(r) => Ok(r.get("name")
            .cloned()
            .unwrap_or(Value::Str(Arc::from("unnamed")))),
        _ => Err(LxError::type_err("agent.name expects agent Record", span)),
    }
}

fn bi_status(args: &[Value], span: Span) -> Result<Value, LxError> {
    let pid = get_pid(&args[0], span)?;
    let state = if REGISTRY.contains_key(&pid) { "running" } else { "stopped" };
    let mut rec = IndexMap::new();
    rec.insert("state".into(), Value::Str(Arc::from(state)));
    rec.insert("pid".into(), Value::Int(BigInt::from(pid)));
    Ok(Value::Record(Arc::new(rec)))
}
