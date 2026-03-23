use std::collections::HashMap;

use miette::SourceSpan;

use super::type_arena::TypeId;
use crate::sym::Sym;

pub struct SymbolTable {
  scopes: Vec<Scope>,
  current: usize,
}

pub struct Definition {
  pub kind: DefKind,
  pub span: SourceSpan,
  pub ty: Option<TypeId>,
}

#[derive(Clone, Copy)]
pub enum DefKind {
  Binding,
  FuncParam,
  PatternBind,
  Import,
  TypeDef,
  TraitDef,
  ClassDef,
  WithBinding,
  ResourceBinding,
}

struct Scope {
  parent: Option<usize>,
  bindings: HashMap<Sym, Definition>,
}

impl SymbolTable {
  pub fn new() -> Self {
    Self { scopes: vec![Scope { parent: None, bindings: HashMap::new() }], current: 0 }
  }

  pub fn push_scope(&mut self) {
    let idx = self.scopes.len();
    self.scopes.push(Scope { parent: Some(self.current), bindings: HashMap::new() });
    self.current = idx;
  }

  pub fn pop_scope(&mut self) {
    if let Some(parent) = self.scopes[self.current].parent {
      self.current = parent;
    }
  }

  pub fn define(&mut self, name: Sym, kind: DefKind, span: SourceSpan) {
    self.scopes[self.current].bindings.insert(name, Definition { kind, span, ty: None });
  }

  pub fn set_type(&mut self, name: Sym, ty: TypeId) {
    let mut scope_idx = self.current;
    loop {
      if let Some(def) = self.scopes[scope_idx].bindings.get_mut(&name) {
        def.ty = Some(ty);
        return;
      }
      match self.scopes[scope_idx].parent {
        Some(parent) => scope_idx = parent,
        None => break,
      }
    }
    panic!("set_type called for undefined name: {name}");
  }

  pub fn lookup_type(&self, name: Sym) -> Option<TypeId> {
    let mut scope_idx = self.current;
    loop {
      if let Some(def) = self.scopes[scope_idx].bindings.get(&name)
        && let Some(ty) = def.ty
      {
        return Some(ty);
      }
      match self.scopes[scope_idx].parent {
        Some(parent) => scope_idx = parent,
        None => return None,
      }
    }
  }

  pub fn resolve(&self, name: Sym) -> Option<&Definition> {
    let mut scope_idx = self.current;
    loop {
      if let Some(def) = self.scopes[scope_idx].bindings.get(&name) {
        return Some(def);
      }
      match self.scopes[scope_idx].parent {
        Some(parent) => scope_idx = parent,
        None => return None,
      }
    }
  }
}
