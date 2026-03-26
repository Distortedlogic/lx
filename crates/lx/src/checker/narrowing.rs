use std::collections::HashMap;

use crate::ast::Pattern;
use crate::sym::Sym;

use super::type_arena::{TypeArena, TypeId};
use super::types::Type;

pub struct NarrowingEnv {
  stack: Vec<HashMap<Sym, TypeId>>,
}

impl NarrowingEnv {
  pub fn new() -> Self {
    Self { stack: vec![] }
  }

  pub fn push(&mut self) {
    self.stack.push(HashMap::new());
  }

  pub fn pop(&mut self) {
    self.stack.pop();
  }

  pub fn narrow(&mut self, name: Sym, ty: TypeId) {
    if let Some(top) = self.stack.last_mut() {
      top.insert(name, ty);
    }
  }

  pub fn lookup(&self, name: Sym) -> Option<TypeId> {
    for scope in self.stack.iter().rev() {
      if let Some(&ty) = scope.get(&name) {
        return Some(ty);
      }
    }
    None
  }
}

pub fn compute_narrowed_type(pattern: &Pattern, scrutinee_type: TypeId, type_arena: &mut TypeArena, resolved_type: &Type) -> TypeId {
  match (pattern, resolved_type) {
    (Pattern::Constructor(ctor), Type::Union { name, variants }) => {
      if let Some(matched) = variants.iter().find(|v| v.name == ctor.name) {
        type_arena.alloc(Type::Union { name: *name, variants: vec![matched.clone()] })
      } else {
        scrutinee_type
      }
    },
    _ => scrutinee_type,
  }
}
