use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{ProtoFieldDef, Value};

pub fn mk_gate_result_protocol() -> Value {
    let fields = vec![
        ProtoFieldDef {
            name: "approved".into(),
            type_name: "Bool".into(),
            default: None,
        },
        ProtoFieldDef {
            name: "approver".into(),
            type_name: "Str".into(),
            default: None,
        },
        ProtoFieldDef {
            name: "reason".into(),
            type_name: "Str".into(),
            default: Some(Value::Str(Arc::from(""))),
        },
        ProtoFieldDef {
            name: "timestamp".into(),
            type_name: "Str".into(),
            default: None,
        },
    ];
    Value::Protocol {
        name: Arc::from("GateResult"),
        fields: Arc::new(fields),
    }
}

pub fn mk_gate() -> Value {
    mk("agent.gate", 2, bi_gate)
}

fn bi_gate(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let name = args[0].as_str().ok_or_else(|| {
        LxError::type_err("agent.gate: first arg must be gate name (Str)", span)
    })?;
    let config = &args[1];
    let (show, timeout, on_timeout) = parse_gate_config(config, span)?;
    let mut gate_msg = IndexMap::new();
    gate_msg.insert("type".into(), Value::Str(Arc::from("gate")));
    gate_msg.insert("name".into(), Value::Str(Arc::from(name)));
    if let Some(show_val) = show {
        gate_msg.insert("show".into(), show_val);
    }
    gate_msg.insert("timeout".into(), Value::Int(timeout.into()));
    gate_msg.insert(
        "on_timeout".into(),
        Value::Str(Arc::from(on_timeout.as_str())),
    );
    let gate_val = Value::Record(Arc::new(gate_msg));
    let response = ctx.yield_.yield_value(gate_val, span)?;
    let now = chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    match &response {
        Value::Record(r) => {
            let approved = r
                .get("approved")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let approver = r
                .get("approver")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let reason = r
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let mut result = IndexMap::new();
            result.insert("approved".into(), Value::Bool(approved));
            result.insert(
                "approver".into(),
                Value::Str(Arc::from(approver.as_str())),
            );
            result.insert(
                "reason".into(),
                Value::Str(Arc::from(reason.as_str())),
            );
            result.insert("timestamp".into(), Value::Str(Arc::from(now.as_str())));
            if approved {
                Ok(Value::Ok(Box::new(Value::Record(Arc::new(result)))))
            } else {
                let msg = if reason.is_empty() {
                    format!("gate '{name}' rejected")
                } else {
                    format!("gate '{name}' rejected: {reason}")
                };
                Ok(Value::Err(Box::new(Value::Str(Arc::from(msg.as_str())))))
            }
        }
        _ => {
            handle_timeout_policy(name, &on_timeout, &now)
        }
    }
}

fn handle_timeout_policy(
    name: &str,
    on_timeout: &str,
    now: &str,
) -> Result<Value, LxError> {
    match on_timeout {
        "approve" => {
            let mut result = IndexMap::new();
            result.insert("approved".into(), Value::Bool(true));
            result.insert(
                "approver".into(),
                Value::Str(Arc::from("auto_timeout")),
            );
            result.insert("reason".into(), Value::Str(Arc::from("auto-approved on timeout")));
            result.insert("timestamp".into(), Value::Str(Arc::from(now)));
            Ok(Value::Ok(Box::new(Value::Record(Arc::new(result)))))
        }
        "reject" => {
            let msg = format!("gate '{name}' auto-rejected on timeout");
            Ok(Value::Err(Box::new(Value::Str(Arc::from(msg.as_str())))))
        }
        _ => {
            let msg = format!("gate '{name}' timed out");
            Ok(Value::Err(Box::new(Value::Str(Arc::from(msg.as_str())))))
        }
    }
}

fn parse_gate_config(
    config: &Value,
    span: Span,
) -> Result<(Option<Value>, i64, String), LxError> {
    match config {
        Value::Unit => Ok((None, 0, "abort".into())),
        Value::Record(r) => {
            let show = r.get("show").cloned();
            let timeout = r
                .get("timeout")
                .and_then(|v| v.as_int())
                .and_then(|n| n.try_into().ok())
                .unwrap_or(0i64);
            let on_timeout = r
                .get("on_timeout")
                .and_then(|v| v.as_str())
                .map(|s| s.trim_start_matches(':').to_string())
                .unwrap_or_else(|| "abort".into());
            Ok((show, timeout, on_timeout))
        }
        _ => Err(LxError::type_err(
            "agent.gate: config must be a Record or ()",
            span,
        )),
    }
}
