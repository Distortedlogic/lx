use crate::ast::{ExprId, SelArm, Stmt, StmtId, WithKind};
use crate::sym::intern;
use miette::SourceSpan;

use super::diagnostics::DiagnosticKind;
use super::types::Type;
use super::unification::TypeContext;
use super::{Checker, DiagLevel};

impl Checker<'_> {
  pub(super) fn synth_with_type(&mut self, kind: &WithKind, body: &[StmtId]) -> Type {
    match kind {
      WithKind::Binding { name, value, .. } => {
        let vt = self.synth_expr(*value);
        self.push_scope();
        self.bind(*name, vt);
        let result = self.check_stmts(body);
        self.pop_scope();
        result
      },
      WithKind::Resources { resources } => {
        self.push_scope();
        for (expr, name) in resources {
          let vt = self.synth_expr(*expr);
          self.bind(*name, vt);
        }
        let result = self.check_stmts(body);
        self.pop_scope();
        result
      },
      WithKind::Context { fields } => {
        self.push_scope();
        for (_, expr) in fields {
          self.synth_expr(*expr);
        }
        self.bind(intern("context"), Type::Unknown);
        let result = self.check_stmts(body);
        self.pop_scope();
        result
      },
    }
  }

  pub(super) fn synth_ternary_type(&mut self, cond: ExprId, then_: ExprId, else_: Option<ExprId>) -> Type {
    let ct = self.synth_expr(cond);
    let cond_span = self.arena.expr_span(cond);
    let resolved = self.table.resolve(&ct);
    if resolved != Type::Bool && resolved != Type::Unknown && resolved != Type::Error {
      self.emit(DiagLevel::Error, DiagnosticKind::TernaryCondNotBool, cond_span);
    }
    let tt = self.synth_expr(then_);
    if let Some(e) = else_ {
      let else_span = self.arena.expr_span(e);
      let et = self.synth_expr(e);
      match self.table.unify_with_context(&tt, &et, TypeContext::General) {
        Ok(t) => t,
        Err(te) => {
          self.emit_type_error(&te, else_span);
          Type::Error
        },
      }
    } else {
      tt
    }
  }

  pub(super) fn synth_par_type(&mut self, stmts: &[StmtId], span: SourceSpan) -> Type {
    let arena = self.arena;
    for sid in stmts {
      if let Stmt::Expr(e) = arena.stmt(*sid) {
        self.check_mutable_captures(*e, span);
      }
    }
    let result = self.check_stmts(stmts);
    Type::List(Box::new(result))
  }

  pub(super) fn synth_sel_type(&mut self, arms: &[SelArm], span: SourceSpan) -> Type {
    for arm in arms {
      self.check_mutable_captures(arm.expr, span);
      self.synth_expr(arm.expr);
      self.synth_expr(arm.handler);
    }
    Type::Unknown
  }

  pub(super) fn synth_timeout_type(&mut self, ms: ExprId, body: ExprId) -> Type {
    let ms_span = self.arena.expr_span(ms);
    let ms_type = self.synth_expr(ms);
    let resolved = self.table.resolve(&ms_type);
    if resolved != Type::Int && resolved != Type::Float && resolved != Type::Unknown && resolved != Type::Error {
      self.emit(DiagLevel::Error, DiagnosticKind::TimeoutMsNotNumeric, ms_span);
    }
    let body_type = self.synth_expr(body);
    let err_fields = vec![(intern("kind"), Type::Str), (intern("ms"), Type::Int)];
    Type::Result { ok: Box::new(body_type), err: Box::new(Type::Record(err_fields)) }
  }
}
