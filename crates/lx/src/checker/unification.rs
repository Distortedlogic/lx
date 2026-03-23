use std::collections::HashMap;

use ena::unify::{InPlaceUnificationTable, UnifyKey};
use miette::SourceSpan;

use crate::sym::Sym;

use super::type_arena::{TypeArena, TypeId};
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

  pub fn fresh_var(&mut self, ta: &mut TypeArena) -> TypeId {
    let key = self.table.new_key(TypeVarValue(None));
    ta.alloc(Type::Var(key))
  }

  pub fn resolve(&mut self, id: TypeId, ta: &TypeArena) -> TypeId {
    match ta.get(id) {
      Type::Var(key) => match self.table.probe_value(*key) {
        TypeVarValue(Some(bound)) => self.resolve(bound, ta),
        TypeVarValue(None) => id,
      },
      _ => id,
    }
  }

  pub fn deep_resolve(&mut self, id: TypeId, ta: &mut TypeArena) -> TypeId {
    let id = self.resolve(id, ta);
    match ta.get(id).clone() {
      Type::List(inner) => {
        let inner = self.deep_resolve(inner, ta);
        ta.alloc(Type::List(inner))
      },
      Type::Map { key, value } => {
        let key = self.deep_resolve(key, ta);
        let value = self.deep_resolve(value, ta);
        ta.alloc(Type::Map { key, value })
      },
      Type::Record(fields) => {
        let fields: Vec<_> = fields.iter().map(|(n, t)| (*n, self.deep_resolve(*t, ta))).collect();
        ta.alloc(Type::Record(fields))
      },
      Type::Tuple(elems) => {
        let elems: Vec<_> = elems.iter().map(|t| self.deep_resolve(*t, ta)).collect();
        ta.alloc(Type::Tuple(elems))
      },
      Type::Func { param, ret } => {
        let param = self.deep_resolve(param, ta);
        let ret = self.deep_resolve(ret, ta);
        ta.alloc(Type::Func { param, ret })
      },
      Type::Result { ok, err } => {
        let ok = self.deep_resolve(ok, ta);
        let err = self.deep_resolve(err, ta);
        ta.alloc(Type::Result { ok, err })
      },
      Type::Maybe(inner) => {
        let inner = self.deep_resolve(inner, ta);
        ta.alloc(Type::Maybe(inner))
      },
      _ => id,
    }
  }

  pub fn unify(&mut self, a: TypeId, b: TypeId, ta: &mut TypeArena) -> Result<TypeId, String> {
    let a = self.resolve(a, ta);
    let b = self.resolve(b, ta);
    if a == b {
      return Ok(a);
    }
    let a_ty = ta.get(a).clone();
    let b_ty = ta.get(b).clone();
    match (&a_ty, &b_ty) {
      (Type::Error, _) | (_, Type::Error) => Ok(ta.error()),
      (Type::Unknown, _) | (Type::Todo, _) => Ok(b),
      (_, Type::Unknown) | (_, Type::Todo) => Ok(a),
      (Type::Var(id), _) => {
        if self.occurs(*id, b, ta) {
          return Err(format!("infinite type: t{} occurs in {:?}", id.index(), b_ty));
        }
        self.table.union_value(*id, TypeVarValue(Some(b)));
        Ok(b)
      },
      (_, Type::Var(id)) => {
        if self.occurs(*id, a, ta) {
          return Err(format!("infinite type: t{} occurs in {:?}", id.index(), a_ty));
        }
        self.table.union_value(*id, TypeVarValue(Some(a)));
        Ok(a)
      },
      (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(ta.float()),
      (Type::List(a_inner), Type::List(b_inner)) => {
        let a_inner = *a_inner;
        let b_inner = *b_inner;
        let inner = self.unify(a_inner, b_inner, ta)?;
        Ok(ta.alloc(Type::List(inner)))
      },
      (Type::Tuple(a_elems), Type::Tuple(b_elems)) if a_elems.len() == b_elems.len() => {
        let pairs: Vec<_> = a_elems.iter().zip(b_elems.iter()).map(|(a, b)| (*a, *b)).collect();
        let elems: Result<Vec<_>, _> = pairs.into_iter().map(|(a, b)| self.unify(a, b, ta)).collect();
        Ok(ta.alloc(Type::Tuple(elems?)))
      },
      (Type::Func { param: ap, ret: ar }, Type::Func { param: bp, ret: br }) => {
        let (ap, ar, bp, br) = (*ap, *ar, *bp, *br);
        let param = self.unify(ap, bp, ta)?;
        let ret = self.unify(ar, br, ta)?;
        Ok(ta.alloc(Type::Func { param, ret }))
      },
      (Type::Result { ok: ao, err: ae }, Type::Result { ok: bo, err: be }) => {
        let (ao, ae, bo, be) = (*ao, *ae, *bo, *be);
        let ok = self.unify(ao, bo, ta)?;
        let err = self.unify(ae, be, ta)?;
        Ok(ta.alloc(Type::Result { ok, err }))
      },
      (Type::Maybe(a_inner), Type::Maybe(b_inner)) => {
        let (a_inner, b_inner) = (*a_inner, *b_inner);
        let inner = self.unify(a_inner, b_inner, ta)?;
        Ok(ta.alloc(Type::Maybe(inner)))
      },
      (Type::Record(a_fields), Type::Record(b_fields)) => {
        let a_fields = a_fields.clone();
        let b_fields = b_fields.clone();
        self.unify_records(&a_fields, &b_fields, ta)
      },
      _ => Err(format!("type mismatch: expected {}, got {}", ta.display(a), ta.display(b))),
    }
  }

  fn unify_records(&mut self, a: &[(Sym, TypeId)], b: &[(Sym, TypeId)], ta: &mut TypeArena) -> Result<TypeId, String> {
    let a_map: HashMap<Sym, TypeId> = a.iter().copied().collect();
    let b_map: HashMap<Sym, TypeId> = b.iter().copied().collect();
    let mut fields = Vec::new();
    for (name, a_ty) in &a_map {
      if let Some(b_ty) = b_map.get(name) {
        fields.push((*name, self.unify(*a_ty, *b_ty, ta)?));
      } else {
        fields.push((*name, *a_ty));
      }
    }
    for (name, b_ty) in &b_map {
      if !a_map.contains_key(name) {
        fields.push((*name, *b_ty));
      }
    }
    Ok(ta.alloc(Type::Record(fields)))
  }

  pub fn unify_with_context(&mut self, a: TypeId, b: TypeId, ctx: TypeContext, ta: &mut TypeArena) -> Result<TypeId, Box<TypeError>> {
    self.unify(a, b, ta).map_err(|_| Box::new(TypeError { expected: a, found: b, context: ctx, expected_origin: None }))
  }

  fn occurs(&mut self, var: TypeVarKey, ty_id: TypeId, ta: &TypeArena) -> bool {
    match ta.get(ty_id) {
      Type::Var(key) => {
        if *key == var {
          return true;
        }
        match self.table.probe_value(*key) {
          TypeVarValue(Some(bound)) => self.occurs(var, bound, ta),
          TypeVarValue(None) => false,
        }
      },
      Type::List(inner) | Type::Maybe(inner) => self.occurs(var, *inner, ta),
      Type::Map { key, value } => self.occurs(var, *key, ta) || self.occurs(var, *value, ta),
      Type::Func { param, ret } => {
        let (param, ret) = (*param, *ret);
        self.occurs(var, param, ta) || self.occurs(var, ret, ta)
      },
      Type::Result { ok, err } => self.occurs(var, *ok, ta) || self.occurs(var, *err, ta),
      Type::Tuple(elems) => {
        let elems: Vec<_> = elems.clone();
        elems.iter().any(|t| self.occurs(var, *t, ta))
      },
      Type::Record(fields) => {
        let fields: Vec<_> = fields.clone();
        fields.iter().any(|(_, t)| self.occurs(var, *t, ta))
      },
      Type::Error => false,
      _ => false,
    }
  }
}

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
      _ => None,
    }
  }
}
