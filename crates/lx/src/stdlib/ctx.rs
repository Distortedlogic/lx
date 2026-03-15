use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::json_conv::{lx_to_json, json_to_lx};

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("empty".into(), mk("ctx.empty", 1, bi_empty));
    m.insert("load".into(), mk("ctx.load", 1, bi_load));
    m.insert("save".into(), mk("ctx.save", 2, bi_save));
    m.insert("get".into(), mk("ctx.get", 2, bi_get));
    m.insert("set".into(), mk("ctx.set", 3, bi_set));
    m.insert("remove".into(), mk("ctx.remove", 2, bi_remove));
    m.insert("keys".into(), mk("ctx.keys", 1, bi_keys));
    m.insert("merge".into(), mk("ctx.merge", 2, bi_merge));
    m
}

fn bi_empty(_args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(Value::Record(Arc::new(IndexMap::new())))
}

fn bi_load(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0].as_str()
        .ok_or_else(|| LxError::type_err("ctx.load expects Str path", span))?;
    match std::fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(jv) => Ok(Value::Ok(Box::new(json_to_lx(jv)))),
            Err(e) => Ok(Value::Err(Box::new(Value::Str(
                Arc::from(format!("ctx.load: invalid JSON: {e}").as_str()),
            )))),
        },
        Err(e) => Ok(Value::Err(Box::new(Value::Str(
            Arc::from(format!("ctx.load: {e}").as_str()),
        )))),
    }
}

fn bi_save(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0].as_str()
        .ok_or_else(|| LxError::type_err("ctx.save expects Str path", span))?;
    let jv = lx_to_json(&args[1], span)?;
    let json_str = serde_json::to_string_pretty(&jv)
        .map_err(|e| LxError::runtime(format!("ctx.save: {e}"), span))?;
    match std::fs::write(path, json_str) {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(
            Arc::from(format!("ctx.save: {e}").as_str()),
        )))),
    }
}

fn bi_get(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let key = args[0].as_str()
        .ok_or_else(|| LxError::type_err("ctx.get expects Str key", span))?;
    match &args[1] {
        Value::Record(fields) => match fields.get(key) {
            Some(v) => Ok(Value::Some(Box::new(v.clone()))),
            None => Ok(Value::None),
        },
        other => Err(LxError::type_err(
            format!("ctx.get expects Record, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_set(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let key = args[0].as_str()
        .ok_or_else(|| LxError::type_err("ctx.set expects Str key", span))?;
    match &args[2] {
        Value::Record(fields) => {
            let mut new_fields = fields.as_ref().clone();
            new_fields.insert(key.to_string(), args[1].clone());
            Ok(Value::Record(Arc::new(new_fields)))
        },
        other => Err(LxError::type_err(
            format!("ctx.set expects Record, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_remove(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let key = args[0].as_str()
        .ok_or_else(|| LxError::type_err("ctx.remove expects Str key", span))?;
    match &args[1] {
        Value::Record(fields) => {
            let mut new_fields = fields.as_ref().clone();
            new_fields.shift_remove(key);
            Ok(Value::Record(Arc::new(new_fields)))
        },
        other => Err(LxError::type_err(
            format!("ctx.remove expects Record, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_keys(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Record(fields) => {
            let keys: Vec<Value> = fields.keys()
                .map(|k| Value::Str(Arc::from(k.as_str())))
                .collect();
            Ok(Value::List(Arc::new(keys)))
        },
        other => Err(LxError::type_err(
            format!("ctx.keys expects Record, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_merge(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match (&args[0], &args[1]) {
        (Value::Record(a), Value::Record(b)) => {
            let mut merged = a.as_ref().clone();
            for (k, v) in b.iter() {
                merged.insert(k.clone(), v.clone());
            }
            Ok(Value::Record(Arc::new(merged)))
        },
        _ => Err(LxError::type_err(
            format!("ctx.merge expects two Records, got {} and {}", args[0].type_name(), args[1].type_name()),
            span,
        )),
    }
}
