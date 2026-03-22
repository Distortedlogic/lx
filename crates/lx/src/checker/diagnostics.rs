use std::fmt;

use crate::sym::Sym;

use super::types::TypeError;

pub enum DiagnosticKind {
  NegationRequiresNumeric,
  PropagateRequiresResultOrMaybe,
  TernaryCondNotBool,
  TimeoutMsNotNumeric,
  LogicalOpRequiresBool,
  MutableCaptureInConcurrent { name: Sym },
  NonExhaustiveMatch { type_name: Sym, missing_variant: Sym },
  DuplicateImport { name: Sym, original_offset: usize },
  TypeMismatch { error: TypeError },
}

impl DiagnosticKind {
  pub fn help(&self) -> Option<String> {
    match self {
      Self::TypeMismatch { error } => error.help(),
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
      Self::NonExhaustiveMatch { type_name, missing_variant } => {
        write!(f, "non-exhaustive match on {type_name}: missing {missing_variant}")
      },
      Self::DuplicateImport { name, original_offset } => {
        write!(f, "'{name}' already imported at offset {original_offset}")
      },
      Self::TypeMismatch { error } => write!(f, "{}", error.to_message()),
    }
  }
}
