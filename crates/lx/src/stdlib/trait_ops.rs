use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::{LxVal, TraitMethodDef};
use miette::SourceSpan;

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("methods".into(), mk("trait.methods", 1, bi_methods));
  m.insert("match".into(), mk("trait.match", 2, bi_match));
  m
}

fn method_to_record(m: &TraitMethodDef) -> LxVal {
  let mut rec = IndexMap::new();
  rec.insert("name".into(), LxVal::str(&m.name));
  let input_fields: Vec<LxVal> = m
    .input
    .iter()
    .map(|f| {
      let mut fr = IndexMap::new();
      fr.insert("name".into(), LxVal::str(&f.name));
      fr.insert("type".into(), LxVal::str(&f.type_name));
      LxVal::record(fr)
    })
    .collect();
  rec.insert("input".into(), LxVal::list(input_fields));
  rec.insert("output".into(), LxVal::str(&m.output));
  LxVal::record(rec)
}

fn bi_methods(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let LxVal::Trait { methods, .. } = &args[0] else {
    return Err(LxError::type_err(format!("trait.methods: expected Trait, got {} `{}`", args[0].type_name(), args[0].short_display()), span));
  };
  let records: Vec<LxVal> = methods.iter().map(method_to_record).collect();
  Ok(LxVal::list(records))
}

fn bi_match(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let LxVal::Trait { methods, .. } = &args[0] else {
    return Err(LxError::type_err(format!("trait.match: expected Trait, got {} `{}`", args[0].type_name(), args[0].short_display()), span));
  };
  let query = args[1]
    .as_str()
    .ok_or_else(|| LxError::type_err(format!("trait.match: expected Str query, got {} `{}`", args[1].type_name(), args[1].short_display()), span))?;
  let query_lower = query.to_lowercase();
  let words: Vec<&str> = query_lower.split_whitespace().collect();
  let mut best_name = String::new();
  let mut best_score = 0.0_f64;
  for m in methods.iter() {
    let name_lower = m.name.to_lowercase();
    let mut hits = 0;
    for w in &words {
      if name_lower.contains(w) {
        hits += 1;
      }
    }
    let score = if words.is_empty() { 0.0 } else { hits as f64 / words.len() as f64 };
    if score > best_score {
      best_score = score;
      best_name = m.name.clone();
    }
  }
  if best_score > 0.0 {
    let mut rec = IndexMap::new();
    rec.insert("method".into(), LxVal::str(best_name));
    rec.insert("score".into(), LxVal::Float(best_score));
    Ok(LxVal::record(rec))
  } else {
    Ok(LxVal::None)
  }
}
