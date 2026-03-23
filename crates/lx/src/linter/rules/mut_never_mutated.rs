use std::collections::HashSet;

use crate::ast::{AstArena, BindTarget, Expr, ExprId, Program, Stmt, StmtId};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::{DefKind, DefinitionInfo, SemanticModel};
use crate::checker::{DiagLevel, Diagnostic};
use crate::sym::Sym;

pub fn check_unused_mut<P>(program: &Program<P>, model: &SemanticModel, arena: &AstArena) -> Vec<Diagnostic> {
  let mut_defs: Vec<(usize, &DefinitionInfo)> = model.definitions.iter().enumerate().filter(|(_, d)| d.mutable && matches!(d.kind, DefKind::Binding)).collect();

  let mut mutated_names: HashSet<Sym> = HashSet::new();
  for &sid in &program.stmts {
    collect_mutations(sid, arena, &mut mutated_names);
  }

  let mut diags = vec![];
  for (_, def) in mut_defs {
    if !mutated_names.contains(&def.name) {
      diags.push(Diagnostic {
        level: DiagLevel::Warning,
        kind: DiagnosticKind::LintWarning {
          rule_name: "mut_never_mutated".into(),
          message: format!("binding '{}' declared as mut but never mutated", def.name),
        },
        span: def.span,
        secondary: vec![],
        fix: None,
      });
    }
  }
  diags
}

fn collect_mutations(sid: StmtId, arena: &AstArena, mutated: &mut HashSet<Sym>) {
  let stmt = arena.stmt(sid);
  match stmt {
    Stmt::FieldUpdate(fu) => {
      mutated.insert(fu.name);
    },
    Stmt::Binding(b) => {
      if let BindTarget::Reassign(name) = &b.target {
        mutated.insert(*name);
      }
      collect_mutations_expr(b.value, arena, mutated);
    },
    Stmt::Expr(eid) => {
      collect_mutations_expr(*eid, arena, mutated);
    },
    _ => {},
  }
}

fn collect_mutations_expr(eid: ExprId, arena: &AstArena, mutated: &mut HashSet<Sym>) {
  let expr = arena.expr(eid);
  match expr {
    Expr::Block(stmts) | Expr::Loop(stmts) | Expr::Par(stmts) => {
      for &sid in stmts {
        collect_mutations(sid, arena, mutated);
      }
    },
    Expr::With(w) => {
      for &sid in &w.body {
        collect_mutations(sid, arena, mutated);
      }
    },
    Expr::Match(m) => {
      for arm in &m.arms {
        collect_mutations_expr(arm.body, arena, mutated);
      }
    },
    Expr::Func(f) => {
      collect_mutations_expr(f.body, arena, mutated);
    },
    _ => {},
  }
}
