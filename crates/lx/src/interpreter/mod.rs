mod apply;
mod collections;
mod patterns;

use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::ast::*;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub struct Interpreter {
  env: Arc<Env>,
  source: String,
}

impl Interpreter {
  pub fn new(source: &str) -> Self {
    let mut env = Env::new();
    crate::builtins::register(&mut env);
    Self { env: env.into_arc(), source: source.to_string() }
  }

  pub fn exec(&mut self, program: &Program) -> Result<Value, LxError> {
    let mut result = Value::Unit;
    for stmt in &program.stmts {
      result = self.eval_stmt(stmt)?;
    }
    Ok(result)
  }

  fn eval(&mut self, expr: &SExpr) -> Result<Value, LxError> {
    let span = expr.span;
    match &expr.node {
      Expr::Literal(lit) => self.eval_literal(lit, span),
      Expr::Ident(name) => self.env.get(name).ok_or_else(|| LxError::runtime(format!("undefined variable '{name}'"), span)),
      Expr::TypeConstructor(name) => self.env.get(name).ok_or_else(|| LxError::runtime(format!("undefined constructor '{name}'"), span)),
      Expr::Binary { op, left, right } => self.eval_binary(op, left, right, span),
      Expr::Unary { op, operand } => self.eval_unary(op, operand, span),
      Expr::Pipe { left, right } => self.eval_pipe(left, right, span),
      Expr::Apply { func, arg } => {
        let f = self.eval(func)?;
        let a = self.eval(arg)?;
        self.apply_func(f, a, span)
      },
      Expr::Section(sec) => self.eval_section(sec, span),
      Expr::Compose { left, right } => self.eval_compose(left, right, span),
      Expr::FieldAccess { expr: e, field } => self.eval_field_access(e, field, span),
      Expr::Block(stmts) => self.eval_block(stmts),
      Expr::Tuple(elems) => self.eval_tuple(elems),
      Expr::List(elems) => self.eval_list(elems),
      Expr::Record(fields) => self.eval_record(fields),
      Expr::Map(entries) => self.eval_map(entries),
      Expr::Set(elems) => self.eval_set(elems),
      Expr::Func { params, body } => self.eval_func(params, body),
      Expr::Match { scrutinee, arms } => self.eval_match(scrutinee, arms, span),
      Expr::Ternary { cond, then_, else_ } => self.eval_ternary(cond, then_, else_, span),
      Expr::Assert { expr: e, msg } => self.eval_assert(e, msg, span),
      Expr::Propagate(_) => Err(LxError::runtime("propagate not yet implemented", span)),
      Expr::Coalesce { expr: e, default } => {
        let v = self.eval(e)?;
        if v.is_truthy_err() { self.eval(default) } else { Ok(v) }
      },
      Expr::Loop(_) => Err(LxError::runtime("loop not yet implemented", span)),
      Expr::Break(_) => Err(LxError::runtime("break outside loop", span)),
    }
  }

  fn eval_stmt(&mut self, stmt: &SStmt) -> Result<Value, LxError> {
    match &stmt.node {
      Stmt::Binding(b) => {
        let val = self.eval(&b.value)?;
        match &b.target {
          BindTarget::Name(name) => {
            let mut env = self.env.child();
            if b.mutable {
              env.bind_mut(name.clone(), val);
            } else {
              env.bind(name.clone(), val);
            }
            self.env = env.into_arc();
          },
          BindTarget::Reassign(name) => {
            self.env.reassign(name, val).map_err(|e| LxError::runtime(e, stmt.span))?;
          },
          BindTarget::Pattern(pat) => {
            let bindings = self.try_match_pattern(&pat.node, &val).ok_or_else(|| LxError::runtime("pattern match failed in binding", stmt.span))?;
            let mut env = self.env.child();
            for (name, v) in bindings {
              if b.mutable {
                env.bind_mut(name, v);
              } else {
                env.bind(name, v);
              }
            }
            self.env = env.into_arc();
          },
        }
        Ok(Value::Unit)
      },
      Stmt::Expr(e) => self.eval(e),
    }
  }

  fn eval_literal(&mut self, lit: &Literal, span: Span) -> Result<Value, LxError> {
    match lit {
      Literal::Int(n) => Ok(Value::Int(n.clone())),
      Literal::Float(f) => Ok(Value::Float(*f)),
      Literal::Bool(b) => Ok(Value::Bool(*b)),
      Literal::Str(parts) => self.eval_string_parts(parts),
      Literal::RawStr(s) => Ok(Value::Str(Arc::from(s.as_str()))),
      Literal::Unit => {
        let _ = span;
        Ok(Value::Unit)
      },
    }
  }

