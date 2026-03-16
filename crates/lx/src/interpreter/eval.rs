use std::sync::Arc;

use crate::ast::{BinOp, SExpr, SStmt, SelArm};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::Interpreter;

impl Interpreter {
    pub(super) fn eval_binary(
        &mut self,
        op: &BinOp,
        left: &SExpr,
        right: &SExpr,
        span: Span,
    ) -> Result<Value, LxError> {
        if *op == BinOp::And {
            return self.eval_short_circuit(left, right, true, span);
        }
        if *op == BinOp::Or {
            return self.eval_short_circuit(left, right, false, span);
        }
        let lv = self.eval(left)?;
        let lv = self.force_defaults(lv, span)?;
        let rv = self.eval(right)?;
        let rv = self.force_defaults(rv, span)?;
        self.binary_op(op, &lv, &rv, span)
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
                v.as_int().and_then(|n| n.try_into().ok()).ok_or_else(|| {
                    LxError::type_err(
                        format!("slice start index must be Int, got {} `{v}`", v.type_name()),
                        span,
                    )
                })?
            }
            None => 0usize,
        };
        let en: usize = match end {
            Some(e) => {
                let v = self.eval(e)?;
                v.as_int().and_then(|n| n.try_into().ok()).ok_or_else(|| {
                    LxError::type_err(
                        format!("slice end index must be Int, got {} `{v}`", v.type_name()),
                        span,
                    )
                })?
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
                format!(
                    "assert requires Bool, got {} `{}`",
                    val.type_name(),
                    val.short_display()
                ),
                span,
            )),
        }
    }
}
