use crate::ast::{Expr, ExprFunc, ExprId, ListElem, MatchArm, Stmt, StmtId};

use super::Checker;
use super::symbol_table::DefKind;
use super::type_arena::TypeId;
use super::types::Type;
use super::unification::TypeContext;

impl Checker<'_> {
  pub(super) fn check_expr(&mut self, eid: ExprId, expected: TypeId) -> TypeId {
    let arena = self.arena;
    let expr = arena.expr(eid);
    match expr {
      Expr::Func(func) => {
        let func = func.clone();
        let resolved = self.table.resolve(expected, &self.type_arena);
        match self.type_arena.get(resolved).clone() {
          Type::Func { param, ret } => {
            let mut exp_params = vec![param];
            let mut cur = ret;
            loop {
              let resolved_cur = self.table.resolve(cur, &self.type_arena);
              match self.type_arena.get(resolved_cur).clone() {
                Type::Func { param: p, ret: r } => {
                  exp_params.push(p);
                  cur = r;
                },
                _ => break,
              }
            }
            self.check_func_against(&func, &exp_params, cur)
          },
          _ => self.synth_expr(eid),
        }
      },
      Expr::List(elems) => {
        let resolved = self.table.resolve(expected, &self.type_arena);
        match self.type_arena.get(resolved).clone() {
          Type::List(exp_elem) => self.check_list(elems, exp_elem),
          _ => self.synth_expr(eid),
        }
      },
      Expr::Match(m) => self.check_match(m.scrutinee, &m.arms, expected),
      Expr::Block(stmts) => self.check_block(stmts, expected),
      _ => {
        let actual = self.synth_expr(eid);
        let span = arena.expr_span(eid);
        match self.table.unify_with_context(expected, actual, TypeContext::General, &mut self.type_arena) {
          Ok(t) => t,
          Err(te) => {
            self.emit_type_error(&te, span);
            self.type_arena.error()
          },
        }
      },
    }
  }

  fn check_func_against(&mut self, func: &ExprFunc, exp_params: &[TypeId], exp_ret: TypeId) -> TypeId {
    self.symbols.push_scope();
    let func_span = self.arena.expr_span(func.body);
    let mut param_types = Vec::new();
    for (i, p) in func.params.iter().enumerate() {
      let ty = match p.type_ann {
        Some(ann) => self.resolve_type_ann(ann),
        None => {
          if i < exp_params.len() {
            exp_params[i]
          } else {
            self.fresh()
          }
        },
      };
      self.symbols.define(p.name, DefKind::FuncParam, func_span);
      self.symbols.set_type(p.name, ty);
      param_types.push(ty);
    }
    let body_type = self.check_expr(func.body, exp_ret);
    self.symbols.pop_scope();
    let mut func_type = body_type;
    for &p in param_types.iter().rev() {
      func_type = self.type_arena.alloc(Type::Func { param: p, ret: func_type });
    }
    func_type
  }

  fn check_list(&mut self, elems: &[ListElem], exp_elem: TypeId) -> TypeId {
    for elem in elems {
      match elem {
        ListElem::Single(e) | ListElem::Spread(e) => {
          self.check_expr(*e, exp_elem);
        },
      }
    }
    self.type_arena.alloc(Type::List(exp_elem))
  }

  fn check_match(&mut self, scrutinee: ExprId, arms: &[MatchArm], expected: TypeId) -> TypeId {
    let scrut_t = self.synth_expr(scrutinee);
    let resolved_scrut = self.table.resolve(scrut_t, &self.type_arena);
    let span = self.arena.expr_span(scrutinee);
    self.check_match_exhaustiveness(resolved_scrut, arms, span);
    for arm in arms {
      self.symbols.push_scope();
      self.bind_pattern_vars(arm.pattern);
      if let Some(guard) = arm.guard {
        self.synth_expr(guard);
      }
      self.check_expr(arm.body, expected);
      self.symbols.pop_scope();
    }
    self.table.resolve(expected, &self.type_arena)
  }

  fn check_block(&mut self, stmts: &[StmtId], expected: TypeId) -> TypeId {
    let Some((&last, init)) = stmts.split_last() else {
      return self.type_arena.unit();
    };
    let arena = self.arena;
    for &sid in init {
      self.check_stmt(sid, arena);
    }
    let last_stmt = arena.stmt(last);
    match last_stmt {
      Stmt::Expr(e) => self.check_expr(*e, expected),
      _ => {
        self.check_stmt(last, arena);
        self.type_arena.unit()
      },
    }
  }
}
