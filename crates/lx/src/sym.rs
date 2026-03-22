use std::borrow::Borrow;
use std::fmt;
use std::sync::OnceLock;

use lasso::{Spur, ThreadedRodeo};

static INTERNER: OnceLock<ThreadedRodeo> = OnceLock::new();

fn interner() -> &'static ThreadedRodeo {
  INTERNER.get_or_init(ThreadedRodeo::default)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sym(Spur);

impl Sym {
  pub fn as_str(self) -> &'static str {
    interner().resolve(&self.0)
  }
}

impl fmt::Debug for Sym {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl fmt::Display for Sym {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl AsRef<str> for Sym {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl Borrow<str> for Sym {
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl PartialEq<str> for Sym {
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl PartialEq<&str> for Sym {
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
  }
}

impl From<&str> for Sym {
  fn from(s: &str) -> Self {
    intern(s)
  }
}

impl From<String> for Sym {
  fn from(s: String) -> Self {
    intern(&s)
  }
}

impl From<&String> for Sym {
  fn from(s: &String) -> Self {
    intern(s)
  }
}

pub fn intern(s: &str) -> Sym {
  Sym(interner().get_or_intern(s))
}

pub fn resolve(sym: Sym) -> &'static str {
  sym.as_str()
}
