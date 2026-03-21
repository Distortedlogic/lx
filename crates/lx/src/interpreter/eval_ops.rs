use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::ast::{BinOp, Literal, SExpr, StrPart, UnaryOp};
use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::Interpreter;

fn dedent_string(s: &str) -> String {
    let lines: Vec<&str> = s.split('\n').collect();
    let trimmed: Vec<&str> = if lines.first() == Some(&"") {
        lines[1..].to_vec()
    } else {
        lines.to_vec()
    };
    if trimmed.is_empty() {
        return String::new();
    }
    let last_is_whitespace = trimmed
        .last()
        .is_some_and(|l| l.chars().all(|c| c == ' ' || c == '\t'));
    let content_lines = if last_is_whitespace {
        &trimmed[..trimmed.len() - 1]
    } else {
        &trimmed[..]
    };
    let min_indent = content_lines
        .iter()
        .filter(|l| !l.is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut result = String::new();
    for line in content_lines {
        if line.len() >= min_indent {
            result.push_str(&line[min_indent..]);
        }
        result.push('\n');
    }
    result
}

impl Interpreter {
    pub(super) async fn eval_literal(
        &mut self,
        lit: &Literal,
        span: Span,
    ) -> Result<LxVal, LxError> {
        match lit {
            Literal::Int(n) => Ok(LxVal::Int(n.clone())),
            Literal::Float(f) => Ok(LxVal::Float(*f)),
            Literal::Bool(b) => Ok(LxVal::Bool(*b)),
            Literal::Str(parts) => self.eval_string_parts(parts).await,
            Literal::RawStr(s) => Ok(LxVal::Str(Arc::from(s.as_str()))),
            Literal::Regex(s) => {
                let re = regex::Regex::new(s)
                    .map_err(|e| LxError::runtime(format!("invalid regex: {e}"), span))?;
                Ok(LxVal::Regex(Arc::new(re)))
            }
            Literal::Unit => {
                let _ = span;
                Ok(LxVal::Unit)
            }
        }
    }

    pub(super) fn binary_op(
        &self,
        op: &BinOp,
        lv: &LxVal,
        rv: &LxVal,
        span: Span,
    ) -> Result<LxVal, LxError> {
        match op {
            BinOp::Eq => return Ok(LxVal::Bool(lv == rv)),
            BinOp::NotEq => return Ok(LxVal::Bool(lv != rv)),
            _ => {}
        }
        match (op, lv, rv) {
            (BinOp::Add, LxVal::Int(a), LxVal::Int(b)) => Ok(LxVal::Int(a + b)),
            (BinOp::Sub, LxVal::Int(a), LxVal::Int(b)) => Ok(LxVal::Int(a - b)),
            (BinOp::Mul, LxVal::Int(a), LxVal::Int(b)) => Ok(LxVal::Int(a * b)),
            (BinOp::Div, LxVal::Int(a), LxVal::Int(b)) => {
                if b.sign() == num_bigint::Sign::NoSign {
                    return Err(LxError::division_by_zero(span));
                }
                let af = a
                    .to_f64()
                    .ok_or_else(|| LxError::runtime("int too large for float", span))?;
                let bf = b
                    .to_f64()
                    .ok_or_else(|| LxError::runtime("int too large for float", span))?;
                Ok(LxVal::Float(af / bf))
            }
            (BinOp::IntDiv, LxVal::Int(a), LxVal::Int(b)) => {
                if b.sign() == num_bigint::Sign::NoSign {
                    return Err(LxError::division_by_zero(span));
                }
                let (q, r) = num_integer::div_rem(a.clone(), b.clone());
                if r.sign() != num_bigint::Sign::NoSign && (a.sign() != b.sign()) {
                    Ok(LxVal::Int(q - 1))
                } else {
                    Ok(LxVal::Int(q))
                }
            }
            (BinOp::Mod, LxVal::Int(a), LxVal::Int(b)) => {
                if b.sign() == num_bigint::Sign::NoSign {
                    return Err(LxError::division_by_zero(span));
                }
                Ok(LxVal::Int(a % b))
            }
            (BinOp::Add, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Float(a + b)),
            (BinOp::Sub, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Float(a - b)),
            (BinOp::Mul, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Float(a * b)),
            (BinOp::Div, LxVal::Float(_), LxVal::Float(b)) if *b == 0.0 => {
                Err(LxError::division_by_zero(span))
            }
            (BinOp::Div, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Float(a / b)),
            (BinOp::IntDiv, LxVal::Float(_), LxVal::Float(b)) if *b == 0.0 => {
                Err(LxError::division_by_zero(span))
            }
            (BinOp::IntDiv, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Float((a / b).floor())),
            (BinOp::Mod, LxVal::Float(_), LxVal::Float(b)) if *b == 0.0 => {
                Err(LxError::division_by_zero(span))
            }
            (BinOp::Mod, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Float(a % b)),
            (
                op @ (BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::IntDiv
                | BinOp::Mod),
                LxVal::Int(a),
                LxVal::Float(b),
            ) => {
                let af = a
                    .to_f64()
                    .ok_or_else(|| LxError::runtime("int too large for float", span))?;
                self.binary_op(op, &LxVal::Float(af), &LxVal::Float(*b), span)
            }
            (
                op @ (BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::IntDiv
                | BinOp::Mod),
                LxVal::Float(a),
                LxVal::Int(b),
            ) => {
                let bf = b
                    .to_f64()
                    .ok_or_else(|| LxError::runtime("int too large for float", span))?;
                self.binary_op(op, &LxVal::Float(*a), &LxVal::Float(bf), span)
            }
            (BinOp::Lt, LxVal::Int(a), LxVal::Int(b)) => Ok(LxVal::Bool(a < b)),
            (BinOp::Gt, LxVal::Int(a), LxVal::Int(b)) => Ok(LxVal::Bool(a > b)),
            (BinOp::LtEq, LxVal::Int(a), LxVal::Int(b)) => Ok(LxVal::Bool(a <= b)),
            (BinOp::GtEq, LxVal::Int(a), LxVal::Int(b)) => Ok(LxVal::Bool(a >= b)),
            (BinOp::Lt, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Bool(a < b)),
            (BinOp::Gt, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Bool(a > b)),
            (BinOp::LtEq, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Bool(a <= b)),
            (BinOp::GtEq, LxVal::Float(a), LxVal::Float(b)) => Ok(LxVal::Bool(a >= b)),
            (BinOp::Lt, LxVal::Str(a), LxVal::Str(b)) => Ok(LxVal::Bool(a < b)),
            (BinOp::Gt, LxVal::Str(a), LxVal::Str(b)) => Ok(LxVal::Bool(a > b)),
            (BinOp::LtEq, LxVal::Str(a), LxVal::Str(b)) => Ok(LxVal::Bool(a <= b)),
            (BinOp::GtEq, LxVal::Str(a), LxVal::Str(b)) => Ok(LxVal::Bool(a >= b)),
            (BinOp::Concat, LxVal::Str(a), LxVal::Str(b)) => {
                let mut s = String::from(a.as_ref());
                s.push_str(b);
                Ok(LxVal::Str(Arc::from(s)))
            }
            (BinOp::Range, LxVal::Int(a), LxVal::Int(b)) => {
                let s = a
                    .to_i64()
                    .ok_or_else(|| LxError::runtime("range start too large", span))?;
                let e = b
                    .to_i64()
                    .ok_or_else(|| LxError::runtime("range end too large", span))?;
                Ok(LxVal::Range {
                    start: s,
                    end: e,
                    inclusive: false,
                })
            }
            (BinOp::RangeInclusive, LxVal::Int(a), LxVal::Int(b)) => {
                let s = a
                    .to_i64()
                    .ok_or_else(|| LxError::runtime("range start too large", span))?;
                let e = b
                    .to_i64()
                    .ok_or_else(|| LxError::runtime("range end too large", span))?;
                Ok(LxVal::Range {
                    start: s,
                    end: e,
                    inclusive: true,
                })
            }
            (BinOp::Concat, LxVal::List(a), LxVal::List(b)) => {
                let mut v = a.as_ref().clone();
                v.extend(b.as_ref().iter().cloned());
                Ok(LxVal::List(Arc::new(v)))
            }
            _ => Err(LxError::type_err(
                format!(
                    "cannot apply '{op}' to {} and {}",
                    lv.type_name(),
                    rv.type_name()
                ),
                span,
            )),
        }
    }

    pub(super) async fn eval_unary(
        &mut self,
        op: &UnaryOp,
        operand: &SExpr,
        span: Span,
    ) -> Result<LxVal, LxError> {
        let v = self.eval(operand).await?;
        match (op, &v) {
            (UnaryOp::Neg, LxVal::Int(n)) => Ok(LxVal::Int(-n)),
            (UnaryOp::Neg, LxVal::Float(f)) => Ok(LxVal::Float(-f)),
            (UnaryOp::Not, LxVal::Bool(b)) => Ok(LxVal::Bool(!b)),
            _ => Err(LxError::type_err(
                format!("cannot apply '{op}' to {}", v.type_name()),
                span,
            )),
        }
    }

    pub(super) async fn eval_string_parts(&mut self, parts: &[StrPart]) -> Result<LxVal, LxError> {
        let mut buf = String::new();
        for part in parts {
            match part {
                StrPart::Text(t) => buf.push_str(t),
                StrPart::Interp(e) => {
                    let v = self.eval(e).await?;
                    let v = self.force_defaults(v, e.span).await?;
                    buf.push_str(&format!("{v}"));
                }
            }
        }
        if buf.starts_with('\n') {
            buf = dedent_string(&buf);
        }
        Ok(LxVal::Str(Arc::from(buf)))
    }

    pub(super) async fn eval_short_circuit(
        &mut self,
        left: &SExpr,
        right: &SExpr,
        is_and: bool,
        span: Span,
    ) -> Result<LxVal, LxError> {
        let l = self.eval(left).await?;
        let l = self.force_defaults(l, span).await?;
        let short_circuit_on = !is_and;
        let op_name = if is_and { "&&" } else { "||" };
        match l.as_bool() {
            Some(b) if b == short_circuit_on => Ok(LxVal::Bool(short_circuit_on)),
            Some(_) => {
                let r = self.eval(right).await?;
                self.force_defaults(r, span).await
            }
            _ => Err(LxError::type_err(
                format!("{op_name} requires Bool operands, got {}", l.type_name()),
                span,
            )),
        }
    }

    pub(super) async fn close_resource(&mut self, val: &LxVal, span: Span) {
        if let LxVal::Record(fields) = val
            && let Some(close_fn) = fields.get("close")
            && let Err(e) =
                crate::builtins::call_value(close_fn, LxVal::Unit, span, &self.ctx).await
        {
            eprintln!("close_resource: close callback failed: {e}");
        }
    }
}
