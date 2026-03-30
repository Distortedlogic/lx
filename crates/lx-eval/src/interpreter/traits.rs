use std::sync::Arc;

use indexmap::IndexMap;

use lx_value::LxError;
use lx_value::LxVal;
use miette::SourceSpan;

use super::Interpreter;

impl Interpreter {
  pub(super) fn inject_traits(
    methods: &mut IndexMap<lx_span::sym::Sym, LxVal>,
    traits: &[lx_span::sym::Sym],
    env: &Arc<lx_value::Env>,
    kind: &str,
    name: &str,
    span: SourceSpan,
  ) -> Result<(), LxError> {
    for tn in traits {
      let Some(LxVal::Trait(t)) = env.get(*tn) else {
        return Err(LxError::runtime(format!("{kind} '{name}' declares Trait '{tn}' but it is not defined — add `use` to import it"), span));
      };
      for (k, v) in t.defaults.iter() {
        if !methods.contains_key(k) {
          methods.insert(*k, v.clone());
        }
      }
      for r in t.methods.iter() {
        if !methods.contains_key(&r.name) {
          return Err(LxError::runtime(format!("{kind} {name} missing method '{}' required by Trait {tn}", r.name), span));
        }
      }
    }
    Ok(())
  }
}
