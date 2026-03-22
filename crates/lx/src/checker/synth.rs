use crate::ast::{
  Expr, ExprApply, ExprAssert, ExprBinary, ExprCoalesce, ExprEmit, ExprFieldAccess, ExprFunc, ExprMatch, ExprNamedArg, ExprPipe, ExprSlice, ExprTernary,
  ExprTimeout, ExprUnary, ExprWith, ExprYield, ListElem, SExpr, Stmt, UnaryOp, WithKind,
};
use crate::sym::intern;

use super::synth_helpers::synth_literal;
use super::types::{Type, TypeContext};
use super::{Checker, DiagLevel};

impl Checker {
  pub(super) fn synth(&mut self, expr: &SExpr) -> Type {
    match &expr.node {
      Expr::Literal(lit) => synth_literal(lit),
      Expr::Ident(name) => self.lookup(*name).unwrap_or(Type::Unknown),
      Expr::TypeConstructor(_) => Type::Unknown,
      Expr::Binary(ExprBinary { op, left, right }) => {
        let lt = self.synth(left);
        let rt = self.synth(right);
        self.synth_binary(op, &lt, &rt, expr.span)
      },
      Expr::Unary(ExprUnary { op: UnaryOp::Neg, operand }) => {
        let t = self.synth(operand);
        match self.table.resolve(&t) {
          Type::Int | Type::Float => t,
          _ => {
            self.emit(DiagLevel::Error, "negation requires Int or Float".into(), expr.span);
            Type::Unknown
          },
        }
      },
      Expr::Unary(ExprUnary { op: UnaryOp::Not, .. }) => Type::Bool,
      Expr::Pipe(ExprPipe { left, right }) => {
        let _ = self.synth(left);
        self.synth(right)
      },
      Expr::Apply(ExprApply { func, arg }) => self.synth_apply(func, arg),
      Expr::Func(ExprFunc { params, ret_type, body, .. }) => self.synth_func(params, ret_type, body),
      Expr::Block(stmts) => self.check_stmts(stmts),
      Expr::Tuple(elems) => Type::Tuple(elems.iter().map(|e| self.synth(e)).collect()),
      Expr::List(elems) => {
        if elems.is_empty() {
          Type::List(Box::new(self.fresh()))
        } else {
          let first = match &elems[0] {
            ListElem::Single(e) => self.synth(e),
            ListElem::Spread(e) => self.synth(e),
          };
          Type::List(Box::new(first))
        }
      },
      Expr::Record(fields) => {
        let fs: Vec<_> = fields.iter().filter_map(|f| f.name.as_ref().map(|n| (*n, self.synth(&f.value)))).collect();
        Type::Record(fs)
      },
      Expr::Propagate(inner) => {
        let t = self.synth(inner);
        match self.table.resolve(&t) {
          Type::Result { ok, .. } => *ok,
          Type::Maybe(inner) => *inner,
          Type::Unknown => Type::Unknown,
          _ => {
            self.emit(DiagLevel::Error, "^ requires Result or Maybe".into(), expr.span);
            Type::Unknown
          },
        }
      },
      Expr::Coalesce(ExprCoalesce { expr: e, default }) => {
        let _ = self.synth(e);
        self.synth(default)
      },
      Expr::Match(ExprMatch { scrutinee, arms }) => self.synth_match(scrutinee, arms, expr.span),
      Expr::Ternary(ExprTernary { cond, then_, else_ }) => {
        let ct = self.synth(cond);
        let resolved = self.table.resolve(&ct);
        if resolved != Type::Bool && resolved != Type::Unknown {
          self.emit(DiagLevel::Error, "ternary condition must be Bool".into(), cond.span);
        }
        let tt = self.synth(then_);
        if let Some(e) = else_ {
          let et = self.synth(e);
          match self.table.unify_with_context(&tt, &et, TypeContext::General) {
            Ok(t) => t,
            Err(te) => {
              self.emit_type_error(&te, e.span);
              Type::Unknown
            },
          }
        } else {
          tt
        }
      },
      Expr::With(ExprWith { kind, body }) => match kind {
        WithKind::Binding { name, value, mutable: _ } => {
          let vt = self.synth(value);
          self.push_scope();
          self.bind(*name, vt);
          let result = self.check_stmts(body);
          self.pop_scope();
          result
        },
        WithKind::Resources { resources } => {
          self.push_scope();
          for (expr, name) in resources {
            let vt = self.synth(expr);
            self.bind(*name, vt);
          }
          let result = self.check_stmts(body);
          self.pop_scope();
          result
        },
        WithKind::Context { fields } => {
          self.push_scope();
          for (_, expr) in fields {
            self.synth(expr);
          }
          self.bind(intern("context"), Type::Unknown);
          let result = self.check_stmts(body);
          self.pop_scope();
          result
        },
      },
      Expr::Yield(ExprYield { .. }) => Type::Unknown,
      Expr::Emit(ExprEmit { value }) => {
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
      Expr::Assert(ExprAssert { expr, msg }) => {
        self.synth(expr);
        if let Some(m) = msg {
          self.synth(m);
        }
        Type::Unit
      },
      Expr::Slice(ExprSlice { expr, start, end }) => {
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
      Expr::FieldAccess(ExprFieldAccess { expr, .. }) => {
        self.synth(expr);
        Type::Unknown
      },
      Expr::NamedArg(ExprNamedArg { value, .. }) => self.synth(value),
      Expr::Par(stmts) => {
        for s in stmts {
          if let Stmt::Expr(e) = &s.node {
            self.check_mutable_captures(e, expr.span);
          }
        }
        let result = self.check_stmts(stmts);
        Type::List(Box::new(result))
      },
      Expr::Timeout(ExprTimeout { ms, body }) => {
        let ms_type = self.synth(ms);
        let resolved = self.table.resolve(&ms_type);
        if resolved != Type::Int && resolved != Type::Float && resolved != Type::Unknown {
          self.emit(DiagLevel::Error, "timeout ms must be Int or Float".into(), ms.span);
        }
        let body_type = self.synth(body);
        let err_fields = vec![(intern("kind"), Type::Str), (intern("ms"), Type::Int)];
        Type::Result { ok: Box::new(body_type), err: Box::new(Type::Record(err_fields)) }
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
