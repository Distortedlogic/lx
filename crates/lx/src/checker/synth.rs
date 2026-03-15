use crate::ast::{BinOp, Expr, Literal, Param, SExpr, SType};

use super::Checker;
use super::types::Type;

impl Checker {
  pub(super) fn synth(&mut self, expr: &SExpr) -> Type {
    match &expr.node {
      Expr::Literal(lit) => synth_literal(lit),
      Expr::Ident(name) => self.lookup(name).unwrap_or(Type::Unknown),
      Expr::TypeConstructor(_) => Type::Unknown,
      Expr::Binary { op, left, right } => {
        let lt = self.synth(left);
        let rt = self.synth(right);
        self.synth_binary(op, &lt, &rt, expr.span)
      },
      Expr::Unary { op: crate::ast::UnaryOp::Neg, operand } => {
        let t = self.synth(operand);
        match self.table.resolve(&t) {
          Type::Int | Type::Float => t,
          _ => { self.emit("negation requires Int or Float".into(), expr.span); Type::Unknown },
        }
      },
      Expr::Unary { op: crate::ast::UnaryOp::Not, .. } => Type::Bool,
      Expr::Pipe { left, right } => {
        let _arg_t = self.synth(left);
        self.synth(right)
      },
      Expr::Apply { func, arg } => {
        let ft = self.synth(func);
        let _at = self.synth(arg);
        match self.table.resolve(&ft) {
          Type::Func { ret, .. } => *ret,
          _ => Type::Unknown,
        }
      },
      Expr::Func { params, ret_type, body } => self.synth_func(params, ret_type, body),
      Expr::Block(stmts) => self.check_stmts(stmts),
      Expr::Tuple(elems) => {
        Type::Tuple(elems.iter().map(|e| self.synth(e)).collect())
      },
      Expr::List(elems) => {
        if elems.is_empty() {
          Type::List(Box::new(self.fresh()))
        } else {
          let first = match &elems[0] {
            crate::ast::ListElem::Single(e) => self.synth(e),
            crate::ast::ListElem::Spread(e) => self.synth(e),
          };
          Type::List(Box::new(first))
        }
      },
      Expr::Record(fields) => {
        let fs: Vec<_> = fields.iter().filter_map(|f| {
          f.name.as_ref().map(|n| (n.clone(), self.synth(&f.value)))
        }).collect();
        Type::Record(fs)
      },
      Expr::Propagate(inner) => {
        let t = self.synth(inner);
        match self.table.resolve(&t) {
          Type::Result { ok, .. } => *ok,
          Type::Maybe(inner) => *inner,
          _ => { self.emit("^ requires Result or Maybe".into(), expr.span); Type::Unknown },
        }
      },
      Expr::Coalesce { expr: e, default } => {
        let _et = self.synth(e);
        self.synth(default)
      },
      Expr::Match { scrutinee, arms } => {
        let _st = self.synth(scrutinee);
        let result = self.fresh();
        for arm in arms {
          let body_t = self.synth(&arm.body);
          let _ = self.table.unify(&result, &body_t);
        }
        self.table.resolve(&result)
      },
      Expr::Ternary { cond, then_, else_ } => {
        let ct = self.synth(cond);
        let resolved = self.table.resolve(&ct);
        if resolved != Type::Bool && resolved != Type::Unknown {
          self.emit("ternary condition must be Bool".into(), cond.span);
        }
        let tt = self.synth(then_);
        if let Some(e) = else_ {
          let et = self.synth(e);
          self.table.unify(&tt, &et).unwrap_or(Type::Unknown)
        } else {
          tt
        }
      },
      Expr::With { value, body, name, mutable: _ } => {
        let vt = self.synth(value);
        self.push_scope();
        self.bind(name.clone(), vt);
        let result = self.check_stmts(body);
        self.pop_scope();
        result
      },
      _ => Type::Unknown,
    }
  }

  pub(super) fn synth_func(&mut self, params: &[Param], ret_type: &Option<SType>, body: &SExpr) -> Type {
    self.push_scope();
    let mut param_types = Vec::new();
    for p in params {
      let ty = match &p.type_ann {
        Some(ann) => self.resolve_type_ann(ann),
        None => self.fresh(),
      };
      self.bind(p.name.clone(), ty.clone());
      param_types.push(ty);
    }
    let body_type = self.synth(body);
    if let Some(ret_ann) = ret_type {
      let expected = self.resolve_type_ann(ret_ann);
      if let Err(msg) = self.table.unify(&expected, &body_type) {
        self.emit(format!("return type mismatch: {msg}"), body.span);
      }
    }
    self.pop_scope();
    let mut result = match ret_type {
      Some(ann) => self.resolve_type_ann(ann),
      None => body_type,
    };
    for pt in param_types.into_iter().rev() {
      result = Type::Func { param: Box::new(pt), ret: Box::new(result) };
    }
    result
  }

  fn synth_binary(&mut self, op: &BinOp, lt: &Type, rt: &Type, span: crate::span::Span) -> Type {
    let lt = self.table.resolve(lt);
    let rt = self.table.resolve(rt);
    match op {
      BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::IntDiv => {
        self.table.unify(&lt, &rt).unwrap_or(Type::Unknown)
      },
      BinOp::Concat => Type::Str,
      BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => Type::Bool,
      BinOp::And | BinOp::Or => {
        if lt != Type::Bool && lt != Type::Unknown {
          self.emit("logical operator requires Bool".into(), span);
        }
        Type::Bool
      },
      BinOp::Range | BinOp::RangeInclusive => Type::List(Box::new(Type::Int)),
    }
  }
}

fn synth_literal(lit: &Literal) -> Type {
  match lit {
    Literal::Int(_) => Type::Int,
    Literal::Float(_) => Type::Float,
    Literal::Str(_) | Literal::RawStr(_) => Type::Str,
    Literal::Bool(_) => Type::Bool,
    Literal::Unit => Type::Unit,
  }
}
