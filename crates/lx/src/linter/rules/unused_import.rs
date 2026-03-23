use crate::ast::{AstArena, Stmt, StmtId, UseKind};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::{DefKind, SemanticModel};
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct UnusedImport;

impl LintRule for UnusedImport {
  fn name(&self) -> &'static str {
    "unused_import"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn check_stmt(&mut self, _id: StmtId, stmt: &Stmt, span: SourceSpan, model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
    let Stmt::Use(use_stmt) = stmt else {
      return vec![];
    };

    let mut diags = vec![];

    let names_to_check: Vec<_> = match &use_stmt.kind {
      UseKind::Whole => use_stmt.path.last().map(|n| vec![*n]).unwrap_or_default(),
      UseKind::Alias(alias) => vec![*alias],
      UseKind::Selective(names) => names.clone(),
    };

    for name in &names_to_check {
      let def = model.definitions.iter().enumerate().find(|(_, d)| matches!(d.kind, DefKind::Import) && d.name == *name && d.span == span);

      if let Some((def_id, _)) = def {
        let refs = model.references_to(def_id);
        if refs.is_empty() {
          diags.push(Diagnostic {
            level: DiagLevel::Warning,
            kind: DiagnosticKind::LintWarning { rule_name: "unused_import".into(), message: format!("unused import '{name}'") },
            span,
            secondary: vec![],
            fix: None,
          });
        }
      }
    }

    diags
  }
}
