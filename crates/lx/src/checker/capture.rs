use std::collections::HashSet;
use std::ops::ControlFlow;

use crate::ast::{AstArena, BindTarget, Binding, ExprFunc, ExprId, ExprMatch, ExprWith, FieldPattern, PatternId, StmtId, WithKind};
use crate::sym::Sym;
use crate::visitor::{AstLeave, AstVisitor, VisitAction, cf_to_action, dispatch_expr, dispatch_stmt, walk_binding, walk_func, walk_pattern};
use miette::SourceSpan;

struct FreeVarCollector {
  free: HashSet<Sym>,
  scopes: Vec<HashSet<Sym>>,
}

impl FreeVarCollector {
  fn new() -> Self {
    Self { free: HashSet::new(), scopes: vec![HashSet::new()] }
  }

  fn is_bound(&self, name: Sym) -> bool {
    self.scopes.iter().any(|s| s.contains(&name))
  }

  fn push_scope(&mut self) {
    self.scopes.push(HashSet::new());
  }

  fn pop_scope(&mut self) {
    self.scopes.pop();
  }

  fn bind(&mut self, name: Sym) {
    if let Some(scope) = self.scopes.last_mut() {
      scope.insert(name);
    }
  }
}

impl AstLeave for FreeVarCollector {}

impl AstVisitor for FreeVarCollector {
  fn visit_ident(&mut self, name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    if !self.is_bound(name) {
      self.free.insert(name);
    }
    VisitAction::Descend
  }

  fn visit_binding(&mut self, binding: &Binding, span: SourceSpan, arena: &AstArena) -> VisitAction {
    if walk_binding(self, binding, span, arena).is_break() {
      return VisitAction::Stop;
    }
    match &binding.target {
      BindTarget::Name(n) => self.bind(*n),
      BindTarget::Reassign(n) => {
        if !self.is_bound(*n) {
          self.free.insert(*n);
        }
      },
      BindTarget::Pattern(_) => {},
    }
    VisitAction::Skip
  }

  fn visit_func(&mut self, func: &ExprFunc, span: SourceSpan, arena: &AstArena) -> VisitAction {
    self.push_scope();
    for p in &func.params {
      self.bind(p.name);
    }
    let result = walk_func(self, func, span, arena);
    self.pop_scope();
    cf_to_action(result)
  }

  fn visit_block(&mut self, stmts: &[StmtId], _span: SourceSpan, arena: &AstArena) -> VisitAction {
    self.push_scope();
    for &s in stmts {
      if dispatch_stmt(self, s, arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
    }
    self.pop_scope();
    VisitAction::Skip
  }

  fn visit_pattern_bind(&mut self, name: Sym, _span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.bind(name);
    VisitAction::Descend
  }

  fn visit_pattern_list(&mut self, elems: &[PatternId], rest: Option<Sym>, _span: SourceSpan, arena: &AstArena) -> VisitAction {
    for &e in elems {
      let pattern = arena.pattern(e);
      let pspan = arena.pattern_span(e);
      if walk_pattern(self, pattern, pspan, arena).is_break() {
        return VisitAction::Stop;
      }
    }
    if let Some(r) = rest {
      self.bind(r);
    }
    VisitAction::Skip
  }

  fn visit_pattern_record(&mut self, fields: &[FieldPattern], rest: Option<Sym>, _span: SourceSpan, arena: &AstArena) -> VisitAction {
    for f in fields {
      if let Some(pid) = f.pattern {
        let pattern = arena.pattern(pid);
        let pspan = arena.pattern_span(pid);
        if walk_pattern(self, pattern, pspan, arena).is_break() {
          return VisitAction::Stop;
        }
      } else {
        self.bind(f.name);
      }
    }
    if let Some(r) = rest {
      self.bind(r);
    }
    VisitAction::Skip
  }

  fn visit_with(&mut self, with: &ExprWith, _span: SourceSpan, arena: &AstArena) -> VisitAction {
    match &with.kind {
      WithKind::Binding { name, value, .. } => {
        if dispatch_expr(self, arena.expr(*value), arena.expr_span(*value), arena).is_break() {
          return VisitAction::Stop;
        }
        self.push_scope();
        self.bind(*name);
        for &s in &with.body {
          if dispatch_stmt(self, s, arena).is_break() {
            self.pop_scope();
            return VisitAction::Stop;
          }
        }
        self.pop_scope();
      },
      WithKind::Resources { resources } => {
        for &(r, _) in resources {
          if dispatch_expr(self, arena.expr(r), arena.expr_span(r), arena).is_break() {
            return VisitAction::Stop;
          }
        }
        self.push_scope();
        for &(_, name) in resources {
          self.bind(name);
        }
        for &s in &with.body {
          if dispatch_stmt(self, s, arena).is_break() {
            self.pop_scope();
            return VisitAction::Stop;
          }
        }
        self.pop_scope();
      },
      WithKind::Context { fields } => {
        for &(_, eid) in fields {
          if dispatch_expr(self, arena.expr(eid), arena.expr_span(eid), arena).is_break() {
            return VisitAction::Stop;
          }
        }
        for &s in &with.body {
          if dispatch_stmt(self, s, arena).is_break() {
            return VisitAction::Stop;
          }
        }
      },
    }
    VisitAction::Skip
  }

  fn visit_loop(&mut self, stmts: &[StmtId], _span: SourceSpan, arena: &AstArena) -> VisitAction {
    self.push_scope();
    for &s in stmts {
      if dispatch_stmt(self, s, arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
    }
    self.pop_scope();
    VisitAction::Skip
  }

  fn visit_par(&mut self, stmts: &[StmtId], _span: SourceSpan, arena: &AstArena) -> VisitAction {
    self.push_scope();
    for &s in stmts {
      if dispatch_stmt(self, s, arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
    }
    self.pop_scope();
    VisitAction::Skip
  }

  fn visit_match(&mut self, m: &ExprMatch, _span: SourceSpan, arena: &AstArena) -> VisitAction {
    if dispatch_expr(self, arena.expr(m.scrutinee), arena.expr_span(m.scrutinee), arena).is_break() {
      return VisitAction::Stop;
    }
    for arm in &m.arms {
      self.push_scope();
      let pattern = arena.pattern(arm.pattern);
      let pspan = arena.pattern_span(arm.pattern);
      if walk_pattern(self, pattern, pspan, arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
      if let Some(g) = arm.guard
        && dispatch_expr(self, arena.expr(g), arena.expr_span(g), arena).is_break()
      {
        self.pop_scope();
        return VisitAction::Stop;
      }
      if dispatch_expr(self, arena.expr(arm.body), arena.expr_span(arm.body), arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
      self.pop_scope();
    }
    VisitAction::Skip
  }
}

pub fn free_vars(eid: ExprId, arena: &AstArena) -> HashSet<Sym> {
  let mut collector = FreeVarCollector::new();
  match dispatch_expr(&mut collector, arena.expr(eid), arena.expr_span(eid), arena) {
    ControlFlow::Continue(()) | ControlFlow::Break(()) => {},
  }
  collector.free
}
