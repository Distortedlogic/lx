use std::collections::HashSet;
use std::ops::ControlFlow;

use crate::ast::{BindTarget, Binding, FieldPattern, MatchArm, Param, SExpr, SPattern, SStmt, SType};
use crate::sym::Sym;
use crate::visitor::{walk_binding, walk_func, AstVisitor};
use miette::SourceSpan;

struct FreeVarCollector {
  free: HashSet<Sym>,
  scopes: Vec<HashSet<Sym>>,
}

impl FreeVarCollector {
  fn new() -> Self {
    Self {
      free: HashSet::new(),
      scopes: vec![HashSet::new()],
    }
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

impl AstVisitor for FreeVarCollector {
  fn visit_ident(&mut self, name: Sym, _span: SourceSpan) -> ControlFlow<()> {
    if !self.is_bound(name) {
      self.free.insert(name);
    }
    ControlFlow::Continue(())
  }

  fn visit_binding(&mut self, binding: &Binding, span: SourceSpan) -> ControlFlow<()> {
    walk_binding(self, binding, span)?;
    match &binding.target {
      BindTarget::Name(n) => self.bind(*n),
      BindTarget::Reassign(n) => {
        if !self.is_bound(*n) {
          self.free.insert(*n);
        }
      },
      BindTarget::Pattern(_) => {},
    }
    ControlFlow::Continue(())
  }

  fn visit_func(
    &mut self,
    params: &[Param],
    ret_type: Option<&SType>,
    guard: Option<&SExpr>,
    body: &SExpr,
    span: SourceSpan,
  ) -> ControlFlow<()> {
    self.push_scope();
    for p in params {
      self.bind(p.name);
    }
    walk_func(self, params, ret_type, guard, body, span)?;
    self.pop_scope();
    ControlFlow::Continue(())
  }

  fn visit_block(&mut self, stmts: &[SStmt], _span: SourceSpan) -> ControlFlow<()> {
    self.push_scope();
    for s in stmts {
      self.visit_stmt(&s.node, s.span)?;
    }
    self.pop_scope();
    ControlFlow::Continue(())
  }

  fn visit_pattern_bind(&mut self, name: Sym, _span: SourceSpan) -> ControlFlow<()> {
    self.bind(name);
    ControlFlow::Continue(())
  }

  fn visit_pattern_list(
    &mut self,
    elems: &[SPattern],
    rest: Option<Sym>,
    _span: SourceSpan,
  ) -> ControlFlow<()> {
    for e in elems {
      self.visit_pattern(&e.node, e.span)?;
    }
    if let Some(r) = rest {
      self.bind(r);
    }
    ControlFlow::Continue(())
  }

  fn visit_pattern_record(
    &mut self,
    fields: &[FieldPattern],
    rest: Option<Sym>,
    _span: SourceSpan,
  ) -> ControlFlow<()> {
    for f in fields {
      if let Some(ref p) = f.pattern {
        self.visit_pattern(&p.node, p.span)?;
      } else {
        self.bind(f.name);
      }
    }
    if let Some(r) = rest {
      self.bind(r);
    }
    ControlFlow::Continue(())
  }

  fn visit_with(
    &mut self,
    name: Sym,
    value: &SExpr,
    body: &[SStmt],
    _mutable: bool,
    _span: SourceSpan,
  ) -> ControlFlow<()> {
    self.visit_expr(&value.node, value.span)?;
    self.push_scope();
    self.bind(name);
    for s in body {
      self.visit_stmt(&s.node, s.span)?;
    }
    self.pop_scope();
    ControlFlow::Continue(())
  }

  fn visit_with_resource(
    &mut self,
    resources: &[(SExpr, Sym)],
    body: &[SStmt],
    _span: SourceSpan,
  ) -> ControlFlow<()> {
    for (r, _) in resources {
      self.visit_expr(&r.node, r.span)?;
    }
    self.push_scope();
    for (_, name) in resources {
      self.bind(*name);
    }
    for s in body {
      self.visit_stmt(&s.node, s.span)?;
    }
    self.pop_scope();
    ControlFlow::Continue(())
  }

  fn visit_with_context(
    &mut self,
    fields: &[(Sym, SExpr)],
    body: &[SStmt],
    _span: SourceSpan,
  ) -> ControlFlow<()> {
    for (_, expr) in fields {
      self.visit_expr(&expr.node, expr.span)?;
    }
    for s in body {
      self.visit_stmt(&s.node, s.span)?;
    }
    ControlFlow::Continue(())
  }

  fn visit_loop(&mut self, stmts: &[SStmt], _span: SourceSpan) -> ControlFlow<()> {
    self.push_scope();
    for s in stmts {
      self.visit_stmt(&s.node, s.span)?;
    }
    self.pop_scope();
    ControlFlow::Continue(())
  }

  fn visit_par(&mut self, stmts: &[SStmt], _span: SourceSpan) -> ControlFlow<()> {
    self.push_scope();
    for s in stmts {
      self.visit_stmt(&s.node, s.span)?;
    }
    self.pop_scope();
    ControlFlow::Continue(())
  }

  fn visit_match(
    &mut self,
    scrutinee: &SExpr,
    arms: &[MatchArm],
    _span: SourceSpan,
  ) -> ControlFlow<()> {
    self.visit_expr(&scrutinee.node, scrutinee.span)?;
    for arm in arms {
      self.push_scope();
      self.visit_pattern(&arm.pattern.node, arm.pattern.span)?;
      if let Some(ref g) = arm.guard {
        self.visit_expr(&g.node, g.span)?;
      }
      self.visit_expr(&arm.body.node, arm.body.span)?;
      self.pop_scope();
    }
    ControlFlow::Continue(())
  }
}

pub fn free_vars(expr: &SExpr) -> HashSet<Sym> {
  let mut collector = FreeVarCollector::new();
  let _ = collector.visit_expr(&expr.node, expr.span);
  collector.free
}
