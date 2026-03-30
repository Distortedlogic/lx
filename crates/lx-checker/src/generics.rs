use std::collections::HashMap;

use lx_span::sym::Sym;

use super::Checker;
use super::type_arena::TypeId;
use super::types::{Type, Variant};

impl Checker<'_> {
  pub(crate) fn push_generic_scope(&mut self, params: &[(Sym, Option<TypeId>)]) {
    let mut scope = HashMap::new();
    for (name, bound) in params {
      let ty = self.type_arena.alloc(Type::Param { name: *name, bound: *bound });
      scope.insert(*name, ty);
    }
    self.generic_scope.push(scope);
  }

  pub(crate) fn pop_generic_scope(&mut self) {
    self.generic_scope.pop();
  }

  pub(crate) fn lookup_type_param(&self, name: Sym) -> Option<TypeId> {
    for scope in self.generic_scope.iter().rev() {
      if let Some(&ty) = scope.get(&name) {
        return Some(ty);
      }
    }
    None
  }

  pub(crate) fn collect_params(&self, ty: TypeId) -> Vec<Sym> {
    let mut params = Vec::new();
    self.collect_params_inner(ty, &mut params);
    params
  }

  fn collect_params_inner(&self, ty: TypeId, out: &mut Vec<Sym>) {
    match self.type_arena.get(ty).clone() {
      Type::Param { name, bound } => {
        if !out.contains(&name) {
          out.push(name);
        }
        if let Some(b) = bound {
          self.collect_params_inner(b, out);
        }
      },
      Type::Func { param, ret } => {
        self.collect_params_inner(param, out);
        self.collect_params_inner(ret, out);
      },
      Type::List(inner) | Type::Maybe(inner) => self.collect_params_inner(inner, out),
      Type::Map { key, value } | Type::Result { ok: key, err: value } => {
        self.collect_params_inner(key, out);
        self.collect_params_inner(value, out);
      },
      Type::Tuple(elems) => {
        for e in &elems {
          self.collect_params_inner(*e, out);
        }
      },
      Type::Record(fields) => {
        for (_, t) in &fields {
          self.collect_params_inner(*t, out);
        }
      },
      Type::Union { variants, .. } => {
        for v in &variants {
          for f in &v.fields {
            self.collect_params_inner(*f, out);
          }
        }
      },
      _ => {},
    }
  }

  pub(crate) fn substitute(&mut self, ty: TypeId, subst: &HashMap<Sym, TypeId>) -> TypeId {
    match self.type_arena.get(ty).clone() {
      Type::Param { name, .. } => {
        if let Some(&mapped) = subst.get(&name) {
          return mapped;
        }
        ty
      },
      Type::List(inner) => {
        let new_inner = self.substitute(inner, subst);
        if new_inner == inner {
          return ty;
        }
        self.type_arena.alloc(Type::List(new_inner))
      },
      Type::Map { key, value } => {
        let new_key = self.substitute(key, subst);
        let new_value = self.substitute(value, subst);
        if new_key == key && new_value == value {
          return ty;
        }
        self.type_arena.alloc(Type::Map { key: new_key, value: new_value })
      },
      Type::Func { param, ret } => {
        let new_param = self.substitute(param, subst);
        let new_ret = self.substitute(ret, subst);
        if new_param == param && new_ret == ret {
          return ty;
        }
        self.type_arena.alloc(Type::Func { param: new_param, ret: new_ret })
      },
      Type::Tuple(elems) => {
        let new_elems: Vec<_> = elems.iter().map(|e| self.substitute(*e, subst)).collect();
        if new_elems == elems {
          return ty;
        }
        self.type_arena.alloc(Type::Tuple(new_elems))
      },
      Type::Record(fields) => {
        let new_fields: Vec<_> = fields.iter().map(|(n, t)| (*n, self.substitute(*t, subst))).collect();
        if new_fields == fields {
          return ty;
        }
        self.type_arena.alloc(Type::Record(new_fields))
      },
      Type::Result { ok, err } => {
        let new_ok = self.substitute(ok, subst);
        let new_err = self.substitute(err, subst);
        if new_ok == ok && new_err == err {
          return ty;
        }
        self.type_arena.alloc(Type::Result { ok: new_ok, err: new_err })
      },
      Type::Maybe(inner) => {
        let new_inner = self.substitute(inner, subst);
        if new_inner == inner {
          return ty;
        }
        self.type_arena.alloc(Type::Maybe(new_inner))
      },
      Type::Union { name, variants } => {
        let new_variants: Vec<_> = variants
          .iter()
          .map(|v| {
            let new_fields: Vec<_> = v.fields.iter().map(|f| self.substitute(*f, subst)).collect();
            Variant { name: v.name, fields: new_fields }
          })
          .collect();
        if new_variants == variants {
          return ty;
        }
        self.type_arena.alloc(Type::Union { name, variants: new_variants })
      },
      _ => ty,
    }
  }
}
