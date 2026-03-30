use std::fmt;

use miette::SourceSpan;

use lx_span::sym::Sym;

use super::type_arena::TypeArena;
use super::type_error::TypeError;

pub struct TextEdit {
  pub range: SourceSpan,
  pub replacement: String,
}

#[derive(PartialEq)]
pub enum Applicability {
  MachineApplicable,
  MaybeIncorrect,
  DisplayOnly,
}

impl fmt::Display for Applicability {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::MachineApplicable => write!(f, "machine-applicable"),
      Self::MaybeIncorrect => write!(f, "maybe-incorrect"),
      Self::DisplayOnly => write!(f, "display-only"),
    }
  }
}

pub struct Fix {
  pub description: String,
  pub edits: Vec<TextEdit>,
  pub applicability: Applicability,
}

pub enum DiagnosticKind {
  NegationRequiresNumeric,
  PropagateRequiresResultOrMaybe,
  TernaryCondNotBool,
  TimeoutMsNotNumeric,
  LogicalOpRequiresBool,
  MutableCaptureInConcurrent { name: Sym },
  NonExhaustiveMatch { type_name: Sym, missing_pattern: String },
  DuplicateImport { name: Sym, original_span: SourceSpan },
  UnknownImport { name: Sym, module: Sym, suggestions: Vec<String> },
  TypeMismatch { error: TypeError },
  LintWarning { rule_name: String, message: String },
  UnknownIdent { name: Sym, suggestions: Vec<String> },
  UnknownModule { name: String, suggestions: Vec<String> },
}

impl DiagnosticKind {
  pub fn code(&self) -> &'static str {
    match self {
      Self::NegationRequiresNumeric => "E001",
      Self::PropagateRequiresResultOrMaybe => "E002",
      Self::TernaryCondNotBool => "E003",
      Self::TimeoutMsNotNumeric => "E004",
      Self::LogicalOpRequiresBool => "E005",
      Self::MutableCaptureInConcurrent { .. } => "E006",
      Self::NonExhaustiveMatch { .. } => "E007",
      Self::DuplicateImport { .. } => "W001",
      Self::UnknownImport { .. } => "E008",
      Self::TypeMismatch { .. } => "E009",
      Self::LintWarning { .. } => "L000",
      Self::UnknownIdent { .. } => "E010",
      Self::UnknownModule { .. } => "E011",
    }
  }

  pub fn help(&self, ta: &TypeArena) -> Option<String> {
    match self {
      Self::TypeMismatch { error } => error.help(ta),
      Self::UnknownImport { name, module, suggestions } => {
        super::suggest::format_suggestions(suggestions).or_else(|| Some(format!("'{name}' is not exported by module '{module}'")))
      },
      Self::UnknownIdent { name, suggestions } => {
        super::suggest::format_suggestions(suggestions).or_else(|| Some(format!("'{name}' is not defined in this scope")))
      },
      Self::UnknownModule { name, suggestions } => super::suggest::format_suggestions(suggestions).or_else(|| Some(format!("module '{name}' not found"))),
      _ => None,
    }
  }

  pub fn suggest_fix(&self, span: SourceSpan, ta: &TypeArena) -> Option<Fix> {
    match self {
      Self::DuplicateImport { .. } => Some(Fix {
        description: "remove duplicate import".into(),
        edits: vec![TextEdit { range: span, replacement: String::new() }],
        applicability: Applicability::MachineApplicable,
      }),
      Self::TypeMismatch { error } => {
        error.help(ta).map(|help_text| Fix { description: help_text, edits: Vec::new(), applicability: Applicability::DisplayOnly })
      },
      _ => None,
    }
  }

  pub fn display(&self, ta: &TypeArena) -> String {
    match self {
      Self::NegationRequiresNumeric => "negation requires Int or Float".into(),
      Self::PropagateRequiresResultOrMaybe => "^ requires Result or Maybe".into(),
      Self::TernaryCondNotBool => "ternary condition must be Bool".into(),
      Self::TimeoutMsNotNumeric => "timeout ms must be Int or Float".into(),
      Self::LogicalOpRequiresBool => "logical operator requires Bool".into(),
      Self::MutableCaptureInConcurrent { name } => {
        format!("cannot capture mutable binding `{name}` in concurrent context")
      },
      Self::NonExhaustiveMatch { type_name, missing_pattern } => {
        format!("non-exhaustive match on {type_name}: missing {missing_pattern}")
      },
      Self::DuplicateImport { name, .. } => {
        format!("'{name}' already imported")
      },
      Self::UnknownImport { name, module, .. } => {
        format!("'{name}' is not exported by module '{module}'")
      },
      Self::TypeMismatch { error } => error.to_message(ta),
      Self::LintWarning { rule_name, message } => {
        format!("[{rule_name}] {message}")
      },
      Self::UnknownIdent { name, .. } => {
        format!("unknown identifier '{name}'")
      },
      Self::UnknownModule { name, .. } => {
        format!("unknown module '{name}'")
      },
    }
  }
}
