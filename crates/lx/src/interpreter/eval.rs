use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::ast::{BinOp, Literal, SExpr, SStmt, SelArm, StrPart, UnaryOp};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

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
    pub(super) fn eval_literal(&mut self, lit: &Literal, span: Span) -> Result<Value, LxError> {
        match lit {
            Literal::Int(n) => Ok(Value::Int(n.clone())),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::Str(parts) => self.eval_string_parts(parts),
            Literal::RawStr(s) => Ok(Value::Str(Arc::from(s.as_str()))),
            Literal::Regex(s) => {
                let re = regex::Regex::new(s)
                    .map_err(|e| LxError::runtime(format!("invalid regex: {e}"), span))?;
                Ok(Value::Regex(Arc::new(re)))
            }
            Literal::Unit => {
                let _ = span;
                Ok(Value::Unit)
            }
        }
    }

    pub(super) fn eval_binary(
        &mut self,
        op: &BinOp,
        left: &SExpr,
        right: &SExpr,
        span: Span,
    ) -> Result<Value, LxError> {
        if *op == BinOp::And {
            let l = self.eval(left)?;
            let l = self.force_defaults(l, span)?;
            return match l.as_bool() {
                Some(false) => Ok(Value::Bool(false)),
                Some(true) => {
                    let r = self.eval(right)?;
                    self.force_defaults(r, span)
                }
                _ => Err(LxError::type_err(
                    format!("&& requires Bool operands, got {}", l.type_name()),
                    span,
                )),
            };
        }
        if *op == BinOp::Or {
            let l = self.eval(left)?;
            let l = self.force_defaults(l, span)?;
            return match l.as_bool() {
                Some(true) => Ok(Value::Bool(true)),
                Some(false) => {
                    let r = self.eval(right)?;
                    self.force_defaults(r, span)
                }
                _ => Err(LxError::type_err(
                    format!("|| requires Bool operands, got {}", l.type_name()),
                    span,
                )),
            };
        }
        let lv = self.eval(left)?;
        let lv = self.force_defaults(lv, span)?;
        let rv = self.eval(right)?;
        let rv = self.force_defaults(rv, span)?;
        self.binary_op(op, &lv, &rv, span)
    }

    pub(super) fn binary_op(
        &self,
        op: &BinOp,
        lv: &Value,
        rv: &Value,
        span: Span,
    ) -> Result<Value, LxError> {
        match op {
            BinOp::Eq => return Ok(Value::Bool(lv == rv)),
            BinOp::NotEq => return Ok(Value::Bool(lv != rv)),
            _ => {}
        }
        match (op, lv, rv) {
            (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinOp::Div, Value::Int(a), Value::Int(b)) => {
                if b.sign() == num_bigint::Sign::NoSign {
                    return Err(LxError::division_by_zero(span));
                }
                Ok(Value::Int(a / b))
            }
            (BinOp::IntDiv, Value::Int(a), Value::Int(b)) => {
                if b.sign() == num_bigint::Sign::NoSign {
                    return Err(LxError::division_by_zero(span));
                }
                let (q, r) = num_integer::div_rem(a.clone(), b.clone());
                if r.sign() != num_bigint::Sign::NoSign && (a.sign() != b.sign()) {
                    Ok(Value::Int(q - 1))
                } else {
                    Ok(Value::Int(q))
                }
            }
            (BinOp::Mod, Value::Int(a), Value::Int(b)) => {
                if b.sign() == num_bigint::Sign::NoSign {
                    return Err(LxError::division_by_zero(span));
                }
                Ok(Value::Int(a % b))
            }
            (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinOp::Div, Value::Float(_), Value::Float(b)) if *b == 0.0 => {
                Err(LxError::division_by_zero(span))
            }
            (BinOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (BinOp::IntDiv, Value::Float(_), Value::Float(b)) if *b == 0.0 => {
                Err(LxError::division_by_zero(span))
            }
            (BinOp::IntDiv, Value::Float(a), Value::Float(b)) => Ok(Value::Float((a / b).floor())),
            (BinOp::Mod, Value::Float(_), Value::Float(b)) if *b == 0.0 => {
                Err(LxError::division_by_zero(span))
            }
            (BinOp::Mod, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
            (
                op @ (BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::IntDiv
                | BinOp::Mod),
                Value::Int(a),
                Value::Float(b),
            ) => {
                let af = a
                    .to_f64()
                    .ok_or_else(|| LxError::runtime("int too large for float", span))?;
                self.binary_op(op, &Value::Float(af), &Value::Float(*b), span)
            }
            (
                op @ (BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::IntDiv
                | BinOp::Mod),
                Value::Float(a),
                Value::Int(b),
            ) => {
                let bf = b
                    .to_f64()
                    .ok_or_else(|| LxError::runtime("int too large for float", span))?;
                self.binary_op(op, &Value::Float(*a), &Value::Float(bf), span)
            }
            (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (BinOp::LtEq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::GtEq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (BinOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (BinOp::LtEq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::GtEq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (BinOp::Lt, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Gt, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a > b)),
            (BinOp::LtEq, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::GtEq, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a >= b)),
            (BinOp::Concat, Value::Str(a), Value::Str(b)) => {
                let mut s = String::from(a.as_ref());
                s.push_str(b);
                Ok(Value::Str(Arc::from(s)))
            }
            (BinOp::Range, Value::Int(a), Value::Int(b)) => {
                let s = a
                    .to_i64()
                    .ok_or_else(|| LxError::runtime("range start too large", span))?;
                let e = b
                    .to_i64()
                    .ok_or_else(|| LxError::runtime("range end too large", span))?;
                Ok(Value::Range {
                    start: s,
                    end: e,
                    inclusive: false,
                })
            }
            (BinOp::RangeInclusive, Value::Int(a), Value::Int(b)) => {
                let s = a
                    .to_i64()
                    .ok_or_else(|| LxError::runtime("range start too large", span))?;
                let e = b
                    .to_i64()
                    .ok_or_else(|| LxError::runtime("range end too large", span))?;
                Ok(Value::Range {
                    start: s,
                    end: e,
                    inclusive: true,
                })
            }
            (BinOp::Concat, Value::List(a), Value::List(b)) => {
                let mut v = a.as_ref().clone();
                v.extend(b.as_ref().iter().cloned());
                Ok(Value::List(Arc::new(v)))
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

    pub(super) fn eval_unary(
        &mut self,
        op: &UnaryOp,
        operand: &SExpr,
        span: Span,
    ) -> Result<Value, LxError> {
        let v = self.eval(operand)?;
        match (op, &v) {
            (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
            (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            _ => Err(LxError::type_err(
                format!("cannot apply '{op}' to {}", v.type_name()),
                span,
            )),
        }
    }

    pub(super) fn eval_string_parts(&mut self, parts: &[StrPart]) -> Result<Value, LxError> {
        let mut buf = String::new();
        for part in parts {
            match part {
                StrPart::Text(t) => buf.push_str(t),
                StrPart::Interp(e) => {
                    let v = self.eval(e)?;
                    let v = self.force_defaults(v, e.span)?;
                    buf.push_str(&format!("{v}"));
                }
            }
        }
        if buf.starts_with('\n') {
            buf = dedent_string(&buf);
        }
        Ok(Value::Str(Arc::from(buf)))
    }

    pub(super) fn eval_block(&mut self, stmts: &[SStmt]) -> Result<Value, LxError> {
        let saved = Arc::clone(&self.env);
        self.env = Arc::new(self.env.child());
        let mut result = Value::Unit;
        for stmt in stmts {
            result = self.eval_stmt(stmt)?;
        }
        self.env = saved;
        Ok(result)
    }

    pub(super) fn eval_loop(&mut self, stmts: &[SStmt]) -> Result<Value, LxError> {
        loop {
            let saved = Arc::clone(&self.env);
            self.env = Arc::new(self.env.child());
            for stmt in stmts {
                match self.eval_stmt(stmt) {
                    Ok(_) => {}
                    Err(LxError::BreakSignal { value }) => {
                        self.env = saved;
                        return Ok(*value);
                    }
                    Err(e) => {
                        self.env = saved;
                        return Err(e);
                    }
                }
            }
            self.env = saved;
        }
    }

    pub(super) fn eval_slice(
        &mut self,
        expr: &SExpr,
        start: Option<&SExpr>,
        end: Option<&SExpr>,
        span: Span,
    ) -> Result<Value, LxError> {
        let val = self.eval(expr)?;
        let items = match &val {
            Value::List(l) => l.as_ref(),
            other => {
                return Err(LxError::type_err(
                    format!("slice requires List, got {}", other.type_name()),
                    span,
                ));
            }
        };
        let len = items.len();
        let s = match start {
            Some(e) => {
                let v = self.eval(e)?;
                v.as_int()
                    .and_then(|n| n.try_into().ok())
                    .ok_or_else(|| LxError::type_err(
                        format!("slice start index must be Int, got {} `{v}`", v.type_name()),
                        span,
                    ))?
            }
            None => 0usize,
        };
        let en: usize = match end {
            Some(e) => {
                let v = self.eval(e)?;
                v.as_int()
                    .and_then(|n| n.try_into().ok())
                    .ok_or_else(|| LxError::type_err(
                        format!("slice end index must be Int, got {} `{v}`", v.type_name()),
                        span,
                    ))?
            }
            None => len,
        };
        let s = s.min(len);
        let en = en.min(len);
        Ok(Value::List(Arc::new(items[s..en].to_vec())))
    }

    pub(super) fn eval_par(&mut self, stmts: &[SStmt]) -> Result<Value, LxError> {
        let saved = Arc::clone(&self.env);
        self.env = Arc::new(self.env.child());
        let mut results = Vec::new();
        for stmt in stmts {
            let val = self.eval_stmt(stmt)?;
            results.push(val);
        }
        self.env = saved;
        Ok(Value::Tuple(Arc::new(results)))
    }

    pub(super) fn eval_sel(&mut self, arms: &[SelArm], span: Span) -> Result<Value, LxError> {
        if arms.is_empty() {
            return Err(LxError::runtime("sel: no arms", span));
        }
        let val = self.eval(&arms[0].expr)?;
        let saved = Arc::clone(&self.env);
        let mut child = self.env.child();
        child.bind("it".into(), val);
        self.env = Arc::new(child);
        let result = self.eval(&arms[0].handler);
        self.env = saved;
        result
    }

    pub(super) fn eval_with_resource(
        &mut self,
        resources: &[(SExpr, String)],
        body: &[SStmt],
        span: Span,
    ) -> Result<Value, LxError> {
        let mut acquired: Vec<(String, Value)> = Vec::new();
        let setup_result = (|| -> Result<(), LxError> {
            for (expr, name) in resources {
                let val = self.eval(expr)?;
                acquired.push((name.clone(), val));
            }
            Ok(())
        })();
        if let Err(e) = setup_result {
            for (_, val) in acquired.iter().rev() {
                self.close_resource(val, span);
            }
            return Err(e);
        }
        let saved = Arc::clone(&self.env);
        let mut child = self.env.child();
        for (name, val) in &acquired {
            child.bind(name.clone(), val.clone());
        }
        self.env = child.into_arc();
        let body_result = (|| -> Result<Value, LxError> {
            let mut result = Value::Unit;
            for stmt in body {
                result = self.eval_stmt(stmt)?;
            }
            Ok(result)
        })();
        self.env = saved;
        for (_, val) in acquired.iter().rev() {
            self.close_resource(val, span);
        }
        body_result
    }

    fn close_resource(&mut self, val: &Value, span: Span) {
        if let Value::Record(fields) = val {
            if let Some(close_fn) = fields.get("close") {
                let _ = crate::builtins::call_value(close_fn, Value::Unit, span, &self.ctx);
            }
        }
    }

    pub(super) fn eval_assert(
        &mut self,
        expr: &SExpr,
        msg: &Option<Box<SExpr>>,
        span: Span,
    ) -> Result<Value, LxError> {
        let val = self.eval(expr)?;
        let val = self.force_defaults(val, span)?;
        match val.as_bool() {
            Some(true) => Ok(Value::Unit),
            Some(false) => {
                let message = match msg {
                    Some(m) => {
                        let mv = self.eval(m)?;
                        Some(format!("{mv}"))
                    }
                    None => None,
                };
                Err(LxError::assert_fail(
                    format!("{:?}", expr.node),
                    message,
                    span,
                ))
            }
            _ => Err(LxError::type_err(
                format!("assert requires Bool, got {} `{}`", val.type_name(), val.short_display()),
                span,
            )),
        }
    }
}
