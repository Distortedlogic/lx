use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::ast::{SExpr, Param, Expr, Spanned, Section, FieldKind};
use crate::value::ProtoFieldDef;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{LxFunc, Value};

use super::Interpreter;

impl Interpreter {
  pub(super) fn apply_func(&mut self, func: Value, arg: Value, span: Span) -> Result<Value, LxError> {
    match func {
      Value::Func(mut lf) => {
        if let Value::Unit = &arg
          && lf.arity == 0 && lf.applied.is_empty() {
            let saved = Arc::clone(&self.env);
            self.env = Arc::clone(&lf.closure);
            let result = self.eval_func_body(&lf.body, lf.returns_result);
            self.env = saved;
            return match result {
              Err(LxError::Propagate { value, .. }) => Ok(*value),
              other => other,
            };
          }
        self.apply_named_args(&mut lf, arg);
        if lf.applied.len() == 1 && lf.arity > 1
          && let Value::Tuple(ref elems) = lf.applied[0]
          && elems.len() == lf.arity {
            let elems = elems.as_ref().clone();
            lf.applied = elems;
          }
        if lf.applied.len() < lf.arity {
          return Ok(Value::Func(lf));
        }
        let saved = Arc::clone(&self.env);
        let mut call_env = lf.closure.child();
        for (i, name) in lf.params.iter().enumerate() {
          if i < lf.applied.len() {
            call_env.bind(name.clone(), lf.applied[i].clone());
          } else if let Some(Some(def)) = lf.defaults.get(i) {
            call_env.bind(name.clone(), def.clone());
          }
        }
        self.env = call_env.into_arc();
        let result = self.eval_func_body(&lf.body, lf.returns_result);
        self.env = saved;
        match result {
          Err(LxError::Propagate { value, .. }) => Ok(*value),
          other => other,
        }
      },
      Value::BuiltinFunc(mut bf) => {
        bf.applied.push(arg);
        if bf.applied.len() < bf.arity {
          return Ok(Value::BuiltinFunc(bf));
        }
        (bf.func)(&bf.applied, span)
      },
      Value::TaggedCtor { tag, arity, mut applied } => {
        applied.push(arg);
        if applied.len() < arity {
          Ok(Value::TaggedCtor { tag, arity, applied })
        } else {
          Ok(Value::Tagged { tag, values: Arc::new(applied) })
        }
      },
      Value::Protocol { name, fields } => self.apply_protocol(&name, &fields, &arg, span),
      other => Err(LxError::type_err(format!("cannot call {}, not a function", other.type_name()), span)),
    }
  }

  pub(super) fn force_defaults(&mut self, val: Value, _span: Span) -> Result<Value, LxError> {
    match val {
      Value::Func(ref lf) if lf.applied.len() < lf.arity
        && (lf.applied.len()..lf.arity).all(|i| matches!(lf.defaults.get(i), Some(Some(_)))) => {
          let Value::Func(lf) = val else { unreachable!() };
          let saved = Arc::clone(&self.env);
          let mut call_env = lf.closure.child();
          for (i, name) in lf.params.iter().enumerate() {
            if i < lf.applied.len() {
              call_env.bind(name.clone(), lf.applied[i].clone());
            } else if let Some(Some(def)) = lf.defaults.get(i) {
              call_env.bind(name.clone(), def.clone());
            }
          }
          self.env = call_env.into_arc();
          let result = self.eval_func_body(&lf.body, lf.returns_result);
          self.env = saved;
          match result {
            Err(LxError::Propagate { value, .. }) => Ok(*value),
            other => other,
          }
        },
      other => Ok(other),
    }
  }

  pub(super) fn eval_pipe(&mut self, left: &SExpr, right: &SExpr, span: Span) -> Result<Value, LxError> {
    let val = self.eval(left)?;
    let val = self.force_defaults(val, span)?;
    let func = self.eval(right)?;
    self.apply_func(func, val, span)
  }

  fn apply_named_args(&self, lf: &mut LxFunc, arg: Value) {
    if let Value::Tagged { ref tag, ref values } = arg
      && tag.as_ref() == "__named"
      && values.len() == 2
      && let Value::Str(ref name) = values[0]
    {
      if let Some(idx) = lf.params.iter().position(|p| p == name.as_ref()) {
        while lf.applied.len() < idx {
          lf.applied.push(Value::Unit);
        }
        if lf.applied.len() == idx {
          lf.applied.push(values[1].clone());
        } else {
          lf.applied[idx] = values[1].clone();
        }
      } else {
        lf.applied.push(arg);
      }
    } else {
      lf.applied.push(arg);
    }
  }

  fn eval_func_body(&mut self, body: &SExpr, returns_result: bool) -> Result<Value, LxError> {
    if returns_result {
      if let Expr::Block(stmts) = &body.node {
        let saved = Arc::clone(&self.env);
        self.env = Arc::new(self.env.child());
        let mut result = Value::Unit;
        for stmt in stmts {
          result = self.eval_stmt_checking_err(stmt)?;
          if matches!(&result, Value::Err(_)) {
            self.env = saved;
            return Ok(result);
          }
        }
        self.env = saved;
        if matches!(&result, Value::Ok(_) | Value::Err(_)) {
          Ok(result)
        } else {
          Ok(Value::Ok(Box::new(result)))
        }
      } else {
        let result = self.eval(body)?;
        if matches!(&result, Value::Ok(_) | Value::Err(_)) {
          Ok(result)
        } else {
          Ok(Value::Ok(Box::new(result)))
        }
      }
    } else {
      self.eval(body)
    }
  }

