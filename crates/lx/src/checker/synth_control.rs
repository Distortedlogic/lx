use crate::ast::{ExprId, SelArm, Stmt, StmtId, WithKind};
use crate::sym::intern;
use miette::SourceSpan;

use super::diagnostics::DiagnosticKind;
use super::semantic::{DefKind, ScopeKind};
use super::type_arena::TypeId;
use super::types::Type;
use super::unification::TypeContext;
use super::{Checker, DiagLevel};

impl Checker<'_> {
  pub(super) fn synth_with_type(&mut self, kind: &WithKind, body: &[StmtId]) -> TypeId {
    match kind {
      WithKind::Binding { name, value, .. } => {
        let vt = self.synth_expr(*value);
        let vspan = self.arena.expr_span(*value);
        self.sem.push_scope(ScopeKind::With, vspan);
        let def_id = self.sem.add_definition(*name, DefKind::WithBinding, vspan, false);
        self.sem.set_definition_type(def_id, vt);
        let result = self.check_stmts(body);
        self.sem.pop_scope();
        result
      },
      WithKind::Resources { resources } => {
        let rspan = resources.first().map(|(e, _)| self.arena.expr_span(*e)).unwrap_or((0, 0).into());
        self.sem.push_scope(ScopeKind::With, rspan);
        for (expr, name) in resources {
          let vt = self.synth_expr(*expr);
          let espan = self.arena.expr_span(*expr);
          let def_id = self.sem.add_definition(*name, DefKind::ResourceBinding, espan, false);
          self.sem.set_definition_type(def_id, vt);
        }
        let result = self.check_stmts(body);
        self.sem.pop_scope();
        result
      },
      WithKind::Context { fields } => {
        let fspan = fields.first().map(|f| self.arena.expr_span(f.1)).unwrap_or((0, 0).into());
        self.sem.push_scope(ScopeKind::With, fspan);
        for (_, expr) in fields {
          self.synth_expr(*expr);
        }
        let unknown = self.type_arena.unknown();
        let def_id = self.sem.add_definition(intern("context"), DefKind::WithBinding, fspan, false);
        self.sem.set_definition_type(def_id, unknown);
        let result = self.check_stmts(body);
        self.sem.pop_scope();
        result
      },
    }
  }

  pub(super) fn synth_ternary_type(&mut self, cond: ExprId, then_: ExprId, else_: Option<ExprId>) -> TypeId {
    let ct = self.synth_expr(cond);
    let cond_span = self.arena.expr_span(cond);
    let resolved = self.table.resolve(ct, &self.type_arena);
    let bool_id = self.type_arena.bool();
    let unknown_id = self.type_arena.unknown();
    let todo_id = self.type_arena.todo();
    let error_id = self.type_arena.error();
    if resolved != bool_id && resolved != unknown_id && resolved != todo_id && resolved != error_id {
      self.emit(DiagLevel::Error, DiagnosticKind::TernaryCondNotBool, cond_span);
    }
    let branch_info = super::narrowing::analyze_condition(cond, self.arena, &self.type_arena);
    self.narrowing.push();
    for (name, ty) in &branch_info.then_narrowings {
      self.narrowing.narrow(*name, *ty);
    }
    let tt = self.synth_expr(then_);
    self.narrowing.pop();
    if let Some(e) = else_ {
      self.narrowing.push();
      for (name, ty) in &branch_info.else_narrowings {
        self.narrowing.narrow(*name, *ty);
      }
      let else_span = self.arena.expr_span(e);
      let et = self.synth_expr(e);
      self.narrowing.pop();
      match self.table.unify_with_context(tt, et, TypeContext::General, &mut self.type_arena) {
        Ok(t) => t,
        Err(te) => {
          self.emit_type_error(&te, else_span);
          self.type_arena.error()
        },
      }
    } else {
      tt
    }
  }

  pub(super) fn synth_par_type(&mut self, stmts: &[StmtId], span: SourceSpan) -> TypeId {
    let arena = self.arena;
    for sid in stmts {
      if let Stmt::Expr(e) = arena.stmt(*sid) {
        self.check_mutable_captures(*e, span);
      }
    }
    let result = self.check_stmts(stmts);
    self.type_arena.alloc(Type::List(result))
  }

  pub(super) fn synth_sel_type(&mut self, arms: &[SelArm], span: SourceSpan) -> TypeId {
    for arm in arms {
      self.check_mutable_captures(arm.expr, span);
      self.synth_expr(arm.expr);
      self.synth_expr(arm.handler);
    }
    self.type_arena.todo()
  }

  pub(super) fn synth_timeout_type(&mut self, ms: ExprId, body: ExprId) -> TypeId {
    let ms_span = self.arena.expr_span(ms);
    let ms_type = self.synth_expr(ms);
    let resolved = self.table.resolve(ms_type, &self.type_arena);
    let int_id = self.type_arena.int();
    let float_id = self.type_arena.float();
    let unknown_id = self.type_arena.unknown();
    let todo_id = self.type_arena.todo();
    let error_id = self.type_arena.error();
    if resolved != int_id && resolved != float_id && resolved != unknown_id && resolved != todo_id && resolved != error_id {
      self.emit(DiagLevel::Error, DiagnosticKind::TimeoutMsNotNumeric, ms_span);
    }
    let body_type = self.synth_expr(body);
    let str_id = self.type_arena.str();
    let err_fields = vec![(intern("kind"), str_id), (intern("ms"), int_id)];
    let err = self.type_arena.alloc(Type::Record(err_fields));
    self.type_arena.alloc(Type::Result { ok: body_type, err })
  }
}
