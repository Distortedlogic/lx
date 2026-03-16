use std::sync::Arc;

use indexmap::IndexMap;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::{BUDGETS, budget_id};

pub(super) fn bi_project(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = budget_id(&args[0], span)?;
    let Value::Record(opts) = &args[1] else {
        return Err(LxError::type_err("budget.project expects Record", span));
    };
    let remaining_steps: f64 = opts
        .get("remaining_steps")
        .and_then(|v| match v {
            Value::Float(f) => Some(*f),
            Value::Int(n) => n.to_f64(),
            _ => None,
        })
        .ok_or_else(|| LxError::type_err("budget.project: remaining_steps required", span))?;
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
    Ok(record! {
        "projected_total" => Value::Record(Arc::new(projected_total)),
        "will_exceed" => Value::List(Arc::new(will_exceed)),
        "headroom" => Value::Record(Arc::new(headroom)),
    })
}

pub(super) fn bi_status(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
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
