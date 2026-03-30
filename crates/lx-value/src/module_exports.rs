use crate::LxVal;
use indexmap::IndexMap;
use lx_span::sym::Sym;

#[derive(Debug, Clone)]
pub struct ModuleExports {
  pub bindings: IndexMap<Sym, LxVal>,
  pub variant_ctors: Vec<Sym>,
}
