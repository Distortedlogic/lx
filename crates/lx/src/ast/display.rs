use crate::sym::resolve;
use std::fmt;

use itertools::Itertools;

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
      Pattern::Tuple(pats) => write!(f, "({})", pats.iter().map(|p| &p.node).format(", ")),
      Pattern::List { elems, rest } => {
        write!(f, "[{}", elems.iter().map(|p| &p.node).format(" "))?;
        if let Some(r) = rest {
          if !elems.is_empty() {
            write!(f, " ")?;
          }
          write!(f, "..{r}")?;
        }
        write!(f, "]")
      },
      Pattern::Record { fields, rest } => {
        write!(
          f,
          "{{{}",
          fields.iter().format_with(" ", |fp, g| {
            if let Some(sub) = &fp.pattern { g(&format_args!("{}: {}", fp.name, sub.node)) } else { g(&format_args!("{}", fp.name)) }
          })
        )?;
        if let Some(r) = rest {
          if !fields.is_empty() {
            write!(f, " ")?;
          }
          write!(f, "..{r}")?;
        }
        write!(f, "}}")
      },
      Pattern::Constructor { name, args } if args.is_empty() => write!(f, "{name}"),
      Pattern::Constructor { name, args } => write!(f, "{name} {}", args.iter().map(|a| &a.node).format(" ")),
    }
  }
}
