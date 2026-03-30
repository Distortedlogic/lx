use std::collections::HashSet;
use std::ops::ControlFlow;

use lx_ast::ast::{
  AstArena, BindTarget, Binding, ExprBlock, ExprFunc, ExprId, ExprLoop, ExprMatch, ExprPar, ExprWith, FieldPattern, PatternId, StmtId, WithKind,
};
use lx_ast::visitor::prelude::*;
use lx_ast::visitor::{walk_binding, walk_func};
use lx_span::sym::Sym;

struct FreeVarCollector<'a> {
  arena: &'a AstArena,
  free: HashSet<Sym>,
  scopes: Vec<HashSet<Sym>>,
}

impl<'a> FreeVarCollector<'a> {
  fn new(arena: &'a AstArena) -> Self {
    Self { arena, free: HashSet::new(), scopes: vec![HashSet::new()] }
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

impl AstVisitor for FreeVarCollector<'_> {
  fn visit_pattern_bind(&mut self, _id: PatternId, name: Sym, _span: SourceSpan) -> VisitAction {
    self.bind(name);
    VisitAction::Descend
  }

  fn visit_pattern_list(&mut self, _id: PatternId, elems: &[PatternId], rest: Option<Sym>, _span: SourceSpan) -> VisitAction {
    for &e in elems {
      if walk_pattern_dispatch(self, e, self.arena).is_break() {
        return VisitAction::Stop;
      }
    }
    if let Some(r) = rest {
      self.bind(r);
    }
    VisitAction::Skip
  }

  fn visit_pattern_record(&mut self, _id: PatternId, fields: &[FieldPattern], rest: Option<Sym>, _span: SourceSpan) -> VisitAction {
    for f in fields {
      if let Some(pid) = f.pattern {
        if walk_pattern_dispatch(self, pid, self.arena).is_break() {
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

  fn visit_ident(&mut self, _id: ExprId, name: Sym, _span: SourceSpan) -> VisitAction {
    if !self.is_bound(name) {
      self.free.insert(name);
    }
    VisitAction::Descend
  }

  fn visit_binding(&mut self, id: StmtId, binding: &Binding, span: SourceSpan) -> VisitAction {
    if walk_binding(self, id, binding, span, self.arena).is_break() {
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

  fn visit_func(&mut self, _id: ExprId, func: &ExprFunc, span: SourceSpan) -> VisitAction {
    self.push_scope();
    for p in &func.params {
      self.bind(p.name);
    }
    let result = walk_func(self, _id, func, span, self.arena);
    self.pop_scope();
    match result {
      ControlFlow::Continue(()) => VisitAction::Skip,
      ControlFlow::Break(()) => VisitAction::Stop,
    }
  }

  fn visit_block(&mut self, _id: ExprId, block: &ExprBlock, _span: SourceSpan) -> VisitAction {
    self.push_scope();
    for &s in &block.stmts {
      if dispatch_stmt(self, s, self.arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
    }
    self.pop_scope();
    VisitAction::Skip
  }

  fn visit_with(&mut self, _id: ExprId, with: &ExprWith, _span: SourceSpan) -> VisitAction {
    match &with.kind {
      WithKind::Binding { name, value, .. } => {
        if dispatch_expr(self, *value, self.arena).is_break() {
          return VisitAction::Stop;
        }
        self.push_scope();
        self.bind(*name);
        for &s in &with.body {
          if dispatch_stmt(self, s, self.arena).is_break() {
            self.pop_scope();
            return VisitAction::Stop;
          }
        }
        self.pop_scope();
      },
      WithKind::Resources { resources } => {
        for &(r, _) in resources {
          if dispatch_expr(self, r, self.arena).is_break() {
            return VisitAction::Stop;
          }
        }
        self.push_scope();
        for &(_, name) in resources {
          self.bind(name);
        }
        for &s in &with.body {
          if dispatch_stmt(self, s, self.arena).is_break() {
            self.pop_scope();
            return VisitAction::Stop;
          }
        }
        self.pop_scope();
      },
      WithKind::Context { fields } => {
        for &(_, eid) in fields {
          if dispatch_expr(self, eid, self.arena).is_break() {
            return VisitAction::Stop;
          }
        }
        for &s in &with.body {
          if dispatch_stmt(self, s, self.arena).is_break() {
            return VisitAction::Stop;
          }
        }
      },
    }
    VisitAction::Skip
  }

  fn visit_loop(&mut self, _id: ExprId, loop_node: &ExprLoop, _span: SourceSpan) -> VisitAction {
    self.push_scope();
    for &s in &loop_node.stmts {
      if dispatch_stmt(self, s, self.arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
    }
    self.pop_scope();
    VisitAction::Skip
  }

  fn visit_par(&mut self, _id: ExprId, par: &ExprPar, _span: SourceSpan) -> VisitAction {
    self.push_scope();
    for &s in &par.stmts {
      if dispatch_stmt(self, s, self.arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
    }
    self.pop_scope();
    VisitAction::Skip
  }

  fn visit_match(&mut self, _id: ExprId, m: &ExprMatch, _span: SourceSpan) -> VisitAction {
    if dispatch_expr(self, m.scrutinee, self.arena).is_break() {
      return VisitAction::Stop;
    }
    for arm in &m.arms {
      self.push_scope();
      if walk_pattern_dispatch(self, arm.pattern, self.arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
      if let Some(g) = arm.guard
        && dispatch_expr(self, g, self.arena).is_break()
      {
        self.pop_scope();
        return VisitAction::Stop;
      }
      if dispatch_expr(self, arm.body, self.arena).is_break() {
        self.pop_scope();
        return VisitAction::Stop;
      }
      self.pop_scope();
    }
    VisitAction::Skip
  }
}

pub fn free_vars(eid: ExprId, arena: &AstArena) -> HashSet<Sym> {
  let mut collector = FreeVarCollector::new(arena);
  match dispatch_expr(&mut collector, eid, arena) {
    ControlFlow::Continue(()) | ControlFlow::Break(()) => {},
  }
  collector.free
}
