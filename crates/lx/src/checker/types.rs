use crate::sym::Sym;
use ena::unify::{NoError, UnifyKey, UnifyValue};

use super::type_arena::TypeId;

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
pub struct TypeVarValue(pub Option<TypeId>);

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

  List(TypeId),
  Map { key: TypeId, value: TypeId },
  Record(Vec<(Sym, TypeId)>),
  Tuple(Vec<TypeId>),

  Func { param: TypeId, ret: TypeId },
  Result { ok: TypeId, err: TypeId },
  Maybe(TypeId),

  Union { name: Sym, variants: Vec<Variant> },

  Var(TypeVarKey),
  Unknown,
  Todo,
  Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
  pub name: Sym,
  pub fields: Vec<TypeId>,
}
