use crate::ast::{Expr, ExprBlock, ExprFunc, ExprId, ListElem, MatchArm, Stmt, StmtId};

use super::Checker;
use super::narrowing;
use super::semantic::{DefKind, ScopeKind};
use super::type_arena::TypeId;
use super::type_error::TypeContext;
use super::types::Type;

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
      Expr::Block(ExprBlock { stmts }) => self.check_block(stmts, expected),
      Expr::Grouped(inner) => self.check_expr(*inner, expected),
      Expr::Pipe(_) | Expr::Tell(_) | Expr::Ask(_) | Expr::Section(_) | Expr::Ternary(_) | Expr::Coalesce(_) => unreachable!(),
      Expr::Spawn(inner) => {
        self.check_expr(*inner, expected);
        self.synth_expr(eid)
      },
      Expr::Stop => self.type_arena.unit(),
      Expr::Literal(_)
      | Expr::Ident(_)
      | Expr::TypeConstructor(_)
      | Expr::Binary(_)
      | Expr::Unary(_)
      | Expr::Apply(_)
      | Expr::FieldAccess(_)
      | Expr::Tuple(_)
      | Expr::Record(_)
      | Expr::Map(_)
      | Expr::Propagate(_)
      | Expr::Slice(_)
      | Expr::NamedArg(_)
      | Expr::Loop(_)
      | Expr::Break(_)
      | Expr::Assert(_)
      | Expr::Par(_)
      | Expr::Sel(_)
      | Expr::Timeout(_)
      | Expr::Emit(_)
      | Expr::Yield(_)
      | Expr::With(_) => {
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
    let func_span = self.arena.expr_span(func.body);
    self.sem.push_scope(ScopeKind::Function, func_span);
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
      let def_id = self.sem.add_definition(p.name, DefKind::FuncParam, func_span, false);
      self.sem.set_definition_type(def_id, ty);
      param_types.push(ty);
    }
    let body_type = self.check_expr(func.body, exp_ret);
    self.sem.pop_scope();
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
      let arm_span = self.arena.pattern_span(arm.pattern);
      self.sem.push_scope(ScopeKind::MatchArm, arm_span);
      self.infer_pattern_bindings(arm.pattern, resolved_scrut);
      self.narrowing.push();
      if let Expr::Ident(scrut_name) = self.arena.expr(scrutinee) {
        let pattern = self.arena.pattern(arm.pattern).clone();
        let resolved_type = self.type_arena.get(resolved_scrut).clone();
        let narrowed = narrowing::compute_narrowed_type(&pattern, resolved_scrut, &mut self.type_arena, &resolved_type);
        self.narrowing.narrow(*scrut_name, narrowed);
      }
      if let Some(guard) = arm.guard {
        self.synth_expr(guard);
      }
      self.check_expr(arm.body, expected);
      self.narrowing.pop();
      self.sem.pop_scope();
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
      Stmt::Binding(_)
      | Stmt::TypeDef(_)
      | Stmt::TraitUnion(_)
      | Stmt::TraitDecl(_)
      | Stmt::ClassDecl(_)
      | Stmt::KeywordDecl(_)
      | Stmt::FieldUpdate(_)
      | Stmt::Use(_)
      | Stmt::ChannelDecl(_) => {
        self.check_stmt(last, arena);
        self.type_arena.unit()
      },
    }
  }
}
