use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::ast::*;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{LxFunc, Value};

use super::Interpreter;

impl Interpreter {
  pub(super) fn apply_func(&mut self, func: Value, arg: Value, span: Span) -> Result<Value, LxError> {
    match func {
      Value::Func(mut lf) => {
        lf.applied.push(arg);
        if lf.applied.len() < lf.arity {
          return Ok(Value::Func(lf));
        }
        let saved = Arc::clone(&self.env);
        let mut call_env = lf.closure.clone();
        for (i, name) in lf.params.iter().enumerate() {
          if i < lf.applied.len() {
            call_env.bind(name.clone(), lf.applied[i].clone());
          } else if let Some(Some(def)) = lf.defaults.get(i) {
            call_env.bind(name.clone(), def.clone());
          }
        }
        self.env = call_env.into_arc();
        let result = self.eval(&lf.body);
        self.env = saved;
        result
      },
      Value::BuiltinFunc(mut bf) => {
        bf.applied.push(arg);
        if bf.applied.len() < bf.arity {
          return Ok(Value::BuiltinFunc(bf));
        }
        (bf.func)(&bf.applied, span)
      },
      other => Err(LxError::type_err(format!("cannot call {}, not a function", other.type_name()), span)),
    }
  }

  pub(super) fn eval_pipe(&mut self, left: &SExpr, right: &SExpr, span: Span) -> Result<Value, LxError> {
    let val = self.eval(left)?;
    let func = self.eval(right)?;
    self.apply_func(func, val, span)
  }

  pub(super) fn eval_func(&mut self, params: &[Param], body: &SExpr) -> Result<Value, LxError> {
    let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
    let defaults: Vec<Option<Value>> = params
      .iter()
      .map(|p| {
        p.default
          .as_ref()
          .map(|d| {
            let mut tmp = Interpreter { env: Arc::clone(&self.env), source: self.source.clone() };
            tmp.eval(d)
          })
          .transpose()
      })
      .collect::<Result<_, _>>()?;
    let arity = params.len();
    Ok(Value::Func(LxFunc { params: param_names, defaults, body: Arc::new(body.clone()), closure: self.env.as_ref().clone(), arity, applied: vec![] }))
  }

  fn make_section_func(&self, param: &str, body_expr: Expr, span: Span) -> Value {
    let body = Spanned::new(body_expr, span);
    Value::Func(LxFunc {
      params: vec![param.into()],
      defaults: vec![None],
      body: Arc::new(body),
      closure: self.env.as_ref().clone(),
      arity: 1,
      applied: vec![],
    })
  }

  pub(super) fn eval_section(&mut self, sec: &Section, span: Span) -> Result<Value, LxError> {
    match sec {
      Section::Right { op, operand } => {
        let body = Expr::Binary { op: *op, left: Box::new(Spanned::new(Expr::Ident("_x".into()), span)), right: Box::new((**operand).clone()) };
        Ok(self.make_section_func("_x", body, span))
      },
      Section::Left { operand, op } => {
        let body = Expr::Binary { op: *op, left: Box::new((**operand).clone()), right: Box::new(Spanned::new(Expr::Ident("_x".into()), span)) };
        Ok(self.make_section_func("_x", body, span))
      },
      Section::Field(name) => {
        let body = Expr::FieldAccess { expr: Box::new(Spanned::new(Expr::Ident("_x".into()), span)), field: FieldKind::Named(name.clone()) };
        Ok(self.make_section_func("_x", body, span))
      },
    }
  }

  pub(super) fn eval_compose(&mut self, left: &SExpr, right: &SExpr, span: Span) -> Result<Value, LxError> {
    let _f = self.eval(left)?;
    let _g = self.eval(right)?;
    Err(LxError::runtime("compose not yet implemented", span))
  }

  pub(super) fn eval_field_access(&mut self, expr: &SExpr, field: &FieldKind, span: Span) -> Result<Value, LxError> {
    let val = self.eval(expr)?;
    match field {
      FieldKind::Named(name) => match &val {
        Value::Record(r) => r.get(name).cloned().ok_or_else(|| LxError::runtime(format!("field '{name}' not found"), span)),
        other => Err(LxError::type_err(format!("field access on {}, not Record", other.type_name()), span)),
      },
      FieldKind::Index(idx) => match &val {
        Value::Tuple(items) => {
          let i = if *idx < 0 { items.len() as i64 + idx } else { *idx } as usize;
          items.get(i).cloned().ok_or_else(|| LxError::runtime(format!("tuple index {idx} out of bounds"), span))
        },
        Value::List(items) => {
          let i = if *idx < 0 { items.len() as i64 + idx } else { *idx } as usize;
          items.get(i).cloned().ok_or_else(|| LxError::runtime(format!("list index {idx} out of bounds"), span))
        },
        other => Err(LxError::type_err(format!("index access on {}, not Tuple/List", other.type_name()), span)),
      },
      FieldKind::Computed(key_expr) => {
        let key = self.eval(key_expr)?;
        match (&val, &key) {
          (Value::Record(r), Value::Str(s)) => r.get(s.as_ref()).cloned().ok_or_else(|| LxError::runtime(format!("field '{s}' not found"), span)),
          (Value::List(items), Value::Int(n)) => {
            let i = n.to_i64().ok_or_else(|| LxError::runtime("index too large", span))?;
            let i = if i < 0 { items.len() as i64 + i } else { i } as usize;
            items.get(i).cloned().ok_or_else(|| LxError::runtime("index out of bounds", span))
          },
          _ => Err(LxError::type_err(format!("computed field access: unsupported types {} / {}", val.type_name(), key.type_name()), span)),
        }
      },
    }
  }

  pub(super) fn eval_ternary(&mut self, cond: &SExpr, then_: &SExpr, else_: &Option<Box<SExpr>>, span: Span) -> Result<Value, LxError> {
    let cv = self.eval(cond)?;
    match cv.as_bool() {
      Some(true) => self.eval(then_),
      Some(false) => match else_ {
        Some(e) => self.eval(e),
        None => Ok(Value::Unit),
      },
      _ => Err(LxError::type_err("ternary condition must be Bool", span)),
    }
  }
}
