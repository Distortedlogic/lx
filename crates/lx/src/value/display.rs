use std::fmt;
use std::sync::Arc;

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
      LxVal::List(items) => {
        write!(f, "[")?;
        for (i, item) in items.iter().enumerate() {
          if i > 0 {
            write!(f, " ")?;
          }
          write!(f, "{item}")?;
        }
        write!(f, "]")
      },
      LxVal::Tuple(items) => {
        write!(f, "(")?;
        for (i, item) in items.iter().enumerate() {
          if i > 0 {
            write!(f, " ")?;
          }
          write!(f, "{item}")?;
        }
        write!(f, ")")
      },
      LxVal::Record(fields) => {
        write!(f, "{{")?;
        for (i, (k, v)) in fields.iter().enumerate() {
          if i > 0 {
            write!(f, "  ")?;
          }
          write!(f, "{k}: {v}")?;
        }
        write!(f, "}}")
      },
      LxVal::Map(entries) => {
        write!(f, "Map{{")?;
        for (i, (k, v)) in entries.iter().enumerate() {
          if i > 0 {
            write!(f, "  ")?;
          }
          write!(f, "{}: {v}", k.0)?;
        }
        write!(f, "}}")
      },
      LxVal::Func(_) => write!(f, "<func>"),
      LxVal::BuiltinFunc(b) => write!(f, "<builtin {}/{}>", b.name, b.arity),
      LxVal::Ok(v) => write!(f, "Ok {v}"),
      LxVal::Err(v) => write!(f, "Err {v}"),
      LxVal::Some(v) => write!(f, "Some {v}"),
      LxVal::None => write!(f, "None"),
      LxVal::Tagged { tag, values } => {
        write!(f, "{tag}")?;
        for v in values.iter() {
          write!(f, " {v}")?;
        }
        Ok(())
      },
      LxVal::TaggedCtor { tag, .. } => write!(f, "<ctor {tag}>"),
      LxVal::Range { start, end, inclusive } => {
        if *inclusive {
          write!(f, "{start}..={end}")
        } else {
          write!(f, "{start}..{end}")
        }
      },
      LxVal::TraitUnion { name, .. } => write!(f, "<Trait {name}>"),
      LxVal::Trait { name, .. } => write!(f, "<Trait {name}>"),
      LxVal::Class { name, traits, .. } => {
        if traits.iter().any(|t| t.as_ref() == "Agent") {
          write!(f, "<Agent {name}>")
        } else {
          write!(f, "<Class {name}>")
        }
      },
      LxVal::Object { class_name, id, .. } => write!(f, "<{class_name}#{id}>"),
      LxVal::Store { id } => write!(f, "<Store#{id}>"),
    }
  }
}

impl fmt::Debug for BuiltinFunc {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<builtin {}/{}>", self.name, self.arity)
  }
}
