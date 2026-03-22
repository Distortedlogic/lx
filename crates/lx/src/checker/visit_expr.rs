use crate::ast::{AstArena, ExprApply, ExprFunc, ExprMatch, ExprTernary, ExprTimeout, ExprWith, ListElem, Literal, MapEntry, RecordField, SelArm, Stmt, StmtId, WithKind};
use crate::sym::{Sym, intern};
use crate::visitor::{AstVisitor, VisitAction};
use miette::SourceSpan;

use super::diagnostics::DiagnosticKind;
use super::types::Type;
use super::unification::TypeContext;
use super::{Checker, DiagLevel};

use crate::ast::Program;

impl AstVisitor for Checker<'_> {
  fn visit_program<P>(&mut self, program: &Program<P>) -> VisitAction {
    let arena = &program.arena;
    for &sid in &program.stmts {
      self.check_stmt(sid, arena);
    }
    VisitAction::Skip
  }

  fn visit_literal(&mut self, lit: &Literal, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.push_type(Checker::synth_literal_type(lit));
    VisitAction::Skip
  }

  fn visit_ident(&mut self, name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.push_type(self.lookup(name).unwrap_or(Type::Unknown));
    VisitAction::Descend
  }

  fn visit_type_constructor(&mut self, _name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.push_type(Type::Unknown);
    VisitAction::Descend
  }

  fn visit_func(&mut self, func: &ExprFunc, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let params = func.params.clone();
    let ret_type = func.ret_type;
    let body = func.body;
    let ty = self.synth_func_type(&params, &ret_type, body);
    self.push_type(ty);
    VisitAction::Skip
  }

  fn visit_block(&mut self, stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let stmts = stmts.to_vec();
    let ty = self.check_stmts(&stmts);
    self.push_type(ty);
    VisitAction::Skip
  }

  fn visit_match(&mut self, m: &ExprMatch, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let arms = m.arms.clone();
    let ty = self.synth_match_type(m.scrutinee, &arms, span);
    self.push_type(ty);
    VisitAction::Skip
  }

  fn visit_apply(&mut self, apply: &ExprApply, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let ty = self.synth_apply_type(apply.func, apply.arg);
    self.push_type(ty);
    VisitAction::Skip
  }

  fn visit_with(&mut self, with: &ExprWith, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let kind = with.kind.clone();
    let body = with.body.clone();
    let ty = match kind {
      WithKind::Binding { name, value, .. } => {
        let vt = self.synth_expr(value);
        self.push_scope();
        self.bind(name, vt);
        let result = self.check_stmts(&body);
        self.pop_scope();
        result
      },
      WithKind::Resources { resources } => {
        self.push_scope();
        for (expr, name) in &resources {
          let vt = self.synth_expr(*expr);
          self.bind(*name, vt);
        }
        let result = self.check_stmts(&body);
        self.pop_scope();
        result
      },
      WithKind::Context { fields } => {
        self.push_scope();
        for (_, expr) in &fields {
          self.synth_expr(*expr);
        }
        self.bind(intern("context"), Type::Unknown);
        let result = self.check_stmts(&body);
        self.pop_scope();
        result
      },
    };
    self.push_type(ty);
    VisitAction::Skip
  }

  fn visit_map(&mut self, entries: &[MapEntry], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let entries = entries.to_vec();
    let ty = self.synth_map_type(&entries);
    self.push_type(ty);
    VisitAction::Skip
  }

  fn visit_list(&mut self, elems: &[ListElem], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let ty = if elems.is_empty() {
      Type::List(Box::new(self.fresh()))
    } else {
      let first = match &elems[0] {
        ListElem::Single(e) | ListElem::Spread(e) => self.synth_expr(*e),
      };
      for elem in &elems[1..] {
        match elem {
          ListElem::Single(e) | ListElem::Spread(e) => {
            self.synth_expr(*e);
          },
        }
      }
      Type::List(Box::new(first))
    };
    self.push_type(ty);
    VisitAction::Skip
  }

  fn visit_record(&mut self, fields: &[RecordField], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let fields = fields.to_vec();
    let fs: Vec<_> = fields
      .iter()
      .filter_map(|f| match f {
        RecordField::Named { name, value } => Some((*name, self.synth_expr(*value))),
        RecordField::Spread(_) => None,
      })
      .collect();
    self.push_type(Type::Record(fs));
    VisitAction::Skip
  }

  fn visit_loop(&mut self, stmts: &[StmtId], _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let stmts = stmts.to_vec();
    self.check_stmts(&stmts);
    self.push_type(Type::Unit);
    VisitAction::Skip
  }

  fn visit_par(&mut self, stmts: &[StmtId], span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let stmts = stmts.to_vec();
    let arena = self.arena;
    for sid in &stmts {
      if let Stmt::Expr(e) = arena.stmt(*sid) {
        self.check_mutable_captures(*e, span);
      }
    }
    let result = self.check_stmts(&stmts);
    self.push_type(Type::List(Box::new(result)));
    VisitAction::Skip
  }

  fn visit_sel(&mut self, arms: &[SelArm], span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let arms = arms.to_vec();
    for arm in &arms {
      self.check_mutable_captures(arm.expr, span);
      self.synth_expr(arm.expr);
      self.synth_expr(arm.handler);
    }
    self.push_type(Type::Unknown);
    VisitAction::Skip
  }

  fn visit_ternary(&mut self, ternary: &ExprTernary, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let ct = self.synth_expr(ternary.cond);
    let cond_span = self.arena.expr_span(ternary.cond);
    let resolved = self.table.resolve(&ct);
    if resolved != Type::Bool && resolved != Type::Unknown && resolved != Type::Error {
      self.emit(DiagLevel::Error, DiagnosticKind::TernaryCondNotBool, cond_span);
    }
    let tt = self.synth_expr(ternary.then_);
    let ty = if let Some(e) = ternary.else_ {
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
    };
    self.push_type(ty);
    VisitAction::Skip
  }

  fn visit_timeout(&mut self, timeout: &ExprTimeout, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    let ms_span = self.arena.expr_span(timeout.ms);
    let ms_type = self.synth_expr(timeout.ms);
    let resolved = self.table.resolve(&ms_type);
    if resolved != Type::Int && resolved != Type::Float && resolved != Type::Unknown && resolved != Type::Error {
      self.emit(DiagLevel::Error, DiagnosticKind::TimeoutMsNotNumeric, ms_span);
    }
    let body_type = self.synth_expr(timeout.body);
    let err_fields = vec![(intern("kind"), Type::Str), (intern("ms"), Type::Int)];
    self.push_type(Type::Result { ok: Box::new(body_type), err: Box::new(Type::Record(err_fields)) });
    VisitAction::Skip
  }
}
