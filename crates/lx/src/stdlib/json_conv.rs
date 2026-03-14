use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn json_to_lx(jv: serde_json::Value) -> Value {
    match jv {
        serde_json::Value::Null => Value::None,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(BigInt::from(i))
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Str(Arc::from(n.to_string().as_str()))
            }
        },
        serde_json::Value::String(s) => Value::Str(Arc::from(s.as_str())),
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr.into_iter().map(json_to_lx).collect();
            Value::List(Arc::new(items))
        },
        serde_json::Value::Object(obj) => {
            let mut fields = IndexMap::new();
            for (k, v) in obj {
                fields.insert(k, json_to_lx(v));
            }
            Value::Record(Arc::new(fields))
        },
    }
}

pub fn lx_to_json(val: &Value, span: Span) -> Result<serde_json::Value, LxError> {
    match val {
        Value::Unit | Value::None => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Int(n) => {
            let i: i64 = n.try_into().map_err(|_| LxError::runtime("json: Int too large for JSON", span))?;
            Ok(serde_json::Value::Number(serde_json::Number::from(i)))
        },
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| LxError::runtime("json: NaN/Infinity not representable in JSON", span)),
        Value::Str(s) => Ok(serde_json::Value::String(s.to_string())),
        Value::List(items) | Value::Tuple(items) => {
            let arr: Result<Vec<_>, _> = items.iter().map(|v| lx_to_json(v, span)).collect();
            Ok(serde_json::Value::Array(arr?))
        },
        Value::Record(fields) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                obj.insert(k.clone(), lx_to_json(v, span)?);
            }
            Ok(serde_json::Value::Object(obj))
        },
        Value::Map(entries) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in entries.iter() {
                let key = match &k.0 {
                    Value::Str(s) => s.to_string(),
                    other => format!("{other}"),
                };
                obj.insert(key, lx_to_json(v, span)?);
            }
            Ok(serde_json::Value::Object(obj))
        },
        Value::Ok(inner) | Value::Some(inner) => lx_to_json(inner, span),
        Value::Err(inner) => {
            let mut obj = serde_json::Map::new();
            obj.insert("error".into(), lx_to_json(inner, span)?);
            Ok(serde_json::Value::Object(obj))
        },
        Value::Tagged { tag, values } => {
            let mut obj = serde_json::Map::new();
            obj.insert("tag".into(), serde_json::Value::String(tag.to_string()));
            let arr: Result<Vec<_>, _> = values.iter().map(|v| lx_to_json(v, span)).collect();
            obj.insert("values".into(), serde_json::Value::Array(arr?));
            Ok(serde_json::Value::Object(obj))
        },
        other => Err(LxError::runtime(
            format!("json: cannot encode {}", other.type_name()),
            span,
        )),
    }
}
