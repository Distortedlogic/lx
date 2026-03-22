use std::sync::OnceLock;

use lasso::{Spur, ThreadedRodeo};

pub type Sym = Spur;

static INTERNER: OnceLock<ThreadedRodeo> = OnceLock::new();

fn interner() -> &'static ThreadedRodeo {
  INTERNER.get_or_init(ThreadedRodeo::default)
}

pub fn intern(s: &str) -> Sym {
  interner().get_or_intern(s)
}

pub fn resolve(sym: Sym) -> &'static str {
  interner().resolve(&sym)
}
