use std::fmt;

use num_bigint::BigInt;

use lx_span::sym::Sym;

#[derive(Debug, Clone, PartialEq)]
pub enum Pat {
  Wildcard,
  Constructor { name: Sym, arity: usize, args: Vec<Pat> },
  Literal(LitPat),
  Tuple(Vec<Pat>),
}

#[derive(Debug, Clone)]
pub enum LitPat {
  Int(BigInt),
  Float(f64),
  Str(String),
  Bool(bool),
  Unit,
}

impl PartialEq for LitPat {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (LitPat::Int(a), LitPat::Int(b)) => a == b,
      (LitPat::Float(a), LitPat::Float(b)) => a.to_bits() == b.to_bits(),
      (LitPat::Str(a), LitPat::Str(b)) => a == b,
      (LitPat::Bool(a), LitPat::Bool(b)) => a == b,
      (LitPat::Unit, LitPat::Unit) => true,
      _ => false,
    }
  }
}

impl fmt::Display for Pat {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Pat::Wildcard => write!(f, "_"),
      Pat::Constructor { name, args, .. } => {
        if args.is_empty() {
          write!(f, "{name}")
        } else {
          write!(f, "{name}(")?;
          for (i, arg) in args.iter().enumerate() {
            if i > 0 {
              write!(f, ", ")?;
            }
            write!(f, "{arg}")?;
          }
          write!(f, ")")
        }
      },
      Pat::Literal(lit) => write!(f, "{lit}"),
      Pat::Tuple(elems) => {
        write!(f, "(")?;
        for (i, e) in elems.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{e}")?;
        }
        write!(f, ")")
      },
    }
  }
}

impl fmt::Display for LitPat {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LitPat::Int(n) => write!(f, "{n}"),
      LitPat::Float(v) => write!(f, "{v}"),
      LitPat::Str(s) => write!(f, "\"{s}\""),
      LitPat::Bool(b) => write!(f, "{b}"),
      LitPat::Unit => write!(f, "()"),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CtorId {
  Named(Sym),
  Literal(LitPat),
  Tuple(usize),
}

impl Pat {
  fn extract_head(&self, n: usize) -> Vec<Pat> {
    match self {
      Pat::Wildcard => vec![Pat::Wildcard; n],
      Pat::Tuple(elems) if elems.len() >= n => elems[..n].to_vec(),
      Pat::Constructor { args, .. } if args.len() >= n => args[..n].to_vec(),
      _ => vec![Pat::Wildcard; n],
    }
  }
}

impl CtorId {
  pub fn reconstruct(&self, arity: usize, witness: &Pat) -> Pat {
    match self {
      CtorId::Named(name) => {
        if arity == 0 {
          Pat::Constructor { name: *name, arity: 0, args: Vec::new() }
        } else {
          let args = witness.extract_head(arity);
          Pat::Constructor { name: *name, arity, args }
        }
      },
      CtorId::Literal(lit) => Pat::Literal(lit.clone()),
      CtorId::Tuple(n) => {
        let elems = witness.extract_head(*n);
        Pat::Tuple(elems)
      },
    }
  }
}
