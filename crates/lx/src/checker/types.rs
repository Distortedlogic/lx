use std::collections::HashMap;
use std::fmt;

use crate::sym::Sym;
use itertools::Itertools;

pub type TypeVarId = u32;

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

  Func { param: Box<Type>, ret: Box<Type> },
  Result { ok: Box<Type>, err: Box<Type> },
  Maybe(Box<Type>),

  Union { name: Sym, variants: Vec<Variant> },

  Var(TypeVarId),
  Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
  pub name: Sym,
  pub fields: Vec<Type>,
}

#[derive(Default)]
pub struct UnificationTable {
  bindings: Vec<Option<Type>>,
}

impl UnificationTable {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn fresh_var(&mut self) -> Type {
    let id = self.bindings.len() as TypeVarId;
    self.bindings.push(None);
    Type::Var(id)
  }

  pub fn resolve(&self, ty: &Type) -> Type {
    match ty {
      Type::Var(id) => match self.bindings.get(*id as usize) {
        Some(Some(bound)) => self.resolve(bound),
        _ => ty.clone(),
      },
      Type::List(inner) => Type::List(Box::new(self.resolve(inner))),
      Type::Map { key, value } => Type::Map { key: Box::new(self.resolve(key)), value: Box::new(self.resolve(value)) },
      Type::Record(fields) => Type::Record(fields.iter().map(|(n, t)| (*n, self.resolve(t))).collect()),
      Type::Tuple(elems) => Type::Tuple(elems.iter().map(|t| self.resolve(t)).collect()),
      Type::Func { param, ret } => Type::Func { param: Box::new(self.resolve(param)), ret: Box::new(self.resolve(ret)) },
      Type::Result { ok, err } => Type::Result { ok: Box::new(self.resolve(ok)), err: Box::new(self.resolve(err)) },
      Type::Maybe(inner) => Type::Maybe(Box::new(self.resolve(inner))),
      _ => ty.clone(),
    }
  }

  pub fn unify(&mut self, a: &Type, b: &Type) -> Result<Type, String> {
    let a = self.resolve(a);
    let b = self.resolve(b);
    match (&a, &b) {
      _ if a == b => Ok(a),
      (Type::Unknown, _) => Ok(b),
      (_, Type::Unknown) => Ok(a),
      (Type::Var(id), _) => {
        if self.occurs(*id, &b) {
          return Err(format!("infinite type: t{id} occurs in {b:?}"));
        }
        self.bindings[*id as usize] = Some(b.clone());
        Ok(b)
      },
      (_, Type::Var(id)) => {
        if self.occurs(*id, &a) {
          return Err(format!("infinite type: t{id} occurs in {a:?}"));
        }
        self.bindings[*id as usize] = Some(a.clone());
        Ok(a)
      },
      (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
      (Type::List(a_inner), Type::List(b_inner)) => {
        let inner = self.unify(a_inner, b_inner)?;
        Ok(Type::List(Box::new(inner)))
      },
      (Type::Tuple(a_elems), Type::Tuple(b_elems)) if a_elems.len() == b_elems.len() => {
        let elems: Result<Vec<_>, _> = a_elems.iter().zip(b_elems.iter()).map(|(a, b)| self.unify(a, b)).collect();
        Ok(Type::Tuple(elems?))
      },
      (Type::Func { param: ap, ret: ar }, Type::Func { param: bp, ret: br }) => {
        let param = self.unify(ap, bp)?;
        let ret = self.unify(ar, br)?;
        Ok(Type::Func { param: Box::new(param), ret: Box::new(ret) })
      },
      (Type::Result { ok: ao, err: ae }, Type::Result { ok: bo, err: be }) => {
        let ok = self.unify(ao, bo)?;
        let err = self.unify(ae, be)?;
        Ok(Type::Result { ok: Box::new(ok), err: Box::new(err) })
      },
      (Type::Maybe(a_inner), Type::Maybe(b_inner)) => {
        let inner = self.unify(a_inner, b_inner)?;
        Ok(Type::Maybe(Box::new(inner)))
      },
      (Type::Record(a_fields), Type::Record(b_fields)) => self.unify_records(a_fields, b_fields),
      _ => Err(format!("type mismatch: expected {a}, got {b}")),
    }
  }

  fn unify_records(&mut self, a: &[(Sym, Type)], b: &[(Sym, Type)]) -> Result<Type, String> {
    let a_map: HashMap<Sym, &Type> = a.iter().map(|(n, t)| (*n, t)).collect();
    let b_map: HashMap<Sym, &Type> = b.iter().map(|(n, t)| (*n, t)).collect();
    let mut fields = Vec::new();
    for (name, a_ty) in &a_map {
      if let Some(b_ty) = b_map.get(name) {
        fields.push((*name, self.unify(a_ty, b_ty)?));
      } else {
        fields.push((*name, (*a_ty).clone()));
      }
    }
    for (name, b_ty) in &b_map {
      if !a_map.contains_key(name) {
        fields.push((*name, (*b_ty).clone()));
      }
    }
    Ok(Type::Record(fields))
  }

  fn occurs(&self, var: TypeVarId, ty: &Type) -> bool {
    match ty {
      Type::Var(id) => {
        if *id == var {
          return true;
        }
        match self.bindings.get(*id as usize) {
          Some(Some(bound)) => self.occurs(var, bound),
          _ => false,
        }
      },
      Type::List(inner) | Type::Maybe(inner) => self.occurs(var, inner),
      Type::Map { key, value } => self.occurs(var, key) || self.occurs(var, value),
      Type::Func { param, ret } => self.occurs(var, param) || self.occurs(var, ret),
      Type::Result { ok, err } => self.occurs(var, ok) || self.occurs(var, err),
      Type::Tuple(elems) => elems.iter().any(|t| self.occurs(var, t)),
      Type::Record(fields) => fields.iter().any(|(_, t)| self.occurs(var, t)),
      _ => false,
    }
  }
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
      Type::Func { param, ret } => write!(f, "{param} -> {ret}"),
      Type::Result { ok, err } => write!(f, "{ok} ^ {err}"),
      Type::Maybe(inner) => write!(f, "Maybe {inner}"),
      Type::Union { name, .. } => write!(f, "{name}"),
      Type::Var(id) => write!(f, "t{id}"),
      Type::Unknown => write!(f, "?"),
    }
  }
}
