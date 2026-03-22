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

macro_rules! try_from_lxval {
  ($($target:ty, $variant:ident, $bind:ident => $extract:expr, $msg:expr);+ $(;)?) => {
    $(
      impl TryFrom<&LxVal> for $target {
        type Error = &'static str;
        fn try_from(v: &LxVal) -> Result<Self, Self::Error> {
          match v {
            LxVal::$variant($bind) => Ok($extract),
            _ => Result::Err($msg),
          }
        }
      }
    )+
  };
}

try_from_lxval! {
  BigInt, Int, n => n.clone(), "expected Int";
  f64, Float, f => *f, "expected Float";
  bool, Bool, b => *b, "expected Bool";
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
      LxVal::Func(_) | LxVal::MultiFunc(_) => write!(f, "<func>"),
      LxVal::BuiltinFunc(b) => write!(f, "<builtin {}/{}>", b.name, b.arity),
      LxVal::Ok(v) => write!(f, "Ok {v}"),
      LxVal::Err(v) => write!(f, "Err {v}"),
      LxVal::Some(v) => write!(f, "Some {v}"),
      LxVal::None => write!(f, "None"),
      LxVal::Tagged { tag, values } if values.is_empty() => write!(f, "{tag}"),
      LxVal::Tagged { tag, values } => write!(f, "{} {}", tag, values.iter().format(" ")),
      LxVal::TaggedCtor { tag, .. } => write!(f, "<ctor {tag}>"),
      LxVal::Range { start, end, inclusive: true } => write!(f, "{start}..={end}"),
      LxVal::Range { start, end, inclusive: false } => write!(f, "{start}..{end}"),
      LxVal::TraitUnion { name, .. } => write!(f, "<Trait {name}>"),
      LxVal::Trait(t) => write!(f, "<Trait {}>", t.name),
      LxVal::Class(c) if c.traits.iter().any(|t| t == "Agent") => {
        write!(f, "<Agent {}>", c.name)
      },
      LxVal::Class(c) => write!(f, "<Class {}>", c.name),
      LxVal::Object(o) => write!(f, "<{}#{}>", o.class_name, o.id),
      LxVal::Store { id } => write!(f, "<Store#{id}>"),
      LxVal::Stream { id } => write!(f, "<Stream#{id}>"),
    }
  }
}

impl fmt::Debug for BuiltinFunc {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<builtin {}/{}>", self.name, self.arity)
  }
}
