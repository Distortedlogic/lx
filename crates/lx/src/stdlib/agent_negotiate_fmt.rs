use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{BuiltinFunc, BuiltinKind, Value};

pub fn mk_negotiate_format() -> Value {
    mk("agent.negotiate_format", 2, bi_negotiate_format)
}

fn bi_negotiate_format(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let producer = &args[0];
    let consumer = &args[1];
    let producer_caps = get_capabilities(producer, span, ctx)?;
    let consumer_caps = get_capabilities(consumer, span, ctx)?;
    let producer_protos = extract_protocols(&producer_caps);
    let consumer_protos = extract_protocols(&consumer_caps);
    if producer_protos.is_empty() {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "producer has no advertised protocols",
        )))));
    }
    if consumer_protos.is_empty() {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "consumer has no advertised protocols",
        )))));
    }
    for pp in &producer_protos {
        for cp in &consumer_protos {
            if let (Value::Trait { name: pn, .. }, Value::Trait { name: cn, .. }) = (pp, cp)
                && pn == cn
            {
                return Ok(Value::Ok(Box::new(mk_identity_adapter())));
            }
        }
    }
    for pp in &producer_protos {
        for cp in &consumer_protos {
            match try_structural_match(pp, cp) {
                MatchResult::Compatible(mapping) | MatchResult::Subset(mapping) => {
                    let adapter = build_adapter_fn(cp, &mapping);
                    return Ok(Value::Ok(Box::new(adapter)));
                }
                MatchResult::Incompatible => continue,
            }
        }
    }
    let conflicts = collect_conflicts(&producer_protos, &consumer_protos);
    Ok(Value::Err(Box::new(Value::Str(Arc::from(
        conflicts.as_str(),
    )))))
}

fn get_capabilities(agent: &Value, span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(r) = agent else {
        return Err(LxError::type_err(
            "agent.negotiate_format: expected agent Record",
            span,
        ));
    };
    if let Some(pid_val) = r.get("__pid") {
        let pid: u32 = pid_val
            .as_int()
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("agent.negotiate_format: invalid __pid", span))?;
        let mut query = IndexMap::new();
        query.insert("type".into(), Value::Str(Arc::from("capabilities")));
        let query_val = Value::Record(Arc::new(query));
        return super::agent::ask_subprocess(pid, &query_val, span);
    }
    if let Some(handler) = r.get("handler") {
        let mut query = IndexMap::new();
        query.insert("type".into(), Value::Str(Arc::from("capabilities")));
        let query_val = Value::Record(Arc::new(query));
        return crate::builtins::call_value_sync(handler, query_val, span, ctx);
    }
    Err(LxError::runtime(
        "agent.negotiate_format: agent has no 'handler' or '__pid'",
        span,
    ))
}

fn extract_protocols(caps: &Value) -> Vec<Value> {
    let rec = match caps {
        Value::Ok(inner) => match inner.as_ref() {
            Value::Record(r) => r,
            _ => return vec![],
        },
        Value::Record(r) => r,
        _ => return vec![],
    };
    rec.get("protocols")
        .and_then(|v| v.as_list())
        .map(|list| {
            list.iter()
                .filter(|v| matches!(v, Value::Trait { .. }))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

enum MatchResult {
    Compatible(Vec<(String, String)>),
    Subset(Vec<(String, String)>),
    Incompatible,
}

fn try_structural_match(source: &Value, target: &Value) -> MatchResult {
    let (Value::Trait { fields: sf, .. }, Value::Trait { fields: tf, .. }) = (source, target)
    else {
        return MatchResult::Incompatible;
    };
    let mut mapping = Vec::new();
    let mut unmatched = Vec::new();
    for tfield in tf.iter() {
        if tfield.default.is_some() {
            continue;
        }
        let mut found = false;
        for sfield in sf.iter() {
            if sfield.name == tfield.name {
                mapping.push((sfield.name.clone(), tfield.name.clone()));
                found = true;
                break;
            }
        }
        if !found {
            for sfield in sf.iter() {
                if sfield.type_name == tfield.type_name
                    && !mapping.iter().any(|(s, _)| s == &sfield.name)
                {
                    let dist = levenshtein(&sfield.name, &tfield.name);
                    let max_len = sfield.name.len().max(tfield.name.len());
                    if max_len > 0 && dist <= max_len / 2 {
                        mapping.push((sfield.name.clone(), tfield.name.clone()));
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            unmatched.push(tfield.name.clone());
        }
    }
    if unmatched.is_empty() {
        if mapping.iter().all(|(s, t)| s == t) && tf.len() <= sf.len() {
            MatchResult::Subset(mapping)
        } else {
            MatchResult::Compatible(mapping)
        }
    } else {
        MatchResult::Incompatible
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let a_len = a_bytes.len();
    let b_len = b_bytes.len();
    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];
    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b_len]
}

fn collect_conflicts(producer_protos: &[Value], consumer_protos: &[Value]) -> String {
    let mut conflicts = Vec::new();
    for cp in consumer_protos {
        let Value::Trait {
            name: cn,
            fields: cf,
            ..
        } = cp
        else {
            continue;
        };
        let required: Vec<&str> = cf
            .iter()
            .filter(|f| f.default.is_none())
            .map(|f| f.name.as_str())
            .collect();
        let mut best_match = "none";
        for pp in producer_protos {
            if let Value::Trait { name: pn, .. } = pp {
                best_match = pn;
                break;
            }
        }
        conflicts.push(format!(
            "consumer expects '{cn}' (fields: {}) — closest producer protocol: '{best_match}'",
            required.join(", ")
        ));
    }
    conflicts.join("; ")
}

fn mk_identity_adapter() -> Value {
    Value::BuiltinFunc(BuiltinFunc {
        name: "agent.adapter.identity",
        arity: 1,
        kind: BuiltinKind::Sync(bi_identity),
        applied: vec![],
    })
}

fn bi_identity(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(args[0].clone())
}

fn build_adapter_fn(target_proto: &Value, mapping: &[(String, String)]) -> Value {
    let mapping_vals = super::agent_adapter::serialize_mapping(mapping);
    Value::BuiltinFunc(BuiltinFunc {
        name: "agent.adapter.transform",
        arity: 3,
        kind: BuiltinKind::Sync(super::agent_adapter::bi_adapter_transform),
        applied: vec![target_proto.clone(), Value::List(Arc::new(mapping_vals))],
    })
}
