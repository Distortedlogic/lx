use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{BuiltinFunc, BuiltinKind, Value};

pub fn mk_adapter() -> Value {
    mk("agent.adapter", 3, bi_adapter)
}

pub fn mk_coerce() -> Value {
    mk("agent.coerce", 3, bi_coerce)
}

fn extract_mapping(mapping: &Value, span: Span) -> Result<Vec<(String, String)>, LxError> {
    let Value::Record(rec) = mapping else {
        return Err(LxError::type_err(
            "agent.adapter: mapping must be a Record {source_field -> target_field, ...}",
            span,
        ));
    };
    let mut pairs = Vec::new();
    for (key, val) in rec.iter() {
        let target = val.as_str().ok_or_else(|| {
            LxError::type_err(
                format!("agent.adapter: mapping value for '{key}' must be Str"),
                span,
            )
        })?;
        pairs.push((key.clone(), target.to_string()));
    }
    Ok(pairs)
}

fn apply_mapping(msg: &Value, mapping: &[(String, String)], target_proto: Option<&Value>) -> Value {
    let Value::Record(rec) = msg else {
        return Value::Err(Box::new(Value::Str(Arc::from(
            "agent.adapter: message must be a Record",
        ))));
    };
    let mut result = IndexMap::new();
    for (key, val) in rec.iter() {
        let mut renamed = false;
        for (src, tgt) in mapping {
            if key == src {
                result.insert(tgt.clone(), val.clone());
                renamed = true;
                break;
            }
        }
        if !renamed {
            result.insert(key.clone(), val.clone());
        }
    }
    if let Some(proto) = target_proto
        && let Some(err_msg) = validate_against_protocol(&result, proto)
    {
        return Value::Err(Box::new(Value::Str(Arc::from(err_msg.as_str()))));
    }
    Value::Record(Arc::new(result))
}

fn validate_against_protocol(rec: &IndexMap<String, Value>, proto: &Value) -> Option<String> {
    let Value::Trait { name, fields, .. } = proto else {
        return None;
    };
    for field in fields.iter() {
        if field.default.is_none() && !rec.contains_key(&field.name) {
            return Some(format!(
                "target Protocol '{name}' requires field '{}' but it is missing",
                field.name
            ));
        }
    }
    None
}

fn bi_adapter(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let source_proto = &args[0];
    let target_proto = &args[1];
    let mapping_val = &args[2];
    if !matches!(source_proto, Value::Trait { .. }) {
        return Err(LxError::type_err(
            "agent.adapter: first arg must be a Protocol (Trait with fields)",
            span,
        ));
    }
    if !matches!(target_proto, Value::Trait { .. }) {
        return Err(LxError::type_err(
            "agent.adapter: second arg must be a Protocol (Trait with fields)",
            span,
        ));
    }
    let mapping = extract_mapping(mapping_val, span)?;
    let mapping_vals = serialize_mapping(&mapping);
    Ok(Value::BuiltinFunc(BuiltinFunc {
        name: "agent.adapter.transform",
        arity: 3,
        kind: BuiltinKind::Sync(bi_adapter_transform),
        applied: vec![target_proto.clone(), Value::List(Arc::new(mapping_vals))],
    }))
}

pub(super) fn bi_adapter_transform(
    args: &[Value],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let target_proto = &args[0];
    let mapping_list = &args[1];
    let msg = &args[2];
    let mapping = deserialize_mapping(mapping_list);
    Ok(apply_mapping(msg, &mapping, Some(target_proto)))
}

fn bi_coerce(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let msg = &args[0];
    let target_proto = &args[1];
    let mapping_val = &args[2];
    if !matches!(target_proto, Value::Trait { .. }) {
        return Err(LxError::type_err(
            "agent.coerce: second arg must be a Protocol",
            span,
        ));
    }
    let mapping = extract_mapping(mapping_val, span)?;
    let result = apply_mapping(msg, &mapping, Some(target_proto));
    match result {
        Value::Err(_) => Ok(result),
        _ => Ok(Value::Ok(Box::new(result))),
    }
}

pub(super) fn serialize_mapping(mapping: &[(String, String)]) -> Vec<Value> {
    mapping
        .iter()
        .map(|(s, t)| {
            let mut pair = IndexMap::new();
            pair.insert("from".into(), Value::Str(Arc::from(s.as_str())));
            pair.insert("to".into(), Value::Str(Arc::from(t.as_str())));
            Value::Record(Arc::new(pair))
        })
        .collect()
}

pub(super) fn deserialize_mapping(mapping_list: &Value) -> Vec<(String, String)> {
    let Value::List(pairs) = mapping_list else {
        return vec![];
    };
    pairs
        .iter()
        .filter_map(|v| {
            let Value::Record(r) = v else { return None };
            let from = r.get("from")?.as_str()?.to_string();
            let to = r.get("to")?.as_str()?.to_string();
            Some((from, to))
        })
        .collect()
}
