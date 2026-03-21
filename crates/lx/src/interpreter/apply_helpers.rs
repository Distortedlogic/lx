use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::ast::{Expr, FieldKind, SExpr, Section, Spanned};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{LxFunc, LxVal};

use super::Interpreter;

impl Interpreter {
  fn make_section_func(&self, params: &[&str], body_expr: Expr, span: Span) -> LxVal {
    let body = Spanned::new(body_expr, span);
    let arity = params.len();
    LxVal::Func(Box::new(LxFunc {
      params: params.iter().map(|p| (*p).into()).collect(),
      defaults: vec![None; arity],
      body: Arc::new(body),
      closure: Arc::clone(&self.env),
      arity,
      applied: vec![],
      source_text: Arc::from(self.source.as_str()),
      source_name: Arc::from(""),
    }))
  }

  pub(super) fn eval_section(&mut self, sec: &Section, span: Span) -> Result<LxVal, LxError> {
    match sec {
      Section::Right { op, operand } => {
        let body = Expr::Binary { op: *op, left: Box::new(Spanned::new(Expr::Ident("_x".into()), span)), right: Box::new((**operand).clone()) };
        Ok(self.make_section_func(&["_x"], body, span))
      },
      Section::Left { operand, op } => {
        let body = Expr::Binary { op: *op, left: Box::new((**operand).clone()), right: Box::new(Spanned::new(Expr::Ident("_x".into()), span)) };
        Ok(self.make_section_func(&["_x"], body, span))
      },
      Section::Field(name) => {
        let body = Expr::FieldAccess { expr: Box::new(Spanned::new(Expr::Ident("_x".into()), span)), field: FieldKind::Named(name.clone()) };
        Ok(self.make_section_func(&["_x"], body, span))
      },
      Section::Index(idx) => {
        let body = Expr::FieldAccess { expr: Box::new(Spanned::new(Expr::Ident("_x".into()), span)), field: FieldKind::Index(*idx) };
        Ok(self.make_section_func(&["_x"], body, span))
      },
      Section::BinOp(op) => {
        let body =
          Expr::Binary { op: *op, left: Box::new(Spanned::new(Expr::Ident("_a".into()), span)), right: Box::new(Spanned::new(Expr::Ident("_b".into()), span)) };
        Ok(self.make_section_func(&["_a", "_b"], body, span))
      },
    }
  }

  pub(super) async fn eval_field_access(&mut self, expr: &SExpr, field: &FieldKind, span: Span) -> Result<LxVal, LxError> {
    let val = self.eval(expr).await?;
    match field {
      FieldKind::Named(name) => match &val {
        LxVal::Record(r) => Ok(r.get(name).cloned().unwrap_or(LxVal::None)),
        LxVal::Class { methods, .. } => {
          if let Some(method) = methods.get(name) {
            Ok(Self::inject_self(method, &val))
          } else {
            Ok(LxVal::None)
          }
        },
        LxVal::Object { id, methods, .. } => {
          if let Some(method) = methods.get(name) {
            Ok(Self::inject_self(method, &val))
          } else {
            Ok(crate::stdlib::object_get_field(*id, name).unwrap_or(LxVal::None))
          }
        },
        LxVal::Store { .. } => crate::stdlib::store_method(name, &val).ok_or_else(|| LxError::type_err(format!("Store has no method '{name}'"), span)),
        other => Err(LxError::type_err(format!("field access on {}, not Record", other.type_name()), span)),
      },
      FieldKind::Index(idx) => {
        let items = match &val {
          LxVal::Tuple(t) => t.as_ref(),
          LxVal::List(l) => l.as_ref(),
          other => {
            return Err(LxError::type_err(format!("index access on {}, not Tuple/List", other.type_name()), span));
          },
        };
        let i = if *idx < 0 { items.len() as i64 + idx } else { *idx } as usize;
        items.get(i).cloned().ok_or_else(|| LxError::runtime(format!("index {idx} out of bounds"), span))
      },
      FieldKind::Computed(key_expr) => {
        let key = self.eval(key_expr).await?;
        match (&val, &key) {
          (LxVal::Record(r), LxVal::Str(s)) => Ok(r.get(s.as_ref()).cloned().unwrap_or(LxVal::None)),
          (LxVal::Map(m), LxVal::Str(s)) => {
            let vk = crate::value::ValueKey(LxVal::Str(s.clone()));
            Ok(m.get(&vk).cloned().unwrap_or(LxVal::None))
          },
          (LxVal::List(items), LxVal::Int(n)) => {
            let i = n.to_i64().ok_or_else(|| LxError::runtime(format!("index {n} too large for i64"), span))?;
            let i = if i < 0 { items.len() as i64 + i } else { i } as usize;
            items.get(i).cloned().ok_or_else(|| LxError::runtime(format!("index {i} out of bounds (list length {})", items.len()), span))
          },
          _ => Err(LxError::type_err(format!("computed field access: unsupported types {} / {}", val.type_name(), key.type_name()), span)),
        }
      },
    }
  }

  fn inject_self(method: &LxVal, self_val: &LxVal) -> LxVal {
    if let LxVal::Func(lf) = method {
      let mut method_env = lf.closure.child();
      method_env.bind("self".to_string(), self_val.clone());
      let mut lf = lf.clone();
      lf.closure = method_env.into_arc();
      LxVal::Func(lf)
    } else {
      method.clone()
    }
  }

  pub(super) async fn eval_ternary(&mut self, cond: &SExpr, then_: &SExpr, else_: &Option<Box<SExpr>>, span: Span) -> Result<LxVal, LxError> {
    let cv = self.eval(cond).await?;
    match cv.as_bool() {
      Some(true) => self.eval(then_).await,
      Some(false) => match else_ {
        Some(e) => self.eval(e).await,
        None => Ok(LxVal::Unit),
      },
      _ => Err(LxError::type_err(format!("ternary `?` condition must be Bool, got {} `{}`", cv.type_name(), cv.short_display()), span)),
    }
  }
}
