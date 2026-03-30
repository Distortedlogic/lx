use std::sync::Arc;

use dashmap::{DashMap, DashSet};

use crate::value::LxVal;
use lx_span::sym::{Sym, intern};

#[derive(Debug, Clone, Default)]
pub struct Env {
  bindings: DashMap<Sym, LxVal>,
  mutables: DashSet<Sym>,
  parent: Option<Arc<Env>>,
}

impl Env {
  pub fn with_parent(parent: Arc<Env>) -> Self {
    Self { parent: Some(parent), ..Self::default() }
  }

  pub fn child(self: &Arc<Self>) -> Self {
    Self::with_parent(Arc::clone(self))
  }

  pub fn bind(&self, name: Sym, value: LxVal) {
    self.bindings.insert(name, value);
  }

  pub fn bind_mut(&self, name: Sym, value: LxVal) {
    self.mutables.insert(name);
    self.bindings.insert(name, value);
  }

  pub fn bind_with_mutability(&self, name: Sym, value: LxVal, mutable: bool) {
    if mutable {
      self.bind_mut(name, value);
    } else {
      self.bind(name, value);
    }
  }

  pub fn bind_params(&self, params: &[Sym], applied: &[LxVal], defaults: &[Option<LxVal>]) {
    for (i, &sym) in params.iter().enumerate() {
      if i < applied.len() {
        self.bind(sym, applied[i].clone());
      } else if let Some(Some(def)) = defaults.get(i) {
        self.bind(sym, def.clone());
      }
    }
  }

  pub fn bind_str(&self, name: &str, value: LxVal) {
    self.bindings.insert(intern(name), value);
  }

  pub fn reassign(&self, name: Sym, value: LxVal) -> Result<(), String> {
    if self.bindings.contains_key(&name) {
      if self.mutables.contains(&name) {
        self.bindings.insert(name, value);
        return Ok(());
      }
      return Err(format!("cannot reassign immutable binding '{}'", name.as_str()));
    }
    if let Some(parent) = &self.parent {
      return parent.reassign(name, value);
    }
    Err(format!("undefined variable '{}'", name.as_str()))
  }

  pub fn get(&self, name: Sym) -> Option<LxVal> {
    if let Some(v) = self.bindings.get(&name) {
      return Some(v.value().clone());
    }
    self.parent.as_ref().and_then(|p| p.get(name))
  }

  pub fn get_str(&self, name: &str) -> Option<LxVal> {
    self.get(intern(name))
  }

  pub fn has_mut(&self, name: Sym) -> bool {
    if self.bindings.contains_key(&name) {
      return self.mutables.contains(&name);
    }
    self.parent.as_ref().is_some_and(|p| p.has_mut(name))
  }
}
