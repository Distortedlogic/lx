use std::sync::Arc;

use indexmap::IndexMap;

use crate::BuiltinCtx;
use crate::error::LxError;
use crate::record;
use crate::std_module;
use crate::value::{LxVal, TraitMethodDef};
use miette::SourceSpan;

pub fn build() -> IndexMap<crate::sym::Sym, LxVal> {
  std_module! {
    "methods" => "trait.methods", 1, bi_methods;
    "match"   => "trait.match",   2, bi_match
  }
}

fn method_to_record(m: &TraitMethodDef) -> LxVal {
  let input_fields: Vec<LxVal> = m
    .input
    .iter()
    .map(|f| {
      record! { "name" => LxVal::str(f.name), "type" => LxVal::str(f.type_name) }
    })
    .collect();
  record! {
      "name" => LxVal::str(m.name),
      "input" => LxVal::list(input_fields),
      "output" => LxVal::str(m.output),
  }
}

fn bi_methods(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let LxVal::Trait(t) = &args[0] else {
    return Err(LxError::type_err(format!("trait.methods: expected Trait, got {} `{}`", args[0].type_name(), args[0].short_display()), span, None));
  };
  let records: Vec<LxVal> = t.methods.iter().map(method_to_record).collect();
  Ok(LxVal::list(records))
}

fn bi_match(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let LxVal::Trait(t) = &args[0] else {
    return Err(LxError::type_err(format!("trait.match: expected Trait, got {} `{}`", args[0].type_name(), args[0].short_display()), span, None));
  };
  let query = args[1]
    .as_str()
    .ok_or_else(|| LxError::type_err(format!("trait.match: expected Str query, got {} `{}`", args[1].type_name(), args[1].short_display()), span, None))?;
  let query_lower = query.to_lowercase();
  let words: Vec<&str> = query_lower.split_whitespace().collect();
  let mut best_name = "";
  let mut best_score = 0.0_f64;
  for m in t.methods.iter() {
    let name_lower = m.name.as_str().to_lowercase();
    let mut hits = 0;
    for w in &words {
      if name_lower.contains(w) {
        hits += 1;
      }
    }
    let score = if words.is_empty() { 0.0 } else { hits as f64 / words.len() as f64 };
    if score > best_score {
      best_score = score;
      best_name = m.name.as_str();
    }
  }
  if best_score > 0.0 { Ok(record! { "method" => LxVal::str(best_name), "score" => LxVal::Float(best_score) }) } else { Ok(LxVal::None) }
}
