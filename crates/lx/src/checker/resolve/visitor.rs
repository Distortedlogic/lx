use miette::SourceSpan;

use crate::ast::{
  AstArena, BindTarget, Binding, ClassDeclData, ExprFunc, ExprMatch, ExprWith, FieldPattern, PatternId, StmtId, StmtTypeDef, TraitDeclData, UseKind, UseStmt,
  WithKind,
};
use crate::sym::{Sym, intern};
use crate::visitor::{AstLeave, AstVisitor, VisitAction, cf_to_action, dispatch_expr, dispatch_stmt, walk_binding, walk_func, walk_pattern};

use super::Resolver;
use crate::checker::symbol_table::DefKind;

impl AstLeave for Resolver<'_> {}

impl AstVisitor for Resolver<'_> {
  fn visit_binding(&mut self, binding: &Binding, span: SourceSpan, arena: &AstArena) -> VisitAction {
    if walk_binding(self, binding, span, arena).is_break() {
      return VisitAction::Stop;
    }
    match &binding.target {
      BindTarget::Name(name) => {
        self.table.define(*name, DefKind::Binding, span);
      },
      BindTarget::Reassign(_) => {},
      BindTarget::Pattern(pid) => {
        self.bind_pattern_names(*pid);
      },
    }
    VisitAction::Skip
  }

  fn visit_func(&mut self, func: &ExprFunc, span: SourceSpan, arena: &AstArena) -> VisitAction {
    self.table.push_scope();
    for p in &func.params {
      self.table.define(p.name, DefKind::FuncParam, span);
    }
    let result = walk_func(self, func, span, arena);
    self.table.pop_scope();
    cf_to_action(result)
  }

  fn visit_block(&mut self, stmts: &[StmtId], _span: SourceSpan, arena: &AstArena) -> VisitAction {
    self.table.push_scope();
    for &s in stmts {
      if dispatch_stmt(self, s, arena).is_break() {
        self.table.pop_scope();
        return VisitAction::Stop;
      }
    }
    self.table.pop_scope();
    VisitAction::Skip
  }

  fn visit_match(&mut self, m: &ExprMatch, _span: SourceSpan, arena: &AstArena) -> VisitAction {
    if dispatch_expr(self, arena.expr(m.scrutinee), arena.expr_span(m.scrutinee), arena).is_break() {
      return VisitAction::Stop;
    }
    for arm in &m.arms {
      self.table.push_scope();
      self.bind_pattern_names(arm.pattern);
      let pattern = arena.pattern(arm.pattern);
      let pspan = arena.pattern_span(arm.pattern);
      if walk_pattern(self, pattern, pspan, arena).is_break() {
        self.table.pop_scope();
        return VisitAction::Stop;
      }
      if let Some(g) = arm.guard
        && dispatch_expr(self, arena.expr(g), arena.expr_span(g), arena).is_break()
      {
        self.table.pop_scope();
        return VisitAction::Stop;
      }
      if dispatch_expr(self, arena.expr(arm.body), arena.expr_span(arm.body), arena).is_break() {
        self.table.pop_scope();
        return VisitAction::Stop;
      }
      self.table.pop_scope();
    }
    VisitAction::Skip
  }

  fn visit_with(&mut self, with: &ExprWith, _span: SourceSpan, arena: &AstArena) -> VisitAction {
    match &with.kind {
      WithKind::Binding { name, value, mutable: _ } => {
        if dispatch_expr(self, arena.expr(*value), arena.expr_span(*value), arena).is_break() {
          return VisitAction::Stop;
        }
        let vspan = arena.expr_span(*value);
        self.table.push_scope();
        self.table.define(*name, DefKind::WithBinding, vspan);
        for &s in &with.body {
          if dispatch_stmt(self, s, arena).is_break() {
            self.table.pop_scope();
            return VisitAction::Stop;
          }
        }
        self.table.pop_scope();
      },
      WithKind::Resources { resources } => {
        for &(r, _) in resources {
          if dispatch_expr(self, arena.expr(r), arena.expr_span(r), arena).is_break() {
            return VisitAction::Stop;
          }
        }
        self.table.push_scope();
        for &(r, name) in resources {
          let rspan = arena.expr_span(r);
          self.table.define(name, DefKind::ResourceBinding, rspan);
        }
        for &s in &with.body {
          if dispatch_stmt(self, s, arena).is_break() {
            self.table.pop_scope();
            return VisitAction::Stop;
          }
        }
        self.table.pop_scope();
      },
      WithKind::Context { fields } => {
        for &(_, eid) in fields {
          if dispatch_expr(self, arena.expr(eid), arena.expr_span(eid), arena).is_break() {
            return VisitAction::Stop;
          }
        }
        self.table.push_scope();
        let ctx_name = intern("context");
        let fspan = fields.first().map(|f| arena.expr_span(f.1)).unwrap_or((0, 0).into());
        self.table.define(ctx_name, DefKind::WithBinding, fspan);
        for &s in &with.body {
          if dispatch_stmt(self, s, arena).is_break() {
            self.table.pop_scope();
            return VisitAction::Stop;
          }
        }
        self.table.pop_scope();
      },
    }
    VisitAction::Skip
  }

  fn visit_loop(&mut self, stmts: &[StmtId], _span: SourceSpan, arena: &AstArena) -> VisitAction {
    self.table.push_scope();
    for &s in stmts {
      if dispatch_stmt(self, s, arena).is_break() {
        self.table.pop_scope();
        return VisitAction::Stop;
      }
    }
    self.table.pop_scope();
    VisitAction::Skip
  }

  fn visit_par(&mut self, stmts: &[StmtId], _span: SourceSpan, arena: &AstArena) -> VisitAction {
    self.table.push_scope();
    for &s in stmts {
      if dispatch_stmt(self, s, arena).is_break() {
        self.table.pop_scope();
        return VisitAction::Stop;
      }
    }
    self.table.pop_scope();
    VisitAction::Skip
  }

  fn visit_use(&mut self, stmt: &UseStmt, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    match &stmt.kind {
      UseKind::Whole => {
        if let Some(name) = stmt.path.last() {
          self.table.define(*name, DefKind::Import, span);
        }
      },
      UseKind::Alias(alias) => {
        self.table.define(*alias, DefKind::Import, span);
      },
      UseKind::Selective(names) => {
        for name in names {
          self.table.define(*name, DefKind::Import, span);
        }
      },
    }
    VisitAction::Descend
  }

  fn visit_type_def(&mut self, def: &StmtTypeDef, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.table.define(def.name, DefKind::TypeDef, span);
    for (vname, _) in &def.variants {
      self.table.define(*vname, DefKind::TypeDef, span);
    }
    VisitAction::Descend
  }

  fn visit_trait_decl(&mut self, data: &TraitDeclData, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.table.define(data.name, DefKind::TraitDef, span);
    VisitAction::Descend
  }

  fn visit_class_decl(&mut self, data: &ClassDeclData, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.table.define(data.name, DefKind::ClassDef, span);
    VisitAction::Descend
  }

  fn visit_pattern_bind(&mut self, name: Sym, span: SourceSpan, _arena: &AstArena) -> VisitAction {
    self.table.define(name, DefKind::PatternBind, span);
    VisitAction::Descend
  }

  fn visit_pattern_list(&mut self, elems: &[PatternId], rest: Option<Sym>, span: SourceSpan, arena: &AstArena) -> VisitAction {
    for &e in elems {
      let pattern = arena.pattern(e);
      let pspan = arena.pattern_span(e);
      if walk_pattern(self, pattern, pspan, arena).is_break() {
        return VisitAction::Stop;
      }
    }
    if let Some(r) = rest {
      self.table.define(r, DefKind::PatternBind, span);
    }
    VisitAction::Skip
  }

  fn visit_pattern_record(&mut self, fields: &[FieldPattern], rest: Option<Sym>, span: SourceSpan, arena: &AstArena) -> VisitAction {
    for f in fields {
      if let Some(pid) = f.pattern {
        let pattern = arena.pattern(pid);
        let pspan = arena.pattern_span(pid);
        if walk_pattern(self, pattern, pspan, arena).is_break() {
          return VisitAction::Stop;
        }
      } else {
        self.table.define(f.name, DefKind::PatternBind, span);
      }
    }
    if let Some(r) = rest {
      self.table.define(r, DefKind::PatternBind, span);
    }
    VisitAction::Skip
  }
}
