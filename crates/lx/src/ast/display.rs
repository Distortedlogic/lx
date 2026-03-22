use std::fmt;

use super::{Literal, Pattern};

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
      Pattern::Tuple(pats) => {
        write!(f, "(")?;
        for (i, p) in pats.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{}", p.node)?;
        }
        write!(f, ")")
      },
      Pattern::List { elems, rest } => {
        write!(f, "[")?;
        for (i, p) in elems.iter().enumerate() {
          if i > 0 {
            write!(f, " ")?;
          }
          write!(f, "{}", p.node)?;
        }
        if let Some(r) = rest {
          if !elems.is_empty() {
            write!(f, " ")?;
          }
          write!(f, "..{r}")?;
        }
        write!(f, "]")
      },
      Pattern::Record { fields, rest } => {
        write!(f, "{{")?;
        for (i, fp) in fields.iter().enumerate() {
          if i > 0 {
            write!(f, " ")?;
          }
          write!(f, "{}", fp.name)?;
          if let Some(sub) = &fp.pattern {
            write!(f, ": {}", sub.node)?;
          }
        }
        if let Some(r) = rest {
          if !fields.is_empty() {
            write!(f, " ")?;
          }
          write!(f, "..{r}")?;
        }
        write!(f, "}}")
      },
      Pattern::Constructor { name, args } => {
        write!(f, "{name}")?;
        for a in args {
          write!(f, " {}", a.node)?;
        }
        Ok(())
      },
    }
  }
}
