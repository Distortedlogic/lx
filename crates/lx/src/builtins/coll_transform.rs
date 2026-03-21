use std::sync::Arc;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{LxVal, ValueKey};

use super::coll::cmp_values;
use super::mk;

fn bi_sort(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let l = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("sort expects List, got {}", args[0].type_name()),
            span,
        )
    })?;
    let mut items = l.as_ref().clone();
    items.sort_by(cmp_values);
    Ok(LxVal::List(Arc::new(items)))
}

fn bi_sorted_q(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let l = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("sorted? expects List, got {}", args[0].type_name()),
            span,
        )
    })?;
    Ok(LxVal::Bool(
        l.windows(2).all(|w| cmp_values(&w[0], &w[1]).is_le()),
    ))
}

fn bi_rev(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let l = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("rev expects List, got {}", args[0].type_name()),
            span,
        )
    })?;
    let mut items = l.as_ref().clone();
    items.reverse();
    Ok(LxVal::List(Arc::new(items)))
}

fn num_fold(
    name: &str,
    list: &[LxVal],
    init_int: BigInt,
    init_float: f64,
    op_int: fn(&BigInt, &BigInt) -> BigInt,
    op_float: fn(f64, f64) -> f64,
    span: Span,
) -> Result<LxVal, LxError> {
    let mut has_float = false;
    let (mut ia, mut fa) = (init_int, init_float);
    for v in list {
        match v {
            LxVal::Int(n) if has_float => {
                fa = op_float(
                    fa,
                    n.to_f64()
                        .ok_or_else(|| LxError::runtime(format!("{name}: int too large"), span))?,
                );
            }
            LxVal::Int(n) => ia = op_int(&ia, n),
            LxVal::Float(f) => {
                if !has_float {
                    has_float = true;
                    fa = ia
                        .to_f64()
                        .ok_or_else(|| LxError::runtime(format!("{name}: int too large"), span))?;
                }
                fa = op_float(fa, *f);
            }
            other => {
                return Err(LxError::type_err(
                    format!("{name}: non-number {}", other.type_name()),
                    span,
                ));
            }
        }
    }
    if has_float {
        Ok(LxVal::Float(fa))
    } else {
        Ok(LxVal::Int(ia))
    }
}

fn bi_sum(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let l = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("sum expects List, got {}", args[0].type_name()),
            span,
        )
    })?;
    num_fold(
        "sum",
        l,
        BigInt::from(0),
        0.0,
        |a, b| a + b,
        |a, b| a + b,
        span,
    )
}

fn bi_product(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let l = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("product expects List, got {}", args[0].type_name()),
            span,
        )
    })?;
    num_fold(
        "product",
        l,
        BigInt::from(1),
        1.0,
        |a, b| a * b,
        |a, b| a * b,
        span,
    )
}

fn bi_min(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let l = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("min expects List, got {}", args[0].type_name()),
            span,
        )
    })?;
    l.iter()
        .min_by(|a, b| cmp_values(a, b))
        .cloned()
        .ok_or_else(|| LxError::runtime("min: empty list", span))
}

fn bi_max(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let l = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("max expects List, got {}", args[0].type_name()),
            span,
        )
    })?;
    l.iter()
        .max_by(|a, b| cmp_values(a, b))
        .cloned()
        .ok_or_else(|| LxError::runtime("max: empty list", span))
}

fn bi_uniq(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let l = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("uniq expects List, got {}", args[0].type_name()),
            span,
        )
    })?;
    let mut out: Vec<LxVal> = Vec::with_capacity(l.len());
    for v in l.iter() {
        if out.last() != Some(v) {
            out.push(v.clone());
        }
    }
    Ok(LxVal::List(Arc::new(out)))
}

fn bi_flatten(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let l = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("flatten expects List, got {}", args[0].type_name()),
            span,
        )
    })?;
    let mut out = Vec::new();
    for v in l.iter() {
        match v {
            LxVal::List(i) => out.extend(i.iter().cloned()),
            o => out.push(o.clone()),
        }
    }
    Ok(LxVal::List(Arc::new(out)))
}

fn bi_has_key(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    match &args[1] {
        LxVal::Map(m) => Ok(LxVal::Bool(m.contains_key(&ValueKey(args[0].clone())))),
        LxVal::Record(r) => {
            let key = args[0].as_str().ok_or_else(|| {
                LxError::type_err(
                    format!(
                        "has_key?: key must be Str for Record, got {}",
                        args[0].type_name()
                    ),
                    span,
                )
            })?;
            Ok(LxVal::Bool(r.contains_key(key)))
        }
        other => Err(LxError::type_err(
            format!("has_key? expects Map/Record, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_remove(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let m = match &args[1] {
        LxVal::Map(m) => m,
        other => {
            return Err(LxError::type_err(
                format!("remove expects Map, got {}", other.type_name()),
                span,
            ));
        }
    };
    let mut out = m.as_ref().clone();
    out.shift_remove(&ValueKey(args[0].clone()));
    Ok(LxVal::Map(Arc::new(out)))
}

fn bi_merge(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    match (&args[0], &args[1]) {
        (LxVal::Map(m1), LxVal::Map(m2)) => {
            let mut merged: IndexMap<ValueKey, LxVal> = m1.as_ref().clone();
            for (k, v) in m2.iter() {
                merged.insert(k.clone(), v.clone());
            }
            Ok(LxVal::Map(Arc::new(merged)))
        }
        _ => Err(LxError::type_err(
            format!(
                "merge expects two Maps, got {} and {}",
                args[0].type_name(),
                args[1].type_name()
            ),
            span,
        )),
    }
}

pub(super) fn register(env: &mut Env) {
    env.bind("sort".into(), mk("sort", 1, bi_sort));
    env.bind("sorted?".into(), mk("sorted?", 1, bi_sorted_q));
    env.bind("rev".into(), mk("rev", 1, bi_rev));
    env.bind("sum".into(), mk("sum", 1, bi_sum));
    env.bind("product".into(), mk("product", 1, bi_product));
    env.bind("min".into(), mk("min", 1, bi_min));
    env.bind("max".into(), mk("max", 1, bi_max));
    env.bind("uniq".into(), mk("uniq", 1, bi_uniq));
    env.bind("flatten".into(), mk("flatten", 1, bi_flatten));
    env.bind("has_key?".into(), mk("has_key?", 2, bi_has_key));
    env.bind("remove".into(), mk("remove", 2, bi_remove));
    env.bind("merge".into(), mk("merge", 2, bi_merge));
}
