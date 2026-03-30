use miette::SourceSpan;

use super::type_arena::{TypeArena, TypeId};
use super::types::Type;

#[derive(Clone)]
pub struct TypeError {
  pub expected: TypeId,
  pub found: TypeId,
  pub context: TypeContext,
  pub expected_origin: Option<SourceSpan>,
}

#[derive(Clone)]
pub enum TypeContext {
  FuncArg { func_name: String, param_name: String, param_idx: usize },
  FuncReturn { func_name: String },
  Binding { name: String },
  RecordField { field_name: String },
  MatchArm { arm_idx: usize },
  BinaryOp { op: String },
  General,
}

impl TypeError {
  pub fn to_message(&self, ta: &TypeArena) -> String {
    let expected = ta.display(self.expected);
    let found = ta.display(self.found);
    match &self.context {
      TypeContext::FuncArg { func_name, param_name, param_idx } => {
        format!("type mismatch in argument '{param_name}' (#{param_idx}) of '{func_name}'\n  expected: {expected}\n     found: {found}")
      },
      TypeContext::FuncReturn { func_name } => {
        format!("type mismatch in return type of '{func_name}'\n  expected: {expected}\n     found: {found}")
      },
      TypeContext::Binding { name } => {
        format!("type mismatch in binding '{name}'\n  expected: {expected}\n     found: {found}")
      },
      TypeContext::RecordField { field_name } => {
        format!("type mismatch in record field '{field_name}'\n  expected: {expected}\n     found: {found}")
      },
      TypeContext::MatchArm { arm_idx } => {
        format!("type mismatch in match arm #{arm_idx}\n  expected: {expected}\n     found: {found}")
      },
      TypeContext::BinaryOp { op } => {
        format!("type mismatch in '{op}' expression\n  expected: {expected}\n     found: {found}")
      },
      TypeContext::General => {
        format!("type mismatch\n  expected: {expected}\n     found: {found}")
      },
    }
  }

  pub fn help(&self, ta: &TypeArena) -> Option<String> {
    match (ta.get(self.expected), ta.get(self.found)) {
      (Type::Int, Type::Str) => Some("did you mean to pass a number?".into()),
      (Type::Str, Type::Int) => Some("did you mean to convert this to a string?".into()),
      (Type::Func { .. }, _) => Some("this value is not callable".into()),
      (Type::Maybe(_), t) | (t, Type::Maybe(_)) if !matches!(t, Type::Maybe(_)) => {
        Some("wrap with Some(...) to create a Maybe, or use ?? to unwrap with a default".into())
      },
      (Type::Result { .. }, t) | (t, Type::Result { .. }) if !matches!(t, Type::Result { .. }) => {
        Some("wrap with Ok(...) to create a Result, or use ^ to propagate errors".into())
      },
      (Type::List(_), t) | (t, Type::List(_)) if !matches!(t, Type::List(_)) => Some("to create a single-element list, use [value]".into()),
      (Type::Bool, Type::Int) | (Type::Int, Type::Bool) => Some("booleans and integers are not interchangeable in lx".into()),
      (Type::Record(expected_fields), Type::Record(found_fields)) => {
        let missing: Vec<String> =
          expected_fields.iter().filter(|(name, _)| !found_fields.iter().any(|(n, _)| n == name)).map(|(name, _)| name.to_string()).collect();
        if !missing.is_empty() {
          Some(format!("record is missing fields: {}", missing.join(", ")))
        } else {
          let extra: Vec<String> =
            found_fields.iter().filter(|(name, _)| !expected_fields.iter().any(|(n, _)| n == name)).map(|(name, _)| name.to_string()).collect();
          if !extra.is_empty() { Some(format!("record has unexpected fields: {}", extra.join(", "))) } else { None }
        }
      },
      _ => None,
    }
  }
}
