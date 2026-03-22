use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

use super::Interpreter;

impl Interpreter {
  pub(super) fn inject_traits(
    methods: &mut IndexMap<String, LxVal>,
    traits: &[String],
    env: &Arc<crate::env::Env>,
    kind: &str,
    name: &str,
    span: SourceSpan,
  ) -> Result<(), LxError> {
    for tn in traits {
      let Some(LxVal::Trait(t)) = env.get_str(tn) else {
        continue;
      };
      for (k, v) in t.defaults.iter() {
        if !methods.contains_key(k) {
          methods.insert(k.clone(), v.clone());
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
