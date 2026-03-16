use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Instant;

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

struct BudgetState {
    initial: IndexMap<String, f64>,
    used: IndexMap<String, f64>,
    steps: u64,
    start: Instant,
    tight_at: f64,
    critical_at: f64,
    parent_id: Option<u64>,
}

static BUDGETS: LazyLock<DashMap<u64, BudgetState>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("budget.create", 1, bi_create));
    m.insert("spend".into(), mk("budget.spend", 2, bi_spend));
    m.insert("remaining".into(), mk("budget.remaining", 1, bi_remaining));
    m.insert("used".into(), mk("budget.used", 1, bi_used));
    m.insert("used_pct".into(), mk("budget.used_pct", 1, bi_used_pct));
    m.insert("project".into(), mk("budget.project", 2, bi_project));
    m.insert("status".into(), mk("budget.status", 1, bi_status));
    m.insert("slice".into(), mk("budget.slice", 2, bi_slice));
    m
}

fn budget_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__budget_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("budget: expected budget handle", span)),
        _ => Err(LxError::type_err("budget: expected budget Record", span)),
    }
}

fn make_handle(id: u64) -> Value {
    let mut rec = IndexMap::new();
    rec.insert("__budget_id".into(), Value::Int(BigInt::from(id)));
    Value::Record(Arc::new(rec))
}

fn record_to_dimensions(r: &IndexMap<String, Value>) -> IndexMap<String, f64> {
    let mut dims = IndexMap::new();
    for (k, v) in r {
        if k.starts_with('_') {
            continue;
        }
        let val = match v {
            Value::Float(f) => *f,
            Value::Int(n) => n.to_string().parse::<f64>().unwrap_or(0.0),
            _ => continue,
        };
        dims.insert(k.clone(), val);
    }
    dims
}

fn dimensions_to_record(dims: &IndexMap<String, f64>) -> Value {
    let mut fields = IndexMap::new();
    for (k, v) in dims {
        fields.insert(k.clone(), Value::Float(*v));
    }
    Value::Record(Arc::new(fields))
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(opts) = &args[0] else {
        return Err(LxError::type_err("budget.create expects Record", span));
    };
    let tight_at = opts
        .get("tight_at")
        .and_then(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Int(n) => n.to_string().parse().ok(),
            _ => None,
        })
        .unwrap_or(50.0);
    let critical_at = opts
        .get("critical_at")
        .and_then(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Int(n) => n.to_string().parse().ok(),
            _ => None,
        })
        .unwrap_or(80.0);
    let initial = record_to_dimensions(opts);
    let used = initial
        .keys()
        .map(|k| (k.clone(), 0.0))
        .collect();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    BUDGETS.insert(
        id,
        BudgetState {
            initial,
            used,
            steps: 0,
            start: Instant::now(),
            tight_at,
            critical_at,
            parent_id: None,
        },
    );
    Ok(make_handle(id))
}

fn bi_spend(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = budget_id(&args[0], span)?;
    let Value::Record(amounts) = &args[1] else {
        return Err(LxError::type_err("budget.spend expects Record amounts", span));
    };
    let spend = record_to_dimensions(amounts);
    let mut exceeded = Vec::new();
    {
        let mut b = BUDGETS
            .get_mut(&id)
            .ok_or_else(|| LxError::runtime("budget: not found", span))?;
        b.steps += 1;
        for (k, amt) in &spend {
            let entry = b.used.entry(k.clone()).or_insert(0.0);
            *entry += amt;
        }
        for (k, limit) in &b.initial {
            let spent = b.used.get(k).copied().unwrap_or(0.0);
            if spent > *limit {
                exceeded.push(k.clone());
            }
        }
    }
    if let Some(parent_id) = BUDGETS.get(&id).and_then(|b| b.parent_id) {
        propagate_spend(parent_id, &spend, span)?;
    }
    if exceeded.is_empty() {
        Ok(Value::Ok(Box::new(Value::Unit)))
    } else {
        let names: Vec<Value> = exceeded
            .into_iter()
            .map(|s| Value::Str(Arc::from(s.as_str())))
            .collect();
        Ok(Value::Err(Box::new(Value::Record(Arc::new({
            let mut f = IndexMap::new();
            f.insert("exceeded".into(), Value::List(Arc::new(names)));
            f
        })))))
    }
}

fn propagate_spend(
    parent_id: u64,
    spend: &IndexMap<String, f64>,
    span: Span,
) -> Result<(), LxError> {
    let next_parent = {
        let mut b = BUDGETS
            .get_mut(&parent_id)
            .ok_or_else(|| LxError::runtime("budget: parent not found", span))?;
        for (k, amt) in spend {
            let entry = b.used.entry(k.clone()).or_insert(0.0);
            *entry += amt;
        }
        b.parent_id
    };
    if let Some(gp) = next_parent {
        propagate_spend(gp, spend, span)?;
    }
    Ok(())
}

