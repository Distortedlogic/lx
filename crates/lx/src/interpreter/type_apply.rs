use std::sync::Arc;

use super::Interpreter;
use crate::ast::{FieldDecl, TraitEntry};
use crate::error::{EvalResult, EvalSignal, LxError};
use crate::sym::Sym;
use crate::value::{ConstraintExpr, FieldDef, LxVal};
use miette::{SourceOffset, SourceSpan};

impl Interpreter {
  pub async fn call(&mut self, func: LxVal, arg: LxVal) -> Result<LxVal, LxError> {
    let span = SourceSpan::new(SourceOffset::from(0), 0);
    self.apply_func(func, arg, span).await.map_err(|e| match e {
      EvalSignal::Error(e) => e,
      EvalSignal::Break(_) => LxError::runtime("break outside loop", span),
      EvalSignal::AgentStop => LxError::runtime("agent stopped", span),
    })
  }

  pub(super) async fn eval_trait_fields(&mut self, name: &str, entries: &[TraitEntry], span: SourceSpan) -> EvalResult<Vec<FieldDef>> {
    let mut fields = Vec::new();
    for entry in entries {
      match entry {
        TraitEntry::Spread(base_name) => {
          let base = self.env.get(*base_name).ok_or_else(|| LxError::runtime(format!("Trait {name}: spread base '{base_name}' not found"), span))?;
          let LxVal::Trait(ref base_trait) = base else {
            return Err(LxError::runtime(format!("Trait {name}: '{base_name}' is not a Trait, got {}", base.type_name()), span).into());
          };
          for f in base_trait.fields.iter() {
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

  async fn eval_field_decl(&mut self, f: &FieldDecl) -> EvalResult<FieldDef> {
    let default = match f.default {
      Some(eid) => Some(self.eval(eid).await?),
      None => None,
    };
    let constraint = f.constraint.map(|eid| ConstraintExpr { expr_id: eid, arena: Arc::clone(&self.arena) });
    Ok(FieldDef { name: f.name, type_name: f.type_name, default, constraint })
  }

  pub(super) fn eval_trait_union(&mut self, name: Sym, variants: &[Sym], span: SourceSpan) -> Result<LxVal, LxError> {
    for v in variants {
      let val = self.env.get(*v).ok_or_else(|| LxError::runtime(format!("Trait union {name}: variant '{v}' not found"), span))?;
      if !matches!(val, LxVal::Trait(_)) {
        return Err(LxError::runtime(format!("Trait union {name}: variant '{v}' is not a Trait, got {}", val.type_name()), span));
      }
    }
    let val = LxVal::TraitUnion { name, variants: Arc::new(variants.to_vec()) };
    let env = self.env.child();
    env.bind(name, val);
    self.env = Arc::new(env);
    Ok(LxVal::Unit)
  }

  pub(super) fn update_record_field(val: &LxVal, fields: &[Sym], new_val: LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
    match (val, fields) {
      (LxVal::Record(rec), [field]) => {
        let mut new_rec = rec.as_ref().clone();
        new_rec.insert(*field, new_val);
        Ok(LxVal::record(new_rec))
      },
      (LxVal::Record(rec), [field, rest @ ..]) => {
        let inner = rec.get(field).ok_or_else(|| LxError::runtime(format!("field '{field}' not found"), span))?;
        let updated = Self::update_record_field(inner, rest, new_val, span)?;
        let mut new_rec = rec.as_ref().clone();
        new_rec.insert(*field, updated);
        Ok(LxVal::record(new_rec))
      },
      (other, _) => Err(LxError::type_err(format!("field update requires Record, got {}", other.type_name()), span, None)),
    }
  }
}
