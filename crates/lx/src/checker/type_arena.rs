use std::fmt;

use ena::unify::UnifyKey;
use itertools::Itertools;
use la_arena::{Arena, Idx};

use super::types::Type;

pub type TypeId = Idx<Type>;

pub struct TypeArena {
  arena: Arena<Type>,
  int_id: TypeId,
  float_id: TypeId,
  bool_id: TypeId,
  str_id: TypeId,
  unit_id: TypeId,
  bytes_id: TypeId,
  unknown_id: TypeId,
  todo_id: TypeId,
  error_id: TypeId,
}

impl Default for TypeArena {
  fn default() -> Self {
    Self::new()
  }
}

impl TypeArena {
  pub fn new() -> Self {
    let mut arena = Arena::new();
    let int_id = arena.alloc(Type::Int);
    let float_id = arena.alloc(Type::Float);
    let bool_id = arena.alloc(Type::Bool);
    let str_id = arena.alloc(Type::Str);
    let unit_id = arena.alloc(Type::Unit);
    let bytes_id = arena.alloc(Type::Bytes);
    let unknown_id = arena.alloc(Type::Unknown);
    let todo_id = arena.alloc(Type::Todo);
    let error_id = arena.alloc(Type::Error);
    Self { arena, int_id, float_id, bool_id, str_id, unit_id, bytes_id, unknown_id, todo_id, error_id }
  }

  pub fn int(&self) -> TypeId {
    self.int_id
  }

  pub fn float(&self) -> TypeId {
    self.float_id
  }

  pub fn bool(&self) -> TypeId {
    self.bool_id
  }

  pub fn str(&self) -> TypeId {
    self.str_id
  }

  pub fn unit(&self) -> TypeId {
    self.unit_id
  }

  pub fn bytes(&self) -> TypeId {
    self.bytes_id
  }

  pub fn unknown(&self) -> TypeId {
    self.unknown_id
  }

  pub fn todo(&self) -> TypeId {
    self.todo_id
  }

  pub fn error(&self) -> TypeId {
    self.error_id
  }

  pub fn alloc(&mut self, ty: Type) -> TypeId {
    self.arena.alloc(ty)
  }

  pub fn get(&self, id: TypeId) -> &Type {
    &self.arena[id]
  }

  pub fn display(&self, id: TypeId) -> String {
    TypeDisplay { arena: self, id }.to_string()
  }
}

struct TypeDisplay<'a> {
  arena: &'a TypeArena,
  id: TypeId,
}

impl fmt::Display for TypeDisplay<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.arena.get(self.id) {
      Type::Int => write!(f, "Int"),
      Type::Float => write!(f, "Float"),
      Type::Bool => write!(f, "Bool"),
      Type::Str => write!(f, "Str"),
      Type::Unit => write!(f, "()"),
      Type::Bytes => write!(f, "Bytes"),
      Type::List(inner) => write!(f, "[{}]", self.arena.display(*inner)),
      Type::Map { key, value } => {
        write!(f, "%{{{}: {}}}", self.arena.display(*key), self.arena.display(*value))
      },
      Type::Record(fields) => {
        let formatted = fields.iter().format_with("  ", |(n, t), g| g(&format_args!("{}: {}", n, self.arena.display(*t))));
        write!(f, "{{{formatted}}}")
      },
      Type::Tuple(elems) => {
        let formatted = elems.iter().map(|e| self.arena.display(*e)).join(", ");
        write!(f, "({formatted})")
      },
      Type::Func { param, ret } => {
        write!(f, "{} -> {}", self.arena.display(*param), self.arena.display(*ret))
      },
      Type::Result { ok, err } => {
        write!(f, "{} ^ {}", self.arena.display(*ok), self.arena.display(*err))
      },
      Type::Maybe(inner) => write!(f, "Maybe {}", self.arena.display(*inner)),
      Type::Union { name, .. } => write!(f, "{name}"),
      Type::Var(key) => write!(f, "t{}", key.index()),
      Type::Unknown => write!(f, "?"),
      Type::Todo => write!(f, "<todo>"),
      Type::Error => write!(f, "<error>"),
    }
  }
}
