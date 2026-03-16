#[path = "context_evict.rs"]
mod context_evict;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

#[derive(Clone)]
pub(super) struct ContextItem {
    pub key: String,
    pub content: String,
    pub tokens: usize,
    pub priority: String,
    pub pinned: bool,
    pub seq: u64,
}

pub(super) struct ContextWindow {
    pub capacity: usize,
    pub items: Vec<ContextItem>,
    pub next_seq: u64,
}

impl ContextWindow {
    pub(super) fn used(&self) -> usize {
        self.items.iter().map(|i| i.tokens).sum()
    }

    pub(super) fn pct(&self) -> f64 {
        if self.capacity == 0 {
            return 100.0;
        }
        self.used() as f64 / self.capacity as f64 * 100.0
    }
}

pub(super) static WINDOWS: LazyLock<DashMap<u64, ContextWindow>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("context.create", 1, bi_create));
    m.insert("add".into(), mk("context.add", 2, bi_add));
    m.insert("usage".into(), mk("context.usage", 1, bi_usage));
    m.insert("pressure".into(), mk("context.pressure", 1, bi_pressure));
    m.insert("estimate".into(), mk("context.estimate", 1, bi_estimate));
    m.insert("pin".into(), mk("context.pin", 2, bi_pin));
    m.insert("unpin".into(), mk("context.unpin", 2, bi_unpin));
    m.insert(
        "evict".into(),
        mk("context.evict", 2, context_evict::bi_evict),
    );
    m.insert(
        "evict_until".into(),
        mk("context.evict_until", 3, context_evict::bi_evict_until),
    );
    m.insert(
        "items".into(),
        mk("context.items", 1, context_evict::bi_items),
    );
    m.insert("get".into(), mk("context.get", 2, context_evict::bi_get));
    m.insert(
        "remove".into(),
        mk("context.remove", 2, context_evict::bi_remove),
    );
    m.insert(
        "clear".into(),
        mk("context.clear", 1, context_evict::bi_clear),
    );
    m
}

pub(super) fn win_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__context_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("context: expected window handle", span)),
        _ => Err(LxError::type_err("context: expected Record", span)),
    }
}

fn make_handle(id: u64) -> Value {
    record! {
        "__context_id" => Value::Int(BigInt::from(id)),
    }
}

pub(super) fn priority_rank(p: &str) -> u8 {
    match p {
        "critical" => 3,
        "high" => 2,
        "normal" => 1,
        "low" => 0,
        _ => 1,
    }
}

pub(super) fn item_to_record(item: &ContextItem) -> Value {
    record! {
        "key" => Value::Str(Arc::from(item.key.as_str())),
        "tokens" => Value::Int(BigInt::from(item.tokens)),
        "priority" => Value::Str(Arc::from(item.priority.as_str())),
        "pinned" => Value::Bool(item.pinned),
    }
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(opts) = &args[0] else {
        return Err(LxError::type_err("context.create expects Record", span));
    };
    let capacity: usize = opts
        .get("capacity")
        .and_then(|v| match v {
            Value::Int(n) => n.try_into().ok(),
            Value::Float(f) => Some(*f as usize),
            _ => None,
        })
        .ok_or_else(|| LxError::type_err("context.create: capacity required", span))?;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    WINDOWS.insert(
        id,
        ContextWindow {
            capacity,
            items: Vec::new(),
            next_seq: 1,
        },
    );
    Ok(make_handle(id))
}

fn bi_add(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let Value::Record(item) = &args[1] else {
        return Err(LxError::type_err("context.add: expects Record item", span));
    };
    let key = item
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::type_err("context.add: key required", span))?;
    let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let estimated = content.len() / 4;
    let tokens: usize = item
        .get("tokens")
        .and_then(|v| match v {
            Value::Int(n) => n.try_into().ok(),
            Value::Float(f) => Some(*f as usize),
            _ => None,
        })
        .unwrap_or(estimated);
    let priority = item
        .get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("normal");
    let mut win = WINDOWS
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    let seq = win.next_seq;
    win.next_seq += 1;
    win.items.retain(|i| i.key != key);
    win.items.push(ContextItem {
        key: key.to_string(),
        content: content.to_string(),
        tokens,
        priority: priority.to_string(),
        pinned: false,
        seq,
    });
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_usage(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let win = WINDOWS
        .get(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    let used = win.used();
    let available = win.capacity.saturating_sub(used);
    Ok(record! {
        "used" => Value::Int(BigInt::from(used)),
        "capacity" => Value::Int(BigInt::from(win.capacity)),
        "available" => Value::Int(BigInt::from(available)),
        "pct" => Value::Float(win.pct()),
    })
}

fn compute_pressure(pct: f64) -> &'static str {
    if pct >= 90.0 {
        "critical"
    } else if pct >= 75.0 {
        "high"
    } else if pct >= 50.0 {
        "moderate"
    } else {
        "low"
    }
}

fn bi_pressure(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let win = WINDOWS
        .get(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    Ok(Value::Str(Arc::from(compute_pressure(win.pct()))))
}

fn bi_estimate(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let text = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("context.estimate: expects Str", span))?;
    Ok(Value::Int(BigInt::from(text.len() / 4)))
}

fn bi_pin(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("context.pin: key must be Str", span))?;
    let mut win = WINDOWS
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    if let Some(item) = win.items.iter_mut().find(|i| i.key == key) {
        item.pinned = true;
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_unpin(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("context.unpin: key must be Str", span))?;
    let mut win = WINDOWS
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    if let Some(item) = win.items.iter_mut().find(|i| i.key == key) {
        item.pinned = false;
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}
