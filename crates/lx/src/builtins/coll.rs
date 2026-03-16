use std::sync::Arc;

use indexmap::IndexMap;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{Value, ValueKey};

use super::mk;

pub(crate) fn cmp_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => x.total_cmp(y),
        (Value::Int(x), Value::Float(y)) => x
            .to_f64()
            .map_or(std::cmp::Ordering::Greater, |xf| xf.total_cmp(y)),
        (Value::Float(x), Value::Int(y)) => y
            .to_f64()
            .map_or(std::cmp::Ordering::Less, |yf| x.total_cmp(&yf)),
        (Value::Str(x), Value::Str(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}

fn maybe(v: Option<&Value>) -> Value {
    v.map_or(Value::None, |v| Value::Some(Box::new(v.clone())))
}

fn bi_first(args: &[Value], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(maybe(
        args[0]
            .as_list()
            .ok_or_else(|| {
                LxError::type_err(
                    format!("first expects List, got {}", args[0].type_name()),
                    sp,
                )
            })?
            .first(),
    ))
}

fn bi_last(args: &[Value], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(maybe(
        args[0]
            .as_list()
            .ok_or_else(|| {
                LxError::type_err(
                    format!("last expects List, got {}", args[0].type_name()),
                    sp,
                )
            })?
            .last(),
    ))
}

fn bi_contains(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[1] {
        Value::Str(s) => {
            let needle = args[0].as_str().ok_or_else(|| {
                LxError::type_err(
                    format!(
                        "contains?: needle must be Str for Str haystack, got {}",
                        args[0].type_name()
                    ),
                    span,
                )
            })?;
            Ok(Value::Bool(s.contains(needle)))
        }
        Value::List(l) => Ok(Value::Bool(l.iter().any(|v| v == &args[0]))),
        other => Err(LxError::type_err(
            format!("contains? expects Str/List, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_get(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[1] {
        Value::List(l) => {
            let n = args[0].as_int().ok_or_else(|| {
                LxError::type_err(
                    format!(
                        "get: index must be Int for List, got {}",
                        args[0].type_name()
                    ),
                    span,
                )
            })?;
            let idx = n
                .to_i64()
                .ok_or_else(|| LxError::runtime("get: index out of range", span))?;
            let idx = if idx < 0 { l.len() as i64 + idx } else { idx };
            if idx < 0 {
                return Ok(Value::None);
            }
            Ok(maybe(l.get(idx as usize)))
        }
        Value::Record(r) => {
            let key = args[0].as_str().ok_or_else(|| {
                LxError::type_err(
                    format!(
                        "get: key must be Str for Record, got {}",
                        args[0].type_name()
                    ),
                    span,
                )
            })?;
            Ok(maybe(r.get(key)))
        }
        Value::Map(m) => Ok(maybe(m.get(&ValueKey(args[0].clone())))),
        other => Err(LxError::type_err(
            format!("get expects List/Record/Map, got {}", other.type_name()),
            span,
        )),
    }
}

fn kv_tuple(k: Value, v: Value) -> Value {
    Value::Tuple(Arc::new(vec![k, v]))
}

fn bi_to_list(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Map(m) => Ok(Value::List(Arc::new(
            m.iter()
                .map(|(k, v)| kv_tuple(k.0.clone(), v.clone()))
                .collect(),
        ))),
        other => Err(LxError::type_err(
            format!("to_list expects Map, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_to_map(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Record(r) => Ok(Value::Map(Arc::new(
            r.iter()
                .map(|(k, v)| (ValueKey(Value::Str(Arc::from(k.as_str()))), v.clone()))
                .collect(),
        ))),
        Value::List(l) => {
            let mut m = IndexMap::new();
            for v in l.iter() {
                match v {
                    Value::Tuple(t) if t.len() == 2 => {
                        m.insert(ValueKey(t[0].clone()), t[1].clone());
                    }
                    other => {
                        return Err(LxError::type_err(
                            format!("to_map: element must be 2-tuple, got {}", other.type_name()),
                            span,
                        ));
                    }
                }
            }
            Ok(Value::Map(Arc::new(m)))
        }
        other => Err(LxError::type_err(
            format!("to_map expects Record/List, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_to_record(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let m = match &args[0] {
        Value::Map(m) => m,
        other => {
            return Err(LxError::type_err(
                format!("to_record expects Map, got {}", other.type_name()),
                span,
            ));
        }
    };
    let mut r = IndexMap::new();
    for (k, v) in m.iter() {
        let key = k.0.as_str().ok_or_else(|| {
            LxError::type_err(
                format!("to_record: map key must be Str, got {}", k.0.type_name()),
                span,
            )
        })?;
        r.insert(key.to_string(), v.clone());
    }
    Ok(Value::Record(Arc::new(r)))
}

fn bi_keys(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Map(m) => Ok(Value::List(Arc::new(
            m.keys().map(|k| k.0.clone()).collect(),
        ))),
        Value::Record(r) => Ok(Value::List(Arc::new(
            r.keys()
                .map(|k| Value::Str(Arc::from(k.as_str())))
                .collect(),
        ))),
        other => Err(LxError::type_err(
            format!("keys expects Map/Record, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_values(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Map(m) => Ok(Value::List(Arc::new(m.values().cloned().collect()))),
        Value::Record(r) => Ok(Value::List(Arc::new(r.values().cloned().collect()))),
        other => Err(LxError::type_err(
            format!("values expects Map/Record, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_entries(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Map(m) => Ok(Value::List(Arc::new(
            m.iter()
                .map(|(k, v)| kv_tuple(k.0.clone(), v.clone()))
                .collect(),
        ))),
        Value::Record(r) => Ok(Value::List(Arc::new(
            r.iter()
                .map(|(k, v)| kv_tuple(Value::Str(Arc::from(k.as_str())), v.clone()))
                .collect(),
        ))),
        other => Err(LxError::type_err(
            format!("entries expects Map/Record, got {}", other.type_name()),
            span,
        )),
    }
}

pub(super) fn register(env: &mut Env) {
    env.bind("first".into(), mk("first", 1, bi_first));
    env.bind("last".into(), mk("last", 1, bi_last));
    env.bind("contains?".into(), mk("contains?", 2, bi_contains));
    env.bind("get".into(), mk("get", 2, bi_get));
    env.bind("to_list".into(), mk("to_list", 1, bi_to_list));
    env.bind("to_map".into(), mk("to_map", 1, bi_to_map));
    env.bind("to_record".into(), mk("to_record", 1, bi_to_record));
    env.bind("keys".into(), mk("keys", 1, bi_keys));
    env.bind("values".into(), mk("values", 1, bi_values));
    env.bind("entries".into(), mk("entries", 1, bi_entries));
    super::coll_transform::register(env);
}
