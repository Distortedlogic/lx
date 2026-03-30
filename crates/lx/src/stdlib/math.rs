use std::f64::consts::{E, PI};
use std::sync::Arc;

use indexmap::IndexMap;

use num_traits::ToPrimitive;

use crate::BuiltinCtx;
use crate::error::LxError;
use crate::std_module;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<crate::sym::Sym, LxVal> {
  let mut m = std_module! {
    "abs"   => "math.abs",   1, bi_abs;
    "ceil"  => "math.ceil",  1, bi_ceil;
    "floor" => "math.floor", 1, bi_floor;
    "round" => "math.round", 1, bi_round;
    "pow"   => "math.pow",   2, bi_pow;
    "sqrt"  => "math.sqrt",  1, bi_sqrt;
    "min"   => "math.min",   2, bi_min;
    "max"   => "math.max",   2, bi_max
  };
  m.insert(crate::sym::intern("pi"), LxVal::Float(PI));
  m.insert(crate::sym::intern("e"), LxVal::Float(E));
  m.insert(crate::sym::intern("inf"), LxVal::Float(f64::INFINITY));
  m
}

fn to_f64(v: &LxVal, span: SourceSpan) -> Result<f64, LxError> {
  match v {
    LxVal::Float(f) => Ok(*f),
    LxVal::Int(n) => n.to_f64().ok_or_else(|| LxError::runtime("math: Int too large for float", span)),
    other => Err(LxError::type_err(format!("math: expected number, got {}", other.type_name()), span, None)),
  }
}

fn bi_abs(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  match &args[0] {
    LxVal::Int(n) => {
      if n.sign() == num_bigint::Sign::Minus {
        Ok(LxVal::Int(-n))
      } else {
        Ok(LxVal::Int(n.clone()))
      }
    },
    LxVal::Float(f) => Ok(LxVal::Float(f.abs())),
    other => Err(LxError::type_err(format!("math.abs expects number, got {}", other.type_name()), span, None)),
  }
}

fn bi_ceil(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let f = to_f64(&args[0], span)?;
  Ok(LxVal::int(f.ceil() as i64))
}

fn bi_floor(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let f = to_f64(&args[0], span)?;
  Ok(LxVal::int(f.floor() as i64))
}

fn bi_round(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let f = to_f64(&args[0], span)?;
  Ok(LxVal::int(f.round() as i64))
}

fn bi_pow(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  match (&args[0], &args[1]) {
    (LxVal::Int(base), LxVal::Int(exp)) => {
      let e: u32 = exp.try_into().map_err(|_| LxError::runtime("math.pow: exponent too large or negative", span))?;
      Ok(LxVal::Int(base.pow(e)))
    },
    _ => {
      let b = to_f64(&args[0], span)?;
      let e = to_f64(&args[1], span)?;
      Ok(LxVal::Float(b.powf(e)))
    },
  }
}

fn bi_sqrt(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let f = to_f64(&args[0], span)?;
  Ok(LxVal::Float(f.sqrt()))
}

fn bi_min(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  match (&args[0], &args[1]) {
    (LxVal::Int(a), LxVal::Int(b)) => Ok(LxVal::Int(a.min(b).clone())),
    (LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Float(a.min(*b))),
    _ => {
      let a = to_f64(&args[0], span)?;
      let b = to_f64(&args[1], span)?;
      Ok(LxVal::Float(a.min(b)))
    },
  }
}

fn bi_max(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  match (&args[0], &args[1]) {
    (LxVal::Int(a), LxVal::Int(b)) => Ok(LxVal::Int(a.max(b).clone())),
    (LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Float(a.max(*b))),
    _ => {
      let a = to_f64(&args[0], span)?;
      let b = to_f64(&args[1], span)?;
      Ok(LxVal::Float(a.max(b)))
    },
  }
}