  fn eval_stmt_checking_err(&mut self, stmt: &crate::ast::SStmt) -> Result<Value, LxError> {
    let val = self.eval_stmt(stmt)?;
    Ok(val)
  }

  pub(super) fn eval_func(&mut self, params: &[Param], body: &SExpr, returns_result: bool) -> Result<Value, LxError> {
    let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
    let defaults: Vec<Option<Value>> = params
      .iter()
      .map(|p| {
        p.default
          .as_ref()
          .map(|d| {
            let mut tmp = Interpreter {
              env: Arc::clone(&self.env),
              source: self.source.clone(),
              source_dir: self.source_dir.clone(),
              module_cache: Arc::clone(&self.module_cache),
              loading: Arc::clone(&self.loading),
            };
            tmp.eval(d)
          })
          .transpose()
      })
      .collect::<Result<_, _>>()?;
    let arity = params.len();
    Ok(Value::Func(LxFunc { params: param_names, defaults, body: Arc::new(body.clone()), closure: Arc::clone(&self.env), arity, applied: vec![], returns_result }))
  }

  fn make_section_func(&self, param: &str, body_expr: Expr, span: Span) -> Value {
    let body = Spanned::new(body_expr, span);
    Value::Func(LxFunc {
      params: vec![param.into()],
      defaults: vec![None],
      body: Arc::new(body),
      closure: Arc::clone(&self.env),
      arity: 1,
      applied: vec![],
      returns_result: false,
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
      Section::Index(idx) => {
        let body = Expr::FieldAccess { expr: Box::new(Spanned::new(Expr::Ident("_x".into()), span)), field: FieldKind::Index(*idx) };
        Ok(self.make_section_func("_x", body, span))
      },
      Section::BinOp(op) => {
        let body = Expr::Binary {
          op: *op,
          left: Box::new(Spanned::new(Expr::Ident("_a".into()), span)),
          right: Box::new(Spanned::new(Expr::Ident("_b".into()), span)),
        };
        Ok(self.make_section_func_2("_a", "_b", body, span))
      },
    }
  }

  fn make_section_func_2(&self, p1: &str, p2: &str, body_expr: Expr, span: Span) -> Value {
    let body = Spanned::new(body_expr, span);
    Value::Func(LxFunc {
      params: vec![p1.into(), p2.into()],
      defaults: vec![None, None],
      body: Arc::new(body),
      closure: Arc::clone(&self.env),
      arity: 2,
      applied: vec![],
      returns_result: false,
    })
  }

  pub(super) fn eval_compose(&mut self, left: &SExpr, right: &SExpr, span: Span) -> Result<Value, LxError> {
    let f = self.eval(left)?;
    let g = self.eval(right)?;
    let body = Expr::Pipe {
      left: Box::new(Spanned::new(
        Expr::Apply { func: Box::new(Spanned::new(Expr::Ident("_cf".into()), span)), arg: Box::new(Spanned::new(Expr::Ident("_cx".into()), span)) },
        span,
      )),
      right: Box::new(Spanned::new(Expr::Ident("_cg".into()), span)),
    };
    let mut closure = self.env.child();
    closure.bind("_cf".into(), f);
    closure.bind("_cg".into(), g);
    Ok(Value::Func(LxFunc {
      params: vec!["_cx".into()],
      defaults: vec![None],
      body: Arc::new(Spanned::new(body, span)),
      closure: closure.into_arc(),
      arity: 1,
      applied: vec![],
      returns_result: false,
    }))
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
          (Value::Map(m), Value::Str(s)) => {
            let vk = crate::value::ValueKey(Value::Str(s.clone()));
            m.get(&vk).cloned().ok_or_else(|| LxError::runtime(format!("key '{s}' not found"), span))
          },
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

  fn apply_protocol(&mut self, name: &str, fields: &Arc<Vec<ProtoFieldDef>>, arg: &Value, span: Span) -> Result<Value, LxError> {
    let Value::Record(rec) = arg else {
      return Err(LxError::runtime(
        format!("Protocol {name}: expected Record, got {}", arg.type_name()),
        span,
      ));
    };
    let mut result = rec.as_ref().clone();
    for field in fields.iter() {
      match rec.get(&field.name) {
        Some(val) => {
          if field.type_name != "Any" && val.type_name() != field.type_name {
            return Err(LxError::runtime(
              format!("Protocol {name}: field '{}' expected {}, got {}", field.name, field.type_name, val.type_name()),
              span,
            ));
          }
        },
        None => {
          if let Some(ref default) = field.default {
            result.insert(field.name.clone(), default.clone());
          } else {
            return Err(LxError::runtime(
              format!("Protocol {name}: missing required field '{}'", field.name),
              span,
            ));
          }
        },
      }
    }
    Ok(Value::Record(Arc::new(result)))
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
