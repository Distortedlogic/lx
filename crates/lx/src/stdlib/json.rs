use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::json_conv::{json_to_lx, lx_to_json};

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("parse".into(), mk("json.parse", 1, bi_parse));
    m.insert("encode".into(), mk("json.encode", 1, bi_encode));
    m.insert("encode_pretty".into(), mk("json.encode_pretty", 1, bi_encode_pretty));
    m
}

fn bi_parse(args: &[Value], span: Span) -> Result<Value, LxError> {
    let s = args[0].as_str()
        .ok_or_else(|| LxError::type_err("json.parse expects Str", span))?;
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(jv) => Ok(Value::Ok(Box::new(json_to_lx(jv)))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(e.to_string().as_str()))))),
    }
}

fn bi_encode(args: &[Value], span: Span) -> Result<Value, LxError> {
    let jv = lx_to_json(&args[0], span)?;
    match serde_json::to_string(&jv) {
        Ok(s) => Ok(Value::Str(Arc::from(s.as_str()))),
        Err(e) => Err(LxError::runtime(format!("json.encode: {e}"), span)),
    }
}

fn bi_encode_pretty(args: &[Value], span: Span) -> Result<Value, LxError> {
    let jv = lx_to_json(&args[0], span)?;
    match serde_json::to_string_pretty(&jv) {
        Ok(s) => Ok(Value::Str(Arc::from(s.as_str()))),
        Err(e) => Err(LxError::runtime(format!("json.encode_pretty: {e}"), span)),
    }
}
