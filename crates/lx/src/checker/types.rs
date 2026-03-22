use std::fmt;

use crate::sym::Sym;
use ena::unify::{NoError, UnifyKey, UnifyValue};
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVarKey(u32);

impl UnifyKey for TypeVarKey {
  type Value = TypeVarValue;

  fn index(&self) -> u32 {
    self.0
  }

  fn from_index(u: u32) -> Self {
    Self(u)
  }

  fn tag() -> &'static str {
    "TypeVarKey"
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeVarValue(pub Option<Type>);

impl UnifyValue for TypeVarValue {
  type Error = NoError;

  fn unify_values(a: &Self, b: &Self) -> Result<Self, NoError> {
    match (&a.0, &b.0) {
      (None, None) => Ok(TypeVarValue(None)),
      (Some(_), None) => Ok(a.clone()),
      (None, Some(_)) => Ok(b.clone()),
      (Some(_), Some(_)) => Ok(a.clone()),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
  Int,
  Float,
  Bool,
  Str,
  Unit,
  Bytes,

  List(Box<Type>),
  Map { key: Box<Type>, value: Box<Type> },
  Record(Vec<(Sym, Type)>),
  Tuple(Vec<Type>),

  Func { params: Vec<Type>, ret: Box<Type> },
  Result { ok: Box<Type>, err: Box<Type> },
  Maybe(Box<Type>),

  Union { name: Sym, variants: Vec<Variant> },

  Var(TypeVarKey),
  Unknown,
  Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
  pub name: Sym,
  pub fields: Vec<Type>,
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Type::Int => write!(f, "Int"),
      Type::Float => write!(f, "Float"),
      Type::Bool => write!(f, "Bool"),
      Type::Str => write!(f, "Str"),
      Type::Unit => write!(f, "()"),
      Type::Bytes => write!(f, "Bytes"),
      Type::List(inner) => write!(f, "[{inner}]"),
      Type::Map { key, value } => write!(f, "%{{{key}: {value}}}"),
      Type::Record(fields) => {
        write!(f, "{{{}}}", fields.iter().format_with("  ", |(n, t), g| g(&format_args!("{n}: {t}"))))
      },
      Type::Tuple(elems) => write!(f, "({})", elems.iter().format(", ")),
      Type::Func { params, ret } => {
        let params_str = params.iter().format(", ");
        write!(f, "({params_str}) -> {ret}")
      },
      Type::Result { ok, err } => write!(f, "{ok} ^ {err}"),
      Type::Maybe(inner) => write!(f, "Maybe {inner}"),
      Type::Union { name, .. } => write!(f, "{name}"),
      Type::Var(key) => write!(f, "t{}", key.index()),
      Type::Unknown => write!(f, "?"),
      Type::Error => write!(f, "<error>"),
    }
  }
}
