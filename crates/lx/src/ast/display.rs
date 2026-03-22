use std::fmt;

use super::{Literal, Pattern, PatternConstructor};

impl fmt::Display for Pattern {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Pattern::Bind(name) => write!(f, "{name}"),
      Pattern::Wildcard => write!(f, "_"),
      Pattern::Literal(lit) => match lit {
        Literal::Int(n) => write!(f, "{n}"),
        Literal::Float(v) => write!(f, "{v}"),
        Literal::Bool(b) => write!(f, "{b}"),
        Literal::Unit => write!(f, "()"),
        Literal::RawStr(s) => write!(f, "\"{s}\""),
        Literal::Str(_) => write!(f, "\"...\""),
      },
      Pattern::Tuple(_) => write!(f, "(...)"),
      Pattern::List(_) => write!(f, "[...]"),
      Pattern::Record(_) => write!(f, "{{...}}"),
      Pattern::Constructor(PatternConstructor { name, args }) if args.is_empty() => {
        write!(f, "{name}")
      },
      Pattern::Constructor(PatternConstructor { name, .. }) => write!(f, "{name} ..."),
    }
  }
}
