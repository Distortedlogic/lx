use std::sync::Arc;

use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::{ContextWindow, WINDOWS, item_to_record, priority_rank, win_id};

pub(super) fn evict_one(win: &mut ContextWindow, strategy: &str) -> bool {
    let idx = match strategy {
        "oldest" => win
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| !i.pinned)
            .min_by_key(|(_, i)| i.seq)
            .map(|(idx, _)| idx),
        "lowest_priority" => win
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| !i.pinned)
            .min_by_key(|(_, i)| (priority_rank(&i.priority), i.seq))
            .map(|(idx, _)| idx),
        "largest" => win
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| !i.pinned)
            .max_by_key(|(_, i)| i.tokens)
            .map(|(idx, _)| idx),
        _ => None,
    };
    if let Some(idx) = idx {
        win.items.remove(idx);
        true
    } else {
        false
    }
}

pub(super) fn bi_evict(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let strategy = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("context.evict: strategy must be Str", span))?;
    let mut win = WINDOWS
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    evict_one(&mut win, strategy);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

pub(super) fn bi_evict_until(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let strategy = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("context.evict_until: strategy must be Str", span))?;
    let Value::Record(opts) = &args[2] else {
        return Err(LxError::type_err(
            "context.evict_until: expects opts Record",
            span,
        ));
    };
    let target_pct: f64 = opts
        .get("target_pct")
        .and_then(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Int(n) => n.to_string().parse().ok(),
            _ => None,
        })
        .unwrap_or(50.0);
    let mut win = WINDOWS
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    while win.pct() > target_pct {
        if !evict_one(&mut win, strategy) {
            break;
        }
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}

pub(super) fn bi_items(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let win = WINDOWS
        .get(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    let items: Vec<Value> = win.items.iter().map(item_to_record).collect();
    Ok(Value::List(Arc::new(items)))
}

pub(super) fn bi_get(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("context.get: key must be Str", span))?;
    let win = WINDOWS
        .get(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    match win.items.iter().find(|i| i.key == key) {
        Some(item) => Ok(Value::Some(Box::new(record! {
            "key" => Value::Str(Arc::from(item.key.as_str())),
            "content" => Value::Str(Arc::from(item.content.as_str())),
            "tokens" => Value::Int(BigInt::from(item.tokens)),
            "priority" => Value::Str(Arc::from(item.priority.as_str())),
            "pinned" => Value::Bool(item.pinned),
        }))),
        None => Ok(Value::None),
    }
}

pub(super) fn bi_remove(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("context.remove: key must be Str", span))?;
    let mut win = WINDOWS
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    win.items.retain(|i| i.key != key);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

pub(super) fn bi_clear(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let mut win = WINDOWS
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    win.items.retain(|i| i.pinned);
    Ok(Value::Ok(Box::new(Value::Unit)))
}
