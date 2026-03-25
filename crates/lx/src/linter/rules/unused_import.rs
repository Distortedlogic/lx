use crate::ast::{Core, Program, Stmt, UseKind};
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::{DefKind, DefinitionId, SemanticModel};
use crate::checker::{DiagLevel, Diagnostic};
use crate::linter::rule::{LintRule, RuleCategory};

pub struct UnusedImport {
  diagnostics: Vec<Diagnostic>,
}

impl Default for UnusedImport {
  fn default() -> Self {
    Self::new()
  }
}

impl UnusedImport {
  pub fn new() -> Self {
    Self { diagnostics: Vec::new() }
  }
}

impl LintRule for UnusedImport {
  fn name(&self) -> &'static str {
    "unused_import"
  }

  fn code(&self) -> &'static str {
    "L005"
  }

  fn category(&self) -> RuleCategory {
    RuleCategory::Correctness
  }

  fn run(&mut self, program: &Program<Core>, model: &SemanticModel) {
    for sid in &program.stmts {
      let span = program.arena.stmt_span(*sid);
      let stmt = program.arena.stmt(*sid);
      let Stmt::Use(use_stmt) = stmt else {
        continue;
      };

      let names_to_check: Vec<_> = match &use_stmt.kind {
        UseKind::Whole => use_stmt.path.last().map(|n| vec![*n]).unwrap_or_default(),
        UseKind::Alias(alias) => vec![*alias],
        UseKind::Selective(names) => names.clone(),
      };

      for name in &names_to_check {
        let def = model.definitions.iter().enumerate().find(|(_, d)| matches!(d.kind, DefKind::Import) && d.name == *name && d.span == span);

        if let Some((idx, _)) = def {
          let refs = model.references_to(DefinitionId::new(idx));
          if refs.is_empty() {
            self.diagnostics.push(Diagnostic {
              level: DiagLevel::Warning,
              kind: DiagnosticKind::LintWarning { rule_name: "unused_import".into(), message: format!("unused import '{name}'") },
              code: "L005",
              span,
              secondary: vec![],
              fix: None,
            });
          }
        }
      }
    }
  }

  fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    std::mem::take(&mut self.diagnostics)
  }
}
