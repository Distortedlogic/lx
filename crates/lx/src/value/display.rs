use crate::sym::resolve;
use std::fmt;
use std::sync::Arc;

use itertools::Itertools;
use num_bigint::BigInt;

use super::{BuiltinFunc, LxVal};

impl From<&str> for LxVal {
  fn from(s: &str) -> Self {
    LxVal::Str(Arc::from(s))
  }
}

impl<T: Into<LxVal>> From<Vec<T>> for LxVal {
  fn from(items: Vec<T>) -> Self {
    LxVal::list(items.into_iter().map(Into::into).collect())
  }
}

impl TryFrom<&LxVal> for BigInt {
  type Error = &'static str;
  fn try_from(v: &LxVal) -> Result<Self, Self::Error> {
    match v {
      LxVal::Int(n) => Ok(n.clone()),
      _ => Result::Err("expected Int"),
    }
  }
}

impl TryFrom<&LxVal> for f64 {
  type Error = &'static str;
  fn try_from(v: &LxVal) -> Result<Self, Self::Error> {
    match v {
      LxVal::Float(f) => Ok(*f),
      _ => Result::Err("expected Float"),
    }
  }
}

impl TryFrom<&LxVal> for bool {
  type Error = &'static str;
  fn try_from(v: &LxVal) -> Result<Self, Self::Error> {
    match v {
      LxVal::Bool(b) => Ok(*b),
      _ => Result::Err("expected Bool"),
    }
  }
}

impl fmt::Display for LxVal {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LxVal::Int(n) => write!(f, "{n}"),
      LxVal::Float(v) => write!(f, "{v}"),
      LxVal::Bool(b) => write!(f, "{b}"),
      LxVal::Str(s) => write!(f, "{s}"),
      LxVal::Unit => write!(f, "()"),
      LxVal::List(items) => write!(f, "[{}]", items.iter().format(" ")),
      LxVal::Tuple(items) => write!(f, "({})", items.iter().format(" ")),
      LxVal::Record(fields) => {
        let inner = fields.iter().format_with("  ", |(k, v), g| g(&format_args!("{k}: {v}")));
        write!(f, "{{{inner}}}")
      },
      LxVal::Map(entries) => {
        let inner = entries.iter().format_with("  ", |(k, v), g| g(&format_args!("{}: {v}", k.0)));
        write!(f, "Map{{{inner}}}")
      },
      LxVal::Func(_) => write!(f, "<func>"),
      LxVal::BuiltinFunc(b) => write!(f, "<builtin {}/{}>", b.name, b.arity),
      LxVal::Ok(v) => write!(f, "Ok {v}"),
      LxVal::Err(v) => write!(f, "Err {v}"),
      LxVal::Some(v) => write!(f, "Some {v}"),
      LxVal::None => write!(f, "None"),
      LxVal::Tagged { tag, values } if values.is_empty() => write!(f, "{}", resolve(*tag)),
      LxVal::Tagged { tag, values } => write!(f, "{} {}", resolve(*tag), values.iter().format(" ")),
      LxVal::TaggedCtor { tag, .. } => write!(f, "<ctor {}>", resolve(*tag)),
      LxVal::Range { start, end, inclusive: true } => write!(f, "{start}..={end}"),
      LxVal::Range { start, end, inclusive: false } => write!(f, "{start}..{end}"),
      LxVal::TraitUnion { name, .. } => write!(f, "<Trait {}>", resolve(*name)),
      LxVal::Trait(t) => write!(f, "<Trait {}>", resolve(t.name)),
      LxVal::Class(c) if c.traits.iter().any(|t| resolve(*t) == "Agent") => {
        write!(f, "<Agent {}>", resolve(c.name))
      },
      LxVal::Class(c) => write!(f, "<Class {}>", resolve(c.name)),
      LxVal::Object(o) => write!(f, "<{}#{}>", resolve(o.class_name), o.id),
      LxVal::Store { id } => write!(f, "<Store#{id}>"),
    }
  }
}

impl fmt::Debug for BuiltinFunc {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<builtin {}/{}>", self.name, self.arity)
  }
}
