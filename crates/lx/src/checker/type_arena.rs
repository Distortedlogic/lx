use std::collections::HashMap;
use std::fmt;

use ena::unify::UnifyKey;
use itertools::Itertools;
use la_arena::{Arena, Idx};

use super::types::{Type, Variant};

pub type TypeId = Idx<Type>;

#[derive(Clone)]
pub struct TypeArena {
  arena: Arena<Type>,
  intern_map: HashMap<Type, TypeId>,
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
    let mut intern_map = HashMap::new();
    let int_id = arena.alloc(Type::Int);
    intern_map.insert(Type::Int, int_id);
    let float_id = arena.alloc(Type::Float);
    intern_map.insert(Type::Float, float_id);
    let bool_id = arena.alloc(Type::Bool);
    intern_map.insert(Type::Bool, bool_id);
    let str_id = arena.alloc(Type::Str);
    intern_map.insert(Type::Str, str_id);
    let unit_id = arena.alloc(Type::Unit);
    intern_map.insert(Type::Unit, unit_id);
    let bytes_id = arena.alloc(Type::Bytes);
    intern_map.insert(Type::Bytes, bytes_id);
    let unknown_id = arena.alloc(Type::Unknown);
    intern_map.insert(Type::Unknown, unknown_id);
    let todo_id = arena.alloc(Type::Todo);
    intern_map.insert(Type::Todo, todo_id);
    let error_id = arena.alloc(Type::Error);
    intern_map.insert(Type::Error, error_id);
    Self { arena, intern_map, int_id, float_id, bool_id, str_id, unit_id, bytes_id, unknown_id, todo_id, error_id }
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
    if let Some(&id) = self.intern_map.get(&ty) {
      return id;
    }
    let id = self.arena.alloc(ty.clone());
    self.intern_map.insert(ty, id);
    id
  }

  pub fn get(&self, id: TypeId) -> &Type {
    &self.arena[id]
  }

  pub fn display(&self, id: TypeId) -> String {
    TypeDisplay { arena: self, id }.to_string()
  }

  pub fn copy_type(&mut self, id: TypeId, source: &TypeArena) -> TypeId {
    match source.get(id).clone() {
      Type::Int => self.int(),
      Type::Float => self.float(),
      Type::Bool => self.bool(),
      Type::Str => self.str(),
      Type::Unit => self.unit(),
      Type::Bytes => self.bytes(),
      Type::Unknown => self.unknown(),
      Type::Todo => self.todo(),
      Type::Error => self.error(),
      Type::List(inner) => {
        let inner = self.copy_type(inner, source);
        self.alloc(Type::List(inner))
      },
      Type::Map { key, value } => {
        let key = self.copy_type(key, source);
        let value = self.copy_type(value, source);
        self.alloc(Type::Map { key, value })
      },
      Type::Func { param, ret } => {
        let param = self.copy_type(param, source);
        let ret = self.copy_type(ret, source);
        self.alloc(Type::Func { param, ret })
      },
      Type::Tuple(elems) => {
        let elems = elems.iter().map(|e| self.copy_type(*e, source)).collect();
        self.alloc(Type::Tuple(elems))
      },
      Type::Record(fields) => {
        let fields = fields.iter().map(|(n, t)| (*n, self.copy_type(*t, source))).collect();
        self.alloc(Type::Record(fields))
      },
      Type::Result { ok, err } => {
        let ok = self.copy_type(ok, source);
        let err = self.copy_type(err, source);
        self.alloc(Type::Result { ok, err })
      },
      Type::Maybe(inner) => {
        let inner = self.copy_type(inner, source);
        self.alloc(Type::Maybe(inner))
      },
      Type::Union { name, variants } => {
        let variants = variants
          .iter()
          .map(|v| {
            let fields = v.fields.iter().map(|f| self.copy_type(*f, source)).collect();
            Variant { name: v.name, fields }
          })
          .collect();
        self.alloc(Type::Union { name, variants })
      },
      Type::Var(_) => self.unknown(),
      Type::Param { name, bound } => {
        let bound = bound.map(|b| self.copy_type(b, source));
        self.alloc(Type::Param { name, bound })
      },
    }
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
      Type::Param { name, bound: None } => write!(f, "{name}"),
      Type::Param { name, bound: Some(b) } => {
        write!(f, "{name}: {}", self.arena.display(*b))
      },
      Type::Var(key) => write!(f, "t{}", key.index()),
      Type::Unknown => write!(f, "?"),
      Type::Todo => write!(f, "<todo>"),
      Type::Error => write!(f, "<error>"),
    }
  }
}
