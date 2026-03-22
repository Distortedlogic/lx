use std::sync::Arc;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use indexmap::IndexMap;

use crate::env::Env;
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::{LxVal, ValueKey};
use miette::SourceSpan;

use super::coll::cmp_values;

fn bi_sort(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let l = args[0].require_list("sort", span)?;
  let mut items = l.to_vec();
  items.sort_by(cmp_values);
  Ok(LxVal::list(items))
}

fn bi_sorted_q(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let l = args[0].require_list("sorted?", span)?;
  Ok(LxVal::Bool(l.windows(2).all(|w| cmp_values(&w[0], &w[1]).is_le())))
}

fn bi_rev(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let l = args[0].require_list("rev", span)?;
  let mut items = l.to_vec();
  items.reverse();
  Ok(LxVal::list(items))
}

fn num_fold(
  name: &str,
  list: &[LxVal],
  init_int: BigInt,
  init_float: f64,
  op_int: fn(&BigInt, &BigInt) -> BigInt,
  op_float: fn(f64, f64) -> f64,
  span: SourceSpan,
) -> Result<LxVal, LxError> {
  let mut has_float = false;
  let (mut ia, mut fa) = (init_int, init_float);
  for v in list {
    match v {
      LxVal::Int(n) if has_float => {
        fa = op_float(fa, n.to_f64().ok_or_else(|| LxError::runtime(format!("{name}: int too large"), span))?);
      },
      LxVal::Int(n) => ia = op_int(&ia, n),
      LxVal::Float(f) => {
        if !has_float {
          has_float = true;
          fa = ia.to_f64().ok_or_else(|| LxError::runtime(format!("{name}: int too large"), span))?;
        }
        fa = op_float(fa, *f);
      },
      other => {
        return Err(LxError::type_err(format!("{name}: non-number {}", other.type_name()), span, None));
      },
    }
  }
  if has_float { Ok(LxVal::Float(fa)) } else { Ok(LxVal::Int(ia)) }
}

fn bi_sum(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let l = args[0].require_list("sum", span)?;
  num_fold("sum", l, BigInt::from(0), 0.0, |a, b| a + b, |a, b| a + b, span)
}

fn bi_product(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let l = args[0].require_list("product", span)?;
  num_fold("product", l, BigInt::from(1), 1.0, |a, b| a * b, |a, b| a * b, span)
}

fn bi_min(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let l = args[0].require_list("min", span)?;
  l.iter().min_by(|a, b| cmp_values(a, b)).cloned().ok_or_else(|| LxError::runtime("min: empty list", span))
}

fn bi_max(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let l = args[0].require_list("max", span)?;
  l.iter().max_by(|a, b| cmp_values(a, b)).cloned().ok_or_else(|| LxError::runtime("max: empty list", span))
}

fn bi_uniq(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let l = args[0].require_list("uniq", span)?;
  let mut out: Vec<LxVal> = Vec::with_capacity(l.len());
  for v in l.iter() {
    if out.last() != Some(v) {
      out.push(v.clone());
    }
  }
  Ok(LxVal::list(out))
}

fn bi_flatten(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let l = args[0].require_list("flatten", span)?;
  let mut out = Vec::new();
  for v in l.iter() {
    match v {
      LxVal::List(i) => out.extend(i.iter().cloned()),
      o => out.push(o.clone()),
    }
  }
  Ok(LxVal::list(out))
}

fn bi_has_key(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match &args[1] {
    LxVal::Map(m) => Ok(LxVal::Bool(m.contains_key(&ValueKey(args[0].clone())))),
    LxVal::Record(r) => {
      let key = args[0].require_str("has_key?", span)?;
      Ok(LxVal::Bool(r.contains_key(&crate::sym::intern(key))))
    },
    other => Err(LxError::type_err(format!("has_key? expects Map/Record, got {}", other.type_name()), span, None)),
  }
}

fn bi_remove(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let m = match &args[1] {
    LxVal::Map(m) => m,
    other => {
      return Err(LxError::type_err(format!("remove expects Map, got {}", other.type_name()), span, None));
    },
  };
  let mut out = m.as_ref().clone();
  out.shift_remove(&ValueKey(args[0].clone()));
  Ok(LxVal::Map(Arc::new(out)))
}

fn bi_merge(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  match (&args[0], &args[1]) {
    (LxVal::Map(m1), LxVal::Map(m2)) => {
      let mut merged: IndexMap<ValueKey, LxVal> = m1.as_ref().clone();
      merged.extend(m2.iter().map(|(k, v)| (k.clone(), v.clone())));
      Ok(LxVal::Map(Arc::new(merged)))
    },
    _ => Err(LxError::type_err(format!("merge expects two Maps, got {} and {}", args[0].type_name(), args[1].type_name()), span, None)),
  }
}

pub(super) fn register(env: &Env) {
  super::register_builtins!(env, {
    "sort"/1 => bi_sort, "sorted?"/1 => bi_sorted_q, "rev"/1 => bi_rev,
    "sum"/1 => bi_sum, "product"/1 => bi_product, "min"/1 => bi_min,
    "max"/1 => bi_max, "uniq"/1 => bi_uniq, "flatten"/1 => bi_flatten,
    "has_key?"/2 => bi_has_key, "remove"/2 => bi_remove, "merge"/2 => bi_merge,
  });
}
