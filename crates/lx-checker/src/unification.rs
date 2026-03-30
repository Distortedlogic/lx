use std::collections::HashMap;

use ena::unify::{InPlaceUnificationTable, UnifyKey};

use lx_span::sym::Sym;

use super::type_arena::{TypeArena, TypeId};
use super::type_error::{TypeContext, TypeError};
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
      Type::Param { name, bound: Some(b) } => {
        let b = self.deep_resolve(b, ta);
        ta.alloc(Type::Param { name, bound: Some(b) })
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
      (Type::Param { name: a_name, .. }, Type::Param { name: b_name, .. }) if a_name == b_name => Ok(a),
      (Type::Param { bound: Some(b_bound), .. }, _) => {
        let b_bound = *b_bound;
        self.unify(b_bound, b, ta)
      },
      (_, Type::Param { bound: Some(b_bound), .. }) => {
        let b_bound = *b_bound;
        self.unify(a, b_bound, ta)
      },
      (Type::Param { bound: None, .. }, _) => {
        let var = self.fresh_var(ta);
        self.unify(var, b, ta)
      },
      (_, Type::Param { bound: None, .. }) => {
        let var = self.fresh_var(ta);
        self.unify(a, var, ta)
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
      Type::Param { bound: Some(b), .. } => self.occurs(var, *b, ta),
      Type::Param { bound: None, .. } => false,
      Type::Error => false,
      _ => false,
    }
  }
}
