use std::sync::Arc;

use super::Interpreter;
use crate::ast::{FieldDecl, TraitEntry};
use crate::error::LxError;
use crate::value::{FieldDef, LxVal};
use miette::{SourceOffset, SourceSpan};

impl Interpreter {
  pub async fn call(&mut self, func: LxVal, arg: LxVal) -> Result<LxVal, LxError> {
    self.apply_func(func, arg, SourceSpan::new(SourceOffset::from(0), 0)).await
  }

  pub(super) async fn eval_trait_fields(&mut self, name: &str, entries: &[TraitEntry], span: SourceSpan) -> Result<Vec<FieldDef>, LxError> {
    let mut fields = Vec::new();
    for entry in entries {
      match entry {
        TraitEntry::Spread(base_name) => {
          let base = self.env.get(base_name).ok_or_else(|| LxError::runtime(format!("Trait {name}: spread base '{base_name}' not found"), span))?;
          let LxVal::Trait { fields: base_fields, .. } = &base else {
            return Err(LxError::runtime(format!("Trait {name}: '{base_name}' is not a Trait, got {}", base.type_name()), span));
          };
          for f in base_fields.iter() {
            if let Some(pos) = fields.iter().position(|pf: &FieldDef| pf.name == f.name) {
              fields[pos] = f.clone();
            } else {
              fields.push(f.clone());
            }
          }
        },
        TraitEntry::Field(f) => {
          let def = self.eval_field_decl(f).await?;
          if let Some(pos) = fields.iter().position(|pf: &FieldDef| pf.name == def.name) {
            fields[pos] = def;
          } else {
            fields.push(def);
          }
        },
      }
    }
    Ok(fields)
  }

  async fn eval_field_decl(&mut self, f: &FieldDecl) -> Result<FieldDef, LxError> {
    let default = match &f.default {
      Some(e) => Some(self.eval(e).await?),
      None => None,
    };
    let constraint = f.constraint.as_ref().map(|e| Arc::new(e.clone()));
    Ok(FieldDef { name: f.name.clone(), type_name: f.type_name.clone(), default, constraint })
  }

  pub(super) fn eval_trait_union(&mut self, name: &str, variants: &[String], span: SourceSpan) -> Result<LxVal, LxError> {
    for v in variants {
      let val = self.env.get(v).ok_or_else(|| LxError::runtime(format!("Trait union {name}: variant '{v}' not found"), span))?;
      if !matches!(val, LxVal::Trait { .. }) {
        return Err(LxError::runtime(format!("Trait union {name}: variant '{v}' is not a Trait, got {}", val.type_name()), span));
      }
    }
    let variant_arcs: Vec<Arc<str>> = variants.iter().map(|v| Arc::from(v.as_str())).collect();
    let val = LxVal::TraitUnion { name: Arc::from(name), variants: Arc::new(variant_arcs) };
    let mut env = self.env.child();
    env.bind(name.to_string(), val);
    self.env = env.into_arc();
    Ok(LxVal::Unit)
  }

  pub(super) fn update_record_field(val: &LxVal, fields: &[String], new_val: LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
    match (val, fields) {
      (LxVal::Record(rec), [field]) => {
        let mut new_rec = rec.as_ref().clone();
        new_rec.insert(field.clone(), new_val);
        Ok(LxVal::record(new_rec))
      },
      (LxVal::Record(rec), [field, rest @ ..]) => {
        let inner = rec.get(field).ok_or_else(|| LxError::runtime(format!("field '{field}' not found"), span))?;
        let updated = Self::update_record_field(inner, rest, new_val, span)?;
        let mut new_rec = rec.as_ref().clone();
        new_rec.insert(field.clone(), updated);
        Ok(LxVal::record(new_rec))
      },
      (other, _) => Err(LxError::type_err(format!("field update requires Record, got {}", other.type_name()), span)),
    }
  }
}
