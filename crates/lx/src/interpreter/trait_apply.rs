use crate::sym::Sym;
use std::sync::Arc;

use crate::error::{EvalResult, LxError};
use crate::value::{FieldDef, LxVal};
use miette::SourceSpan;

use super::Interpreter;

impl Interpreter {
  pub(super) async fn apply_trait_fields(&mut self, name: &str, fields: &Arc<Vec<FieldDef>>, arg: &LxVal, _span: SourceSpan) -> EvalResult<LxVal> {
    let LxVal::Record(rec) = arg else {
      return Ok(LxVal::err_str(format!("Trait {name}: expected Record, got {}", arg.type_name())));
    };
    let mut result = rec.as_ref().clone();
    for field in fields.iter() {
      match rec.get(&field.name) {
        Some(val) => {
          if field.type_name != "Any" && val.type_name() != field.type_name.as_str() {
            return Ok(LxVal::err_str(format!("Trait {name}: field '{}' expected {}, got {}", field.name, field.type_name, val.type_name())));
          }
        },
        None => {
          if let Some(ref default) = field.default {
            result.insert(field.name, default.clone());
          } else {
            return Ok(LxVal::err_str(format!("Trait {name}: missing required field '{}'", field.name)));
          }
        },
      }
    }
    for field in fields.iter() {
      if let Some(ref constraint) = field.constraint {
        let val = result.get(&field.name).cloned().unwrap_or(LxVal::Unit);
        let saved = Arc::clone(&self.env);
        let saved_arena = Arc::clone(&self.arena);
        let scope = self.env.child();
        scope.bind(field.name, val);
        self.env = Arc::new(scope);
        self.arena = Arc::clone(&constraint.arena);
        let ok = self.eval(constraint.expr_id).await?;
        self.env = saved;
        self.arena = saved_arena;
        match ok.as_bool() {
          Some(true) => {},
          _ => {
            return Ok(LxVal::err_str(format!("Trait {name}: field '{}' constraint violated", field.name)));
          },
        }
      }
    }
    Ok(LxVal::record(result))
  }

  pub(super) async fn apply_trait_union(&mut self, name: &str, variants: &Arc<Vec<Sym>>, arg: &LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
    let LxVal::Record(rec) = arg else {
      return Ok(LxVal::err_str(format!("Trait union {name}: expected Record, got {}", arg.type_name())));
    };
    for variant_name in variants.iter() {
      let Some(proto) = self.env.get(*variant_name) else {
        continue;
      };
      let LxVal::Trait(ref proto_trait) = proto else {
        continue;
      };
      if self.try_match_variant(&proto_trait.fields, rec, span).is_ok() {
        let mut result = rec.as_ref().clone();
        result.insert(crate::sym::intern("_variant"), LxVal::str(variant_name.as_str()));
        for field in proto_trait.fields.iter() {
          if !rec.contains_key(&field.name)
            && let Some(ref default) = field.default
          {
            result.insert(field.name, default.clone());
          }
        }
        return Ok(LxVal::record(result));
      }
    }
    Ok(LxVal::err_str(format!("Trait union {name}: no variant matched. Tried: {}", variants.iter().map(|v| v.as_str()).collect::<Vec<_>>().join(", "))))
  }

  fn try_match_variant(&mut self, fields: &Arc<Vec<FieldDef>>, rec: &Arc<indexmap::IndexMap<Sym, LxVal>>, span: SourceSpan) -> Result<(), LxError> {
    for field in fields.iter() {
      match rec.get(&field.name) {
        Some(val) => {
          if field.type_name != "Any" && val.type_name() != field.type_name.as_str() {
            return Err(LxError::runtime(
              format!("field '{}': expected {}, got {} `{}`", field.name, field.type_name, val.type_name(), val.short_display()),
              span,
            ));
          }
        },
        None => {
          if field.default.is_none() {
            return Err(LxError::runtime(format!("missing required field '{}'", field.name), span));
          }
        },
      }
    }
    Ok(())
  }
}
