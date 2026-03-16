use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::ast::{Expr, FieldKind, SExpr, Section, Spanned};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{LxFunc, Value};

use super::Interpreter;

impl Interpreter {
    fn make_section_func(&self, params: &[&str], body_expr: Expr, span: Span) -> Value {
        let body = Spanned::new(body_expr, span);
        let arity = params.len();
        Value::Func(LxFunc {
            params: params.iter().map(|p| (*p).into()).collect(),
            defaults: vec![None; arity],
            body: Arc::new(body),
            closure: Arc::clone(&self.env),
            arity,
            applied: vec![],
            source_text: Arc::from(self.source.as_str()),
            source_name: Arc::from(""),
        })
    }

    pub(super) fn eval_section(&mut self, sec: &Section, span: Span) -> Result<Value, LxError> {
        match sec {
            Section::Right { op, operand } => {
                let body = Expr::Binary {
                    op: *op,
                    left: Box::new(Spanned::new(Expr::Ident("_x".into()), span)),
                    right: Box::new((**operand).clone()),
                };
                Ok(self.make_section_func(&["_x"], body, span))
            }
            Section::Left { operand, op } => {
                let body = Expr::Binary {
                    op: *op,
                    left: Box::new((**operand).clone()),
                    right: Box::new(Spanned::new(Expr::Ident("_x".into()), span)),
                };
                Ok(self.make_section_func(&["_x"], body, span))
            }
            Section::Field(name) => {
                let body = Expr::FieldAccess {
                    expr: Box::new(Spanned::new(Expr::Ident("_x".into()), span)),
                    field: FieldKind::Named(name.clone()),
                };
                Ok(self.make_section_func(&["_x"], body, span))
            }
            Section::Index(idx) => {
                let body = Expr::FieldAccess {
                    expr: Box::new(Spanned::new(Expr::Ident("_x".into()), span)),
                    field: FieldKind::Index(*idx),
                };
                Ok(self.make_section_func(&["_x"], body, span))
            }
            Section::BinOp(op) => {
                let body = Expr::Binary {
                    op: *op,
                    left: Box::new(Spanned::new(Expr::Ident("_a".into()), span)),
                    right: Box::new(Spanned::new(Expr::Ident("_b".into()), span)),
                };
                Ok(self.make_section_func(&["_a", "_b"], body, span))
            }
        }
    }

    pub(super) fn eval_field_access(
        &mut self,
        expr: &SExpr,
        field: &FieldKind,
        span: Span,
    ) -> Result<Value, LxError> {
        let val = self.eval(expr)?;
        match field {
            FieldKind::Named(name) => match &val {
                Value::Record(r) => r
                    .get(name)
                    .cloned()
                    .ok_or_else(|| LxError::runtime(format!("field '{name}' not found"), span)),
                Value::Agent { methods, init, .. } => {
                    if let Some(m) = methods.get(name) {
                        Ok(m.clone())
                    } else if name == "init" {
                        match init {
                            Some(v) => Ok(*v.clone()),
                            None => Ok(Value::Unit),
                        }
                    } else {
                        Err(LxError::runtime(
                            format!("Agent method '{name}' not found"),
                            span,
                        ))
                    }
                }
                other => Err(LxError::type_err(
                    format!("field access on {}, not Record", other.type_name()),
                    span,
                )),
            },
            FieldKind::Index(idx) => {
                let items = match &val {
                    Value::Tuple(t) => t.as_ref(),
                    Value::List(l) => l.as_ref(),
                    other => {
                        return Err(LxError::type_err(
                            format!("index access on {}, not Tuple/List", other.type_name()),
                            span,
                        ));
                    }
                };
                let i = if *idx < 0 {
                    items.len() as i64 + idx
                } else {
                    *idx
                } as usize;
                items
                    .get(i)
                    .cloned()
                    .ok_or_else(|| LxError::runtime(format!("index {idx} out of bounds"), span))
            }
            FieldKind::Computed(key_expr) => {
                let key = self.eval(key_expr)?;
                match (&val, &key) {
                    (Value::Record(r), Value::Str(s)) => r
                        .get(s.as_ref())
                        .cloned()
                        .ok_or_else(|| LxError::runtime(format!("field '{s}' not found"), span)),
                    (Value::Map(m), Value::Str(s)) => {
                        let vk = crate::value::ValueKey(Value::Str(s.clone()));
                        m.get(&vk)
                            .cloned()
                            .ok_or_else(|| LxError::runtime(format!("key '{s}' not found"), span))
                    }
                    (Value::List(items), Value::Int(n)) => {
                        let i = n.to_i64().ok_or_else(|| {
                            LxError::runtime(format!("index {n} too large for i64"), span)
                        })?;
                        let i = if i < 0 { items.len() as i64 + i } else { i } as usize;
                        items.get(i).cloned().ok_or_else(|| {
                            LxError::runtime(
                                format!("index {i} out of bounds (list length {})", items.len()),
                                span,
                            )
                        })
                    }
                    _ => Err(LxError::type_err(
                        format!(
                            "computed field access: unsupported types {} / {}",
                            val.type_name(),
                            key.type_name()
                        ),
                        span,
                    )),
                }
            }
        }
    }

    pub(super) fn eval_ternary(
        &mut self,
        cond: &SExpr,
        then_: &SExpr,
        else_: &Option<Box<SExpr>>,
        span: Span,
    ) -> Result<Value, LxError> {
        let cv = self.eval(cond)?;
        match cv.as_bool() {
            Some(true) => self.eval(then_),
            Some(false) => match else_ {
                Some(e) => self.eval(e),
                None => Ok(Value::Unit),
            },
            _ => Err(LxError::type_err(
                format!(
                    "ternary `?` condition must be Bool, got {} `{}`",
                    cv.type_name(),
                    cv.short_display()
                ),
                span,
            )),
        }
    }
}
