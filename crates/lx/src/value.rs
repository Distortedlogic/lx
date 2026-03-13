use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::ast::SExpr;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Value {
  Int(BigInt),
  Float(f64),
  Bool(bool),
  Str(Arc<str>),
  Unit,

  List(Arc<Vec<Value>>),
  Record(Arc<IndexMap<String, Value>>),
  Map(Arc<IndexMap<ValueKey, Value>>),
  Set(Arc<IndexSet<ValueKey>>),
  Tuple(Arc<Vec<Value>>),

  Func(LxFunc),
  BuiltinFunc(BuiltinFunc),

  Ok(Box<Value>),
  Err(Box<Value>),
  Some(Box<Value>),
  None,
}

#[derive(Debug, Clone)]
pub struct ValueKey(pub Value);

impl PartialEq for ValueKey {
  fn eq(&self, other: &Self) -> bool {
    self.0.structural_eq(&other.0)
  }
}

impl Eq for ValueKey {}

impl Hash for ValueKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash_value(state);
  }
}

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    self.structural_eq(other)
  }
}

impl Value {
  fn structural_eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Value::Int(a), Value::Int(b)) => a == b,
      (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
      (Value::Bool(a), Value::Bool(b)) => a == b,
      (Value::Str(a), Value::Str(b)) => a == b,
      (Value::Unit, Value::Unit) => true,
      (Value::List(a), Value::List(b)) => a == b,
      (Value::Tuple(a), Value::Tuple(b)) => a == b,
      (Value::Record(a), Value::Record(b)) => {
        if a.len() != b.len() {
          return false;
        }
        let mut a_sorted: Vec<_> = a.iter().collect();
        let mut b_sorted: Vec<_> = b.iter().collect();
        a_sorted.sort_by_key(|(k, _)| k.clone());
        b_sorted.sort_by_key(|(k, _)| k.clone());
        a_sorted.iter().zip(b_sorted.iter()).all(|((ak, av), (bk, bv))| ak == bk && av == bv)
      },
      (Value::Map(a), Value::Map(b)) => a == b,
      (Value::Set(a), Value::Set(b)) => a == b,
      (Value::Ok(a), Value::Ok(b)) => a == b,
      (Value::Err(a), Value::Err(b)) => a == b,
      (Value::Some(a), Value::Some(b)) => a == b,
      (Value::None, Value::None) => true,
      (Value::Func(_), _) | (_, Value::Func(_)) => false,
      (Value::BuiltinFunc(_), _) | (_, Value::BuiltinFunc(_)) => false,
      _ => false,
    }
  }

  fn hash_value<H: Hasher>(&self, state: &mut H) {
    std::mem::discriminant(self).hash(state);
    match self {
      Value::Int(n) => n.hash(state),
      Value::Float(f) => f.to_bits().hash(state),
      Value::Bool(b) => b.hash(state),
      Value::Str(s) => s.hash(state),
      Value::Unit => {},
      Value::List(items) | Value::Tuple(items) => {
        items.len().hash(state);
        for item in items.iter() {
          item.hash_value(state);
        }
      },
      Value::Record(fields) => {
        fields.len().hash(state);
        let mut pairs: Vec<_> = fields.iter().collect();
        pairs.sort_by_key(|(k, _)| k.clone());
        for (k, v) in pairs {
          k.hash(state);
          v.hash_value(state);
        }
      },
      Value::Map(entries) => {
        entries.len().hash(state);
        for (k, v) in entries.iter() {
          k.hash(state);
          v.hash_value(state);
        }
      },
      Value::Set(elems) => {
        elems.len().hash(state);
        for e in elems.iter() {
          e.hash(state);
        }
      },
      Value::Ok(v) | Value::Err(v) | Value::Some(v) => v.hash_value(state),
      Value::None => {},
      Value::Func(_) | Value::BuiltinFunc(_) => {},
    }
  }

  pub fn as_int(&self) -> Option<&BigInt> {
    match self {
      Value::Int(n) => Some(n),
      _ => Option::None,
    }
  }

  pub fn as_float(&self) -> Option<f64> {
    match self {
      Value::Float(f) => Some(*f),
      _ => Option::None,
    }
  }

  pub fn as_bool(&self) -> Option<bool> {
    match self {
      Value::Bool(b) => Some(*b),
      _ => Option::None,
    }
  }

  pub fn as_str(&self) -> Option<&str> {
    match self {
      Value::Str(s) => Some(s),
      _ => Option::None,
    }
  }

  pub fn as_list(&self) -> Option<&Arc<Vec<Value>>> {
    match self {
      Value::List(l) => Some(l),
      _ => Option::None,
    }
  }

  pub fn is_truthy_err(&self) -> bool {
    matches!(self, Value::Err(_) | Value::None)
  }

  pub fn type_name(&self) -> &'static str {
    match self {
      Value::Int(_) => "Int",
      Value::Float(_) => "Float",
      Value::Bool(_) => "Bool",
      Value::Str(_) => "Str",
      Value::Unit => "Unit",
      Value::List(_) => "List",
      Value::Record(_) => "Record",
      Value::Map(_) => "Map",
      Value::Set(_) => "Set",
      Value::Tuple(_) => "Tuple",
      Value::Func(_) | Value::BuiltinFunc(_) => "Func",
      Value::Ok(_) => "Ok",
      Value::Err(_) => "Err",
      Value::Some(_) => "Some",
      Value::None => "None",
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Value::Int(n) => write!(f, "{n}"),
      Value::Float(v) => write!(f, "{v}"),
      Value::Bool(b) => write!(f, "{b}"),
      Value::Str(s) => write!(f, "{s}"),
      Value::Unit => write!(f, "()"),
      Value::List(items) => {
        write!(f, "[")?;
        for (i, item) in items.iter().enumerate() {
          if i > 0 {
            write!(f, " ")?;
          }
          write!(f, "{item}")?;
        }
        write!(f, "]")
      },
      Value::Tuple(items) => {
        write!(f, "(")?;
        for (i, item) in items.iter().enumerate() {
          if i > 0 {
            write!(f, " ")?;
          }
          write!(f, "{item}")?;
        }
        write!(f, ")")
      },
      Value::Record(fields) => {
        write!(f, "{{")?;
        for (i, (k, v)) in fields.iter().enumerate() {
          if i > 0 {
            write!(f, "  ")?;
          }
          write!(f, "{k}: {v}")?;
        }
        write!(f, "}}")
      },
      Value::Map(entries) => {
        write!(f, "Map{{")?;
        for (i, (k, v)) in entries.iter().enumerate() {
          if i > 0 {
            write!(f, "  ")?;
          }
          write!(f, "{}: {v}", k.0)?;
        }
        write!(f, "}}")
      },
      Value::Set(elems) => {
        write!(f, "Set{{")?;
        for (i, e) in elems.iter().enumerate() {
          if i > 0 {
            write!(f, " ")?;
          }
          write!(f, "{}", e.0)?;
        }
        write!(f, "}}")
      },
      Value::Func(_) => write!(f, "<func>"),
      Value::BuiltinFunc(b) => write!(f, "<builtin {}/{}>", b.name, b.arity),
      Value::Ok(v) => write!(f, "Ok {v}"),
      Value::Err(v) => write!(f, "Err {v}"),
      Value::Some(v) => write!(f, "Some {v}"),
      Value::None => write!(f, "None"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct LxFunc {
  pub params: Vec<String>,
  pub defaults: Vec<Option<Value>>,
  pub body: Arc<SExpr>,
  pub closure: Env,
  pub arity: usize,
  pub applied: Vec<Value>,
}

pub type BuiltinFn = fn(&[Value], Span) -> Result<Value, LxError>;

#[derive(Clone)]
pub struct BuiltinFunc {
  pub name: &'static str,
  pub arity: usize,
  pub func: BuiltinFn,
  pub applied: Vec<Value>,
}

impl fmt::Debug for BuiltinFunc {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<builtin {}/{}>", self.name, self.arity)
  }
}