fn bi_remaining(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = budget_id(&args[0], span)?;
    let b = BUDGETS
        .get(&id)
        .ok_or_else(|| LxError::runtime("budget: not found", span))?;
    let mut fields = IndexMap::new();
    for (k, limit) in &b.initial {
        let spent = b.used.get(k).copied().unwrap_or(0.0);
        fields.insert(k.clone(), Value::Float(limit - spent));
    }
    if b.initial.contains_key("wall_time") {
        let elapsed = b.start.elapsed().as_secs_f64();
        let limit = b.initial["wall_time"];
        fields.insert("wall_time".into(), Value::Float(limit - elapsed));
    }
    Ok(Value::Record(Arc::new(fields)))
}

fn bi_used(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = budget_id(&args[0], span)?;
    let b = BUDGETS
        .get(&id)
        .ok_or_else(|| LxError::runtime("budget: not found", span))?;
    let mut used = b.used.clone();
    if b.initial.contains_key("wall_time") {
        used.insert("wall_time".into(), b.start.elapsed().as_secs_f64());
    }
    Ok(dimensions_to_record(&used))
}

fn bi_used_pct(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = budget_id(&args[0], span)?;
    let b = BUDGETS
        .get(&id)
        .ok_or_else(|| LxError::runtime("budget: not found", span))?;
    let mut fields = IndexMap::new();
    for (k, limit) in &b.initial {
        if *limit == 0.0 {
            fields.insert(k.clone(), Value::Float(100.0));
            continue;
        }
        let spent = if k == "wall_time" {
            b.start.elapsed().as_secs_f64()
        } else {
            b.used.get(k).copied().unwrap_or(0.0)
        };
        fields.insert(k.clone(), Value::Float(spent / limit * 100.0));
    }
    Ok(Value::Record(Arc::new(fields)))
}

fn bi_project(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = budget_id(&args[0], span)?;
    let Value::Record(opts) = &args[1] else {
        return Err(LxError::type_err("budget.project expects Record", span));
    };
    let remaining_steps: f64 = opts
        .get("remaining_steps")
        .and_then(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Int(n) => n.to_string().parse().ok(),
            _ => None,
        })
        .ok_or_else(|| {
            LxError::type_err("budget.project: remaining_steps required", span)
        })?;
    let b = BUDGETS
        .get(&id)
        .ok_or_else(|| LxError::runtime("budget: not found", span))?;
    let steps_done = b.steps.max(1) as f64;
    let total_steps = steps_done + remaining_steps;
    let mut projected_total = IndexMap::new();
    let mut will_exceed = Vec::new();
    let mut headroom = IndexMap::new();
    for (k, limit) in &b.initial {
        let spent = b.used.get(k).copied().unwrap_or(0.0);
        let avg_per_step = spent / steps_done;
        let projected = avg_per_step * total_steps;
        projected_total.insert(k.clone(), Value::Float(projected));
        let room = limit - projected;
        headroom.insert(k.clone(), Value::Float(room));
        if projected > *limit {
            will_exceed.push(Value::Str(Arc::from(k.as_str())));
        }
    }
    let mut fields = IndexMap::new();
    fields.insert(
        "projected_total".into(),
        Value::Record(Arc::new(projected_total)),
    );
    fields.insert("will_exceed".into(), Value::List(Arc::new(will_exceed)));
    fields.insert("headroom".into(), Value::Record(Arc::new(headroom)));
    Ok(Value::Record(Arc::new(fields)))
}

fn bi_status(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = budget_id(&args[0], span)?;
    let b = BUDGETS
        .get(&id)
        .ok_or_else(|| LxError::runtime("budget: not found", span))?;
    let mut max_pct: f64 = 0.0;
    let mut any_exceeded = false;
    for (k, limit) in &b.initial {
        if *limit == 0.0 {
            any_exceeded = true;
            continue;
        }
        let spent = if k == "wall_time" {
            b.start.elapsed().as_secs_f64()
        } else {
            b.used.get(k).copied().unwrap_or(0.0)
        };
        if spent > *limit {
            any_exceeded = true;
        }
        let pct = spent / limit * 100.0;
        if pct > max_pct {
            max_pct = pct;
        }
    }
    let status = if any_exceeded {
        "exceeded"
    } else if max_pct >= b.critical_at {
        "critical"
    } else if max_pct >= b.tight_at {
        "tight"
    } else {
        "comfortable"
    };
    Ok(Value::Str(Arc::from(status)))
}

fn bi_slice(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let parent_id = budget_id(&args[0], span)?;
    let Value::Record(limits) = &args[1] else {
        return Err(LxError::type_err("budget.slice expects Record limits", span));
    };
    let initial = record_to_dimensions(limits);
    let used = initial
        .keys()
        .map(|k| (k.clone(), 0.0))
        .collect();
    let parent = BUDGETS
        .get(&parent_id)
        .ok_or_else(|| LxError::runtime("budget: parent not found", span))?;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    BUDGETS.insert(
        id,
        BudgetState {
            initial,
            used,
            steps: 0,
            start: parent.start,
            tight_at: parent.tight_at,
            critical_at: parent.critical_at,
            parent_id: Some(parent_id),
        },
    );
    Ok(make_handle(id))
}
