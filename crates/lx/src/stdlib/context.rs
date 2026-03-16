use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

#[derive(Clone)]
struct ContextItem {
    key: String,
    content: String,
    tokens: usize,
    priority: String,
    pinned: bool,
    seq: u64,
}

struct ContextWindow {
    capacity: usize,
    items: Vec<ContextItem>,
    next_seq: u64,
}

impl ContextWindow {
    fn used(&self) -> usize {
        self.items.iter().map(|i| i.tokens).sum()
    }

    fn pct(&self) -> f64 {
        if self.capacity == 0 {
            return 100.0;
        }
        self.used() as f64 / self.capacity as f64 * 100.0
    }
}

static WINDOWS: LazyLock<DashMap<u64, ContextWindow>> = LazyLock::new(DashMap::new);
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
    m.insert("evict".into(), mk("context.evict", 2, bi_evict));
    m.insert(
        "evict_until".into(),
        mk("context.evict_until", 3, bi_evict_until),
    );
    m.insert("items".into(), mk("context.items", 1, bi_items));
    m.insert("get".into(), mk("context.get", 2, bi_get));
    m.insert("remove".into(), mk("context.remove", 2, bi_remove));
    m.insert("clear".into(), mk("context.clear", 1, bi_clear));
    m
}

fn win_id(v: &Value, span: Span) -> Result<u64, LxError> {
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
    let mut rec = IndexMap::new();
    rec.insert("__context_id".into(), Value::Int(BigInt::from(id)));
    Value::Record(Arc::new(rec))
}

fn priority_rank(p: &str) -> u8 {
    match p {
        "critical" => 3,
        "high" => 2,
        "normal" => 1,
        "low" => 0,
        _ => 1,
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
    let content = item
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");
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
    let mut fields = IndexMap::new();
    fields.insert("used".into(), Value::Int(BigInt::from(used)));
    fields.insert("capacity".into(), Value::Int(BigInt::from(win.capacity)));
    fields.insert("available".into(), Value::Int(BigInt::from(available)));
    fields.insert("pct".into(), Value::Float(win.pct()));
    Ok(Value::Record(Arc::new(fields)))
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

fn evict_one(win: &mut ContextWindow, strategy: &str) -> bool {
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

fn bi_evict(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
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

fn bi_evict_until(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let strategy = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("context.evict_until: strategy must be Str", span))?;
    let Value::Record(opts) = &args[2] else {
        return Err(LxError::type_err("context.evict_until: expects opts Record", span));
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

fn item_to_record(item: &ContextItem) -> Value {
    let mut fields = IndexMap::new();
    fields.insert("key".into(), Value::Str(Arc::from(item.key.as_str())));
    fields.insert("tokens".into(), Value::Int(BigInt::from(item.tokens)));
    fields.insert(
        "priority".into(),
        Value::Str(Arc::from(item.priority.as_str())),
    );
    fields.insert("pinned".into(), Value::Bool(item.pinned));
    Value::Record(Arc::new(fields))
}

fn bi_items(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let win = WINDOWS
        .get(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    let items: Vec<Value> = win.items.iter().map(item_to_record).collect();
    Ok(Value::List(Arc::new(items)))
}

fn bi_get(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let key = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("context.get: key must be Str", span))?;
    let win = WINDOWS
        .get(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    match win.items.iter().find(|i| i.key == key) {
        Some(item) => {
            let mut fields = IndexMap::new();
            fields.insert("key".into(), Value::Str(Arc::from(item.key.as_str())));
            fields.insert(
                "content".into(),
                Value::Str(Arc::from(item.content.as_str())),
            );
            fields.insert("tokens".into(), Value::Int(BigInt::from(item.tokens)));
            fields.insert(
                "priority".into(),
                Value::Str(Arc::from(item.priority.as_str())),
            );
            fields.insert("pinned".into(), Value::Bool(item.pinned));
            Ok(Value::Some(Box::new(Value::Record(Arc::new(fields)))))
        }
        None => Ok(Value::None),
    }
}

fn bi_remove(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
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

fn bi_clear(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = win_id(&args[0], span)?;
    let mut win = WINDOWS
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("context: window not found", span))?;
    win.items.retain(|i| i.pinned);
    Ok(Value::Ok(Box::new(Value::Unit)))
}
