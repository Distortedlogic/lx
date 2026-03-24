use crate::ast::{AstArena, Expr, Stmt, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};

pub struct UnreachableCode {
  diagnostics: Vec<Diagnostic>,
}

impl Default for UnreachableCode {
  fn default() -> Self {
    Self::new()
  }
}

impl UnreachableCode {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new() }
  }

  fn walk_stmts(&mut self, stmts: &[StmtId], arena: &AstArena) {
    let mut found_break = false;
    for &sid in stmts {
      let stmt_span = arena.stmt_span(sid);
      if found_break {
        self.diagnostics.push(Diagnostic {
          level: DiagLevel::Warning,
          kind: DiagnosticKind::LintWarning { rule_name: "unreachable_code".into(), message: "unreachable code after break".into() },
          code: "L004",
          span: stmt_span,
          secondary: vec![],
          fix: None,
        });
        break;
      }
      let stmt = arena.stmt(sid);
      if let Stmt::Expr(eid) = stmt
        && matches!(arena.expr(*eid), Expr::Break(_))
      {
        found_break = true;
      }
      self.walk_stmt(sid, arena);
    }
  }

  fn walk_stmt(&mut self, sid: StmtId, arena: &AstArena) {
    match arena.stmt(sid) {
      Stmt::Binding(b) => self.walk_expr(b.value, arena),
      Stmt::Expr(eid) => self.walk_expr(*eid, arena),
      _ => {},
    }
  }

  fn walk_expr(&mut self, eid: crate::ast::ExprId, arena: &AstArena) {
    match arena.expr(eid) {
      Expr::Block(stmts) | Expr::Loop(stmts) => self.walk_stmts(stmts, arena),
      Expr::Func(f) => self.walk_expr(f.body, arena),
      Expr::Match(m) => {
        self.walk_expr(m.scrutinee, arena);
        for arm in &m.arms {
          self.walk_expr(arm.body, arena);
        }
      },
      Expr::Ternary(t) => {
        self.walk_expr(t.cond, arena);
        self.walk_expr(t.then_, arena);
        if let Some(e) = t.else_ {
          self.walk_expr(e, arena);
        }
      },
      Expr::Binary(b) => {
        self.walk_expr(b.left, arena);
        self.walk_expr(b.right, arena);
      },
      Expr::Unary(u) => self.walk_expr(u.operand, arena),
      Expr::Apply(a) => {
        self.walk_expr(a.func, arena);
        self.walk_expr(a.arg, arena);
      },
      Expr::Par(stmts) => {
        for &sid in stmts {
          self.walk_stmt(sid, arena);
        }
      },
      Expr::Sel(arms) => {
        for arm in arms {
          self.walk_expr(arm.expr, arena);
          self.walk_expr(arm.handler, arena);
        }
      },
      Expr::Propagate(inner) | Expr::Break(Some(inner)) => self.walk_expr(*inner, arena),
      Expr::Pipe(p) => {
        self.walk_expr(p.left, arena);
        self.walk_expr(p.right, arena);
      },
      Expr::Coalesce(c) => {
        self.walk_expr(c.expr, arena);
        self.walk_expr(c.default, arena);
      },
      Expr::Timeout(t) => {
        self.walk_expr(t.ms, arena);
        self.walk_expr(t.body, arena);
      },
      Expr::With(w) => {
        for &sid in &w.body {
          self.walk_stmt(sid, arena);
        }
      },
      Expr::FieldAccess(fa) => self.walk_expr(fa.expr, arena),
      Expr::Tuple(elems) => {
        for &e in elems {
          self.walk_expr(e, arena);
        }
      },
      _ => {},
    }
  }
}

impl LintRule for UnreachableCode {
  fn name(&self) -> &'static str {
    "unreachable_code"
  }

  fn code(&self) -> &'static str {
    "L004"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn run(&mut self, stmts: &[StmtId], arena: &AstArena, _model: &SemanticModel) {
    self.walk_stmts(stmts, arena);
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    std::mem::take(&mut self.diagnostics)
  }
}
