use std::collections::HashMap;

use ena::unify::{InPlaceUnificationTable, UnifyKey};
use miette::SourceSpan;

use crate::sym::Sym;

use super::types::{Type, TypeVarKey, TypeVarValue};

pub struct UnificationTable {
  table: InPlaceUnificationTable<TypeVarKey>,
}

impl Default for UnificationTable {
  fn default() -> Self {
    Self::new()
  }
}

impl UnificationTable {
  pub fn new() -> Self {
    Self { table: InPlaceUnificationTable::new() }
  }

  pub fn fresh_var(&mut self) -> Type {
    let key = self.table.new_key(TypeVarValue(None));
    Type::Var(key)
  }

  pub fn resolve(&mut self, ty: &Type) -> Type {
    match ty {
      Type::Var(key) => match self.table.probe_value(*key) {
        TypeVarValue(Some(bound)) => self.resolve(&bound),
        TypeVarValue(None) => ty.clone(),
      },
      Type::List(inner) => Type::List(Box::new(self.resolve(inner))),
      Type::Map { key, value } => Type::Map { key: Box::new(self.resolve(key)), value: Box::new(self.resolve(value)) },
      Type::Record(fields) => Type::Record(fields.iter().map(|(n, t)| (*n, self.resolve(t))).collect()),
      Type::Tuple(elems) => Type::Tuple(elems.iter().map(|t| self.resolve(t)).collect()),
      Type::Func { params, ret } => Type::Func { params: params.iter().map(|p| self.resolve(p)).collect(), ret: Box::new(self.resolve(ret)) },
      Type::Result { ok, err } => Type::Result { ok: Box::new(self.resolve(ok)), err: Box::new(self.resolve(err)) },
      Type::Maybe(inner) => Type::Maybe(Box::new(self.resolve(inner))),
      Type::Error => Type::Error,
      _ => ty.clone(),
    }
  }

  pub fn unify(&mut self, a: &Type, b: &Type) -> Result<Type, String> {
    let a = self.resolve(a);
    let b = self.resolve(b);
    match (&a, &b) {
      _ if a == b => Ok(a),
      (Type::Error, _) | (_, Type::Error) => Ok(Type::Error),
      (Type::Unknown, _) => Ok(b),
      (_, Type::Unknown) => Ok(a),
      (Type::Var(id), _) => {
        if self.occurs(*id, &b) {
          return Err(format!("infinite type: t{} occurs in {b:?}", id.index()));
        }
        self.table.union_value(*id, TypeVarValue(Some(b.clone())));
        Ok(b)
      },
      (_, Type::Var(id)) => {
        if self.occurs(*id, &a) {
          return Err(format!("infinite type: t{} occurs in {a:?}", id.index()));
        }
        self.table.union_value(*id, TypeVarValue(Some(a.clone())));
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
      (Type::Func { params: ap, ret: ar }, Type::Func { params: bp, ret: br }) if ap.len() == bp.len() => {
        let params: Result<Vec<_>, _> = ap.iter().zip(bp.iter()).map(|(a, b)| self.unify(a, b)).collect();
        let ret = self.unify(ar, br)?;
        Ok(Type::Func { params: params?, ret: Box::new(ret) })
      },
      (Type::Func { params: ap, .. }, Type::Func { params: bp, .. }) => {
        Err(format!("function parameter count mismatch: expected {}, got {}", ap.len(), bp.len()))
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

  pub fn unify_with_context(&mut self, a: &Type, b: &Type, ctx: TypeContext) -> Result<Type, Box<TypeError>> {
    self.unify(a, b).map_err(|_| Box::new(TypeError { expected: a.clone(), found: b.clone(), context: ctx, expected_origin: None }))
  }

  fn occurs(&mut self, var: TypeVarKey, ty: &Type) -> bool {
    match ty {
      Type::Var(key) => {
        if *key == var {
          return true;
        }
        match self.table.probe_value(*key) {
          TypeVarValue(Some(bound)) => self.occurs(var, &bound),
          TypeVarValue(None) => false,
        }
      },
      Type::List(inner) | Type::Maybe(inner) => self.occurs(var, inner),
      Type::Map { key, value } => self.occurs(var, key) || self.occurs(var, value),
      Type::Func { params, ret } => params.iter().any(|p| self.occurs(var, p)) || self.occurs(var, ret),
      Type::Result { ok, err } => self.occurs(var, ok) || self.occurs(var, err),
      Type::Tuple(elems) => elems.iter().any(|t| self.occurs(var, t)),
      Type::Record(fields) => fields.iter().any(|(_, t)| self.occurs(var, t)),
      Type::Error => false,
      _ => false,
    }
  }
}

#[derive(Clone)]
pub struct TypeError {
  pub expected: Type,
  pub found: Type,
  pub context: TypeContext,
  pub expected_origin: Option<SourceSpan>,
}

#[derive(Clone)]
pub enum TypeContext {
  FuncArg { func_name: String, param_name: String, param_idx: usize },
  FuncReturn { func_name: String },
  Binding { name: String },
  RecordField { field_name: String },
  ListElement { index: usize },
  MatchArm { arm_idx: usize },
  BinaryOp { op: String },
  General,
}

impl TypeError {
  pub fn to_message(&self) -> String {
    let expected = &self.expected;
    let found = &self.found;
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
      TypeContext::ListElement { index } => {
        format!("type mismatch in list element #{index}\n  expected: {expected}\n     found: {found}")
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

  pub fn help(&self) -> Option<String> {
    match (&self.expected, &self.found) {
      (Type::Int, Type::Str) => Some("did you mean to pass a number?".into()),
      (Type::Str, Type::Int) => Some("did you mean to convert this to a string?".into()),
      (Type::Func { .. }, _) => Some("this value is not callable".into()),
      _ => None,
    }
  }
}
