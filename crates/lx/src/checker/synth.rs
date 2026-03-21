use crate::ast::{Expr, SExpr};

use super::Checker;
use super::synth_helpers::synth_literal;
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
          _ => {
            self.emit("negation requires Int or Float".into(), expr.span);
            Type::Unknown
          },
        }
      },
      Expr::Unary { op: crate::ast::UnaryOp::Not, .. } => Type::Bool,
      Expr::Pipe { left, right } => {
        let _ = self.synth(left);
        self.synth(right)
      },
      Expr::Apply { func, arg } => self.synth_apply(func, arg),
      Expr::Func { params, ret_type, body } => self.synth_func(params, ret_type, body),
      Expr::Block(stmts) => self.check_stmts(stmts),
      Expr::Tuple(elems) => Type::Tuple(elems.iter().map(|e| self.synth(e)).collect()),
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
        let fs: Vec<_> = fields.iter().filter_map(|f| f.name.as_ref().map(|n| (n.clone(), self.synth(&f.value)))).collect();
        Type::Record(fs)
      },
      Expr::Propagate(inner) => {
        let t = self.synth(inner);
        match self.table.resolve(&t) {
          Type::Result { ok, .. } => *ok,
          Type::Maybe(inner) => *inner,
          Type::Unknown => Type::Unknown,
          _ => {
            self.emit("^ requires Result or Maybe".into(), expr.span);
            Type::Unknown
          },
        }
      },
      Expr::Coalesce { expr: e, default } => {
        let _ = self.synth(e);
        self.synth(default)
      },
      Expr::Match { scrutinee, arms } => self.synth_match(scrutinee, arms, expr.span),
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
      Expr::WithResource { resources, body } => {
        self.push_scope();
        for (expr, name) in resources {
          let vt = self.synth(expr);
          self.bind(name.clone(), vt);
        }
        let result = self.check_stmts(body);
        self.pop_scope();
        result
      },
      Expr::WithContext { fields, body } => {
        self.push_scope();
        for (_, expr) in fields {
          self.synth(expr);
        }
        self.bind("context".into(), Type::Unknown);
        let result = self.check_stmts(body);
        self.pop_scope();
        result
      },
      Expr::Yield { .. } => Type::Unknown,
      Expr::Emit { value } => {
        self.synth(value);
        Type::Unit
      },
      Expr::Loop(stmts) => {
        self.check_stmts(stmts);
        Type::Unit
      },
      Expr::Break(val) => {
        if let Some(v) = val {
          self.synth(v);
        }
        Type::Unit
      },
      Expr::Assert { expr, msg } => {
        self.synth(expr);
        if let Some(m) = msg {
          self.synth(m);
        }
        Type::Unit
      },
      Expr::Slice { expr, start, end } => {
        let t = self.synth(expr);
        if let Some(s) = start {
          self.synth(s);
        }
        if let Some(e) = end {
          self.synth(e);
        }
        t
      },
      Expr::Section(_) => Type::Func { param: Box::new(Type::Unknown), ret: Box::new(Type::Unknown) },
      Expr::Map(entries) => self.synth_map(entries),
      Expr::FieldAccess { expr, .. } => {
        self.synth(expr);
        Type::Unknown
      },
      Expr::NamedArg { value, .. } => self.synth(value),
      Expr::Shell { .. } => Type::Result { ok: Box::new(Type::Str), err: Box::new(Type::Str) },
      Expr::Par(stmts) => {
        self.check_mutable_captures_stmts(stmts, expr.span);
        let result = self.check_stmts(stmts);
        Type::List(Box::new(result))
      },
      Expr::Sel(arms) => {
        for arm in arms {
          self.check_mutable_captures(&arm.expr, expr.span);
          self.synth(&arm.expr);
          self.synth(&arm.handler);
        }
        Type::Unknown
      },
    }
  }
}
