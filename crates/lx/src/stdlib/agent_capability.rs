use std::sync::Arc;

use dashmap::DashMap;
use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{ProtoFieldDef, Value};

static ADVERTISED: std::sync::LazyLock<DashMap<String, Value>> =
    std::sync::LazyLock::new(DashMap::new);

pub fn mk_capabilities_protocol() -> Value {
    let fields = vec![
        ProtoFieldDef {
            name: "protocols".into(),
            type_name: "List".into(),
            default: None,
            constraint: None,
        },
        ProtoFieldDef {
            name: "tools".into(),
            type_name: "List".into(),
            default: None,
            constraint: None,
        },
        ProtoFieldDef {
            name: "domains".into(),
            type_name: "List".into(),
            default: None,
            constraint: None,
        },
        ProtoFieldDef {
            name: "budget_remaining".into(),
            type_name: "Int".into(),
            default: Some(Value::Int((-1).into())),
            constraint: None,
        },
        ProtoFieldDef {
            name: "accepts".into(),
            type_name: "List".into(),
            default: Some(Value::List(Arc::new(vec![]))),
            constraint: None,
        },
        ProtoFieldDef {
            name: "status".into(),
            type_name: "Str".into(),
            default: Some(Value::Str(Arc::from("ready"))),
            constraint: None,
        },
    ];
    Value::Protocol {
        name: Arc::from("Capabilities"),
        fields: Arc::new(fields),
    }
}

pub fn mk_capabilities() -> Value {
    mk("agent.capabilities", 1, bi_capabilities)
}

pub fn mk_advertise() -> Value {
    mk("agent.advertise", 2, bi_advertise)
}

fn bi_capabilities(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = &args[0];
    let mut query = IndexMap::new();
    query.insert("type".into(), Value::Str(Arc::from("capabilities")));
    let query_val = Value::Record(Arc::new(query));
    let Value::Record(r) = agent else {
        return Err(LxError::type_err(
            "agent.capabilities: expected agent Record",
            span,
        ));
    };
    if let Some(pid_val) = r.get("__pid") {
        let pid: u32 = pid_val
            .as_int()
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("agent.capabilities: invalid __pid", span))?;
        return super::agent::ask_subprocess(pid, &query_val, span);
    }
    let handler = r.get("handler").ok_or_else(|| {
        LxError::runtime(
            "agent.capabilities: agent has no 'handler' or '__pid'",
            span,
        )
    })?;
    let result = call_value(handler, query_val, span, ctx)?;
    Ok(Value::Ok(Box::new(result)))
}

fn bi_advertise(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| {
            LxError::type_err("agent.advertise: first arg must be agent name (Str)", span)
        })?
        .to_string();
    let Value::Record(_) = &args[1] else {
        return Err(LxError::type_err(
            "agent.advertise: second arg must be capabilities Record",
            span,
        ));
    };
    ADVERTISED.insert(name, args[1].clone());
    Ok(Value::Ok(Box::new(Value::Unit)))
}