  fn eval_binary(&mut self, op: &BinOp, left: &SExpr, right: &SExpr, span: Span) -> Result<Value, LxError> {
    if *op == BinOp::And {
      let l = self.eval(left)?;
      return match l.as_bool() {
        Some(false) => Ok(Value::Bool(false)),
        Some(true) => self.eval(right),
        _ => Err(LxError::type_err("&& requires Bool operands", span)),
      };
    }
    if *op == BinOp::Or {
      let l = self.eval(left)?;
      return match l.as_bool() {
        Some(true) => Ok(Value::Bool(true)),
        Some(false) => self.eval(right),
        _ => Err(LxError::type_err("|| requires Bool operands", span)),
      };
    }
    let lv = self.eval(left)?;
    let rv = self.eval(right)?;
    self.binary_op(op, &lv, &rv, span)
  }

  fn binary_op(&self, op: &BinOp, lv: &Value, rv: &Value, span: Span) -> Result<Value, LxError> {
    match op {
      BinOp::Eq => return Ok(Value::Bool(lv == rv)),
      BinOp::NotEq => return Ok(Value::Bool(lv != rv)),
      _ => {},
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
      },
      (BinOp::IntDiv, Value::Int(a), Value::Int(b)) => {
        if b.sign() == num_bigint::Sign::NoSign {
          return Err(LxError::division_by_zero(span));
        }
        Ok(Value::Int(a / b))
      },
      (BinOp::Mod, Value::Int(a), Value::Int(b)) => {
        if b.sign() == num_bigint::Sign::NoSign {
          return Err(LxError::division_by_zero(span));
        }
        Ok(Value::Int(a % b))
      },
      (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
      (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
      (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
      (BinOp::Div, Value::Float(_), Value::Float(b)) if *b == 0.0 => Err(LxError::division_by_zero(span)),
      (BinOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
      (BinOp::IntDiv, Value::Float(_), Value::Float(b)) if *b == 0.0 => Err(LxError::division_by_zero(span)),
      (BinOp::IntDiv, Value::Float(a), Value::Float(b)) => Ok(Value::Float((a / b).floor())),
      (BinOp::Mod, Value::Float(_), Value::Float(b)) if *b == 0.0 => Err(LxError::division_by_zero(span)),
      (BinOp::Mod, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
      (op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::IntDiv | BinOp::Mod), Value::Int(a), Value::Float(b)) => {
        let af = a.to_f64().ok_or_else(|| LxError::runtime("int too large for float", span))?;
        self.binary_op(op, &Value::Float(af), &Value::Float(*b), span)
      },
      (op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::IntDiv | BinOp::Mod), Value::Float(a), Value::Int(b)) => {
        let bf = b.to_f64().ok_or_else(|| LxError::runtime("int too large for float", span))?;
        self.binary_op(op, &Value::Float(*a), &Value::Float(bf), span)
      },
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
      },
      (BinOp::Concat, Value::List(a), Value::List(b)) => {
        let mut v = a.as_ref().clone();
        v.extend(b.as_ref().iter().cloned());
        Ok(Value::List(Arc::new(v)))
      },
      _ => Err(LxError::type_err(format!("cannot apply '{op}' to {} and {}", lv.type_name(), rv.type_name()), span)),
    }
  }

  fn eval_unary(&mut self, op: &UnaryOp, operand: &SExpr, span: Span) -> Result<Value, LxError> {
    let v = self.eval(operand)?;
    match (op, &v) {
      (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
      (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
      (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
      _ => Err(LxError::type_err(format!("cannot apply '{op}' to {}", v.type_name()), span)),
    }
  }

  fn eval_string_parts(&mut self, parts: &[StrPart]) -> Result<Value, LxError> {
    let mut buf = String::new();
    for part in parts {
      match part {
        StrPart::Text(t) => buf.push_str(t),
        StrPart::Interp(e) => {
          let v = self.eval(e)?;
          buf.push_str(&format!("{v}"));
        },
      }
    }
    Ok(Value::Str(Arc::from(buf)))
  }

  fn eval_block(&mut self, stmts: &[SStmt]) -> Result<Value, LxError> {
    let saved = Arc::clone(&self.env);
    self.env = Arc::new(self.env.child());
    let mut result = Value::Unit;
    for stmt in stmts {
      result = self.eval_stmt(stmt)?;
    }
    self.env = saved;
    Ok(result)
  }

  fn eval_assert(&mut self, expr: &SExpr, msg: &Option<Box<SExpr>>, span: Span) -> Result<Value, LxError> {
    let val = self.eval(expr)?;
    match val.as_bool() {
      Some(true) => Ok(Value::Unit),
      Some(false) => {
        let message = match msg {
          Some(m) => {
            let mv = self.eval(m)?;
            Some(format!("{mv}"))
          },
          None => None,
        };
        Err(LxError::assert_fail(format!("{:?}", expr.node), message, span))
      },
      _ => Err(LxError::type_err("assert requires Bool", span)),
    }
  }
}
