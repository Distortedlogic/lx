use std::io::{BufRead, Write};
use std::sync::Arc;

use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

use super::agent::{REGISTRY, get_pid};

pub fn builtins() -> Vec<(&'static str, Value)> {
    vec![
        ("ask", mk("agent.ask", 2, bi_ask)),
        ("send", mk("agent.send", 2, bi_send)),
        ("name", mk("agent.name", 1, bi_name)),
        ("status", mk("agent.status", 1, bi_status)),
        (
            "implements",
            mk("agent.implements", 2, bi_implements),
        ),
    ]
}

pub fn ask_subprocess(pid: u32, msg: &Value, span: Span) -> Result<Value, LxError> {
    let json = json_conv::lx_to_json(msg, span)?;
    let json_str = serde_json::to_string(&json)
        .map_err(|e| LxError::runtime(format!("agent.ask: JSON encode: {e}"), span))?;
    let mut agent = REGISTRY
        .get_mut(&pid)
        .ok_or_else(|| LxError::runtime(format!("agent.ask: agent {pid} not found"), span))?;
    writeln!(agent.stdin, "{json_str}")
        .map_err(|e| LxError::runtime(format!("agent.ask: write error: {e}"), span))?;
    agent
        .stdin
        .flush()
        .map_err(|e| LxError::runtime(format!("agent.ask: flush error: {e}"), span))?;
    let mut response = String::new();
    agent
        .stdout
        .read_line(&mut response)
        .map_err(|e| LxError::runtime(format!("agent.ask: read error: {e}"), span))?;
    if response.is_empty() {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "agent disconnected",
        )))));
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
    let mut agent = REGISTRY
        .get_mut(&pid)
        .ok_or_else(|| LxError::runtime(format!("agent.send: agent {pid} not found"), span))?;
    writeln!(agent.stdin, "{json_str}")
        .map_err(|e| LxError::runtime(format!("agent.send: write: {e}"), span))?;
    agent
        .stdin
        .flush()
        .map_err(|e| LxError::runtime(format!("agent.send: flush: {e}"), span))?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_ask(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = get_pid(&args[0], span)?;
    ask_subprocess(pid, &args[1], span)
}

fn bi_send(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = get_pid(&args[0], span)?;
    send_subprocess(pid, &args[1], span)
}

fn bi_name(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Record(r) => Ok(r
            .get("name")
            .cloned()
            .unwrap_or(Value::Str(Arc::from("unnamed")))),
        _ => Err(LxError::type_err("agent.name expects agent Record", span)),
    }
}

fn bi_status(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = get_pid(&args[0], span)?;
    let state = if REGISTRY.contains_key(&pid) {
        "running"
    } else {
        "stopped"
    };
    Ok(record! {
        "state" => Value::Str(Arc::from(state)),
        "pid" => Value::Int(BigInt::from(pid)),
    })
}

fn bi_implements(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(agent) = &args[0] else {
        return Err(LxError::type_err(
            "implements?: expected agent record",
            span,
        ));
    };
    let Value::Trait {
        name: trait_name, ..
    } = &args[1]
    else {
        return Err(LxError::type_err("implements?: expected Trait value", span));
    };
    if let Some(Value::List(traits)) = agent.get("__traits") {
        let found = traits.iter().any(|t| {
            if let Value::Str(s) = t {
                s.as_ref() == trait_name.as_ref()
            } else {
                false
            }
        });
        Ok(Value::Bool(found))
    } else {
        Ok(Value::Bool(false))
    }
}
