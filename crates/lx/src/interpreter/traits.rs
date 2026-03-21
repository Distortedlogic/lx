use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::Interpreter;

impl Interpreter {
  pub(super) fn inject_traits(
    methods: &mut IndexMap<String, LxVal>,
    traits: &[String],
    env: &Arc<crate::env::Env>,
    kind: &str,
    name: &str,
    span: Span,
  ) -> Result<(), LxError> {
    for tn in traits {
      let Some(LxVal::Trait { methods: req, defaults, .. }) = env.get(tn) else {
        continue;
      };
      for (k, v) in defaults.iter() {
        if !methods.contains_key(k) {
          methods.insert(k.clone(), v.clone());
        }
      }
      for r in req.iter() {
        if !methods.contains_key(&r.name) {
          return Err(LxError::runtime(format!("{kind} {name} missing method '{}' required by Trait {tn}", r.name), span));
        }
      }
    }
    Ok(())
  }
}
