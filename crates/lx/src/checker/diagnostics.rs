use std::fmt;

use miette::SourceSpan;

use crate::sym::Sym;

use super::unification::TypeError;

pub struct TextEdit {
  pub range: SourceSpan,
  pub replacement: String,
}

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
  TypeMismatch { error: TypeError },
}

impl DiagnosticKind {
  pub fn help(&self) -> Option<String> {
    match self {
      Self::TypeMismatch { error } => error.help(),
      _ => None,
    }
  }

  pub fn suggest_fix(&self, span: SourceSpan) -> Option<Fix> {
    match self {
      Self::DuplicateImport { .. } => Some(Fix {
        description: "remove duplicate import".into(),
        edits: vec![TextEdit { range: span, replacement: String::new() }],
        applicability: Applicability::MachineApplicable,
      }),
      Self::TypeMismatch { error } => {
        error.help().map(|help_text| Fix { description: help_text, edits: Vec::new(), applicability: Applicability::DisplayOnly })
      },
      _ => None,
    }
  }
}

impl fmt::Display for DiagnosticKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NegationRequiresNumeric => write!(f, "negation requires Int or Float"),
      Self::PropagateRequiresResultOrMaybe => write!(f, "^ requires Result or Maybe"),
      Self::TernaryCondNotBool => write!(f, "ternary condition must be Bool"),
      Self::TimeoutMsNotNumeric => write!(f, "timeout ms must be Int or Float"),
      Self::LogicalOpRequiresBool => write!(f, "logical operator requires Bool"),
      Self::MutableCaptureInConcurrent { name } => {
        write!(f, "cannot capture mutable binding `{name}` in concurrent context")
      },
      Self::NonExhaustiveMatch { type_name, missing_pattern } => {
        write!(f, "non-exhaustive match on {type_name}: missing {missing_pattern}")
      },
      Self::DuplicateImport { name, .. } => {
        write!(f, "'{name}' already imported")
      },
      Self::TypeMismatch { error } => write!(f, "{}", error.to_message()),
    }
  }
}
