use crate::ast::{Expr, ExprFunc, ExprId, ListElem, MatchArm, Stmt, StmtId};

use super::Checker;
use super::types::Type;
use super::unification::TypeContext;

impl Checker<'_> {
  pub(super) fn check_expr(&mut self, eid: ExprId, expected: &Type) -> Type {
    let arena = self.arena;
    let expr = arena.expr(eid).clone();
    match &expr {
      Expr::Func(func) => {
        if let Type::Func { params: exp_params, ret: exp_ret } = self.table.resolve(expected) {
          self.check_func_against(func, &exp_params, &exp_ret)
        } else {
          self.synth_expr(eid)
        }
      },
      Expr::List(elems) => {
        if let Type::List(exp_elem) = self.table.resolve(expected) {
          self.check_list(elems, &exp_elem)
        } else {
          self.synth_expr(eid)
        }
      },
      Expr::Match(m) => {
        let arms = m.arms.clone();
        self.check_match(m.scrutinee, &arms, expected)
      },
      Expr::Block(stmts) => {
        let stmts = stmts.clone();
        self.check_block(&stmts, expected)
      },
      _ => {
        let actual = self.synth_expr(eid);
        let span = arena.expr_span(eid);
        match self.table.unify_with_context(expected, &actual, TypeContext::General) {
          Ok(t) => t,
          Err(te) => {
            self.emit_type_error(&te, span);
            Type::Error
          },
        }
      },
    }
  }

  fn check_func_against(&mut self, func: &ExprFunc, exp_params: &[Type], exp_ret: &Type) -> Type {
    self.push_scope();
    let mut param_types = Vec::new();
    for (i, p) in func.params.iter().enumerate() {
      let ty = match p.type_ann {
        Some(ann) => self.resolve_type_ann(ann),
        None => {
          if i < exp_params.len() {
            exp_params[i].clone()
          } else {
            self.fresh()
          }
        },
      };
      self.bind(p.name, ty.clone());
      param_types.push(ty);
    }
    let body_type = self.check_expr(func.body, exp_ret);
    self.pop_scope();
    Type::Func { params: param_types, ret: Box::new(body_type) }
  }

  fn check_list(&mut self, elems: &[ListElem], exp_elem: &Type) -> Type {
    for elem in elems {
      match elem {
        ListElem::Single(e) | ListElem::Spread(e) => {
          self.check_expr(*e, exp_elem);
        },
      }
    }
    Type::List(Box::new(exp_elem.clone()))
  }

  fn check_match(&mut self, scrutinee: ExprId, arms: &[MatchArm], expected: &Type) -> Type {
    let scrut_t = self.synth_expr(scrutinee);
    let resolved_scrut = self.table.resolve(&scrut_t);
    let span = self.arena.expr_span(scrutinee);
    self.check_match_exhaustiveness(&resolved_scrut, arms, span);
    for arm in arms {
      self.push_scope();
      self.bind_pattern_vars(arm.pattern);
      if let Some(guard) = arm.guard {
        self.synth_expr(guard);
      }
      self.check_expr(arm.body, expected);
      self.pop_scope();
    }
    self.table.resolve(expected)
  }

  fn check_block(&mut self, stmts: &[StmtId], expected: &Type) -> Type {
    let Some((&last, init)) = stmts.split_last() else {
      return Type::Unit;
    };
    let arena = self.arena;
    for &sid in init {
      self.check_stmt(sid, arena);
    }
    let last_stmt = arena.stmt(last).clone();
    match &last_stmt {
      Stmt::Expr(e) => self.check_expr(*e, expected),
      _ => {
        self.check_stmt(last, arena);
        Type::Unit
      },
    }
  }
}
