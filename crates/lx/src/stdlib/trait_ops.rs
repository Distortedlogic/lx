use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{McpOutputDef, TraitMethodDef, Value};

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("methods".into(), mk("trait.methods", 1, bi_methods));
    m.insert("match".into(), mk("trait.match", 2, bi_match));
    m
}

fn method_to_record(m: &TraitMethodDef) -> Value {
    let mut rec = IndexMap::new();
    rec.insert("name".into(), Value::Str(Arc::from(m.name.as_str())));
    let input_fields: Vec<Value> = m
        .input
        .iter()
        .map(|f| {
            let mut fr = IndexMap::new();
            fr.insert("name".into(), Value::Str(Arc::from(f.name.as_str())));
            fr.insert("type".into(), Value::Str(Arc::from(f.type_name.as_str())));
            Value::Record(Arc::new(fr))
        })
        .collect();
    rec.insert("input".into(), Value::List(Arc::new(input_fields)));
    let output_str = output_type_str(&m.output);
    rec.insert("output".into(), Value::Str(Arc::from(output_str.as_str())));
    Value::Record(Arc::new(rec))
}

fn output_type_str(out: &McpOutputDef) -> String {
    match out {
        McpOutputDef::Simple(s) => s.clone(),
        McpOutputDef::Record(fields) => {
            let parts: Vec<String> = fields
                .iter()
                .map(|f| format!("{}: {}", f.name, f.type_name))
                .collect();
            format!("{{{}}}", parts.join("  "))
        }
        McpOutputDef::List(inner) => format!("[{}]", output_type_str(inner)),
    }
}

fn bi_methods(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Trait { methods, .. } = &args[0] else {
        return Err(LxError::type_err(
            format!(
                "trait.methods: expected Trait, got {} `{}`",
                args[0].type_name(),
                args[0].short_display()
            ),
            span,
        ));
    };
    let records: Vec<Value> = methods.iter().map(method_to_record).collect();
    Ok(Value::List(Arc::new(records)))
}

fn bi_match(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Trait { methods, .. } = &args[0] else {
        return Err(LxError::type_err(
            format!(
                "trait.match: expected Trait, got {} `{}`",
                args[0].type_name(),
                args[0].short_display()
            ),
            span,
        ));
    };
    let query = args[1].as_str().ok_or_else(|| {
        LxError::type_err(
            format!(
                "trait.match: expected Str query, got {} `{}`",
                args[1].type_name(),
                args[1].short_display()
            ),
            span,
        )
    })?;
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
        let score = if words.is_empty() {
            0.0
        } else {
            hits as f64 / words.len() as f64
        };
        if score > best_score {
            best_score = score;
            best_name = m.name.clone();
        }
    }
    if best_score > 0.0 {
        let mut rec = IndexMap::new();
        rec.insert("method".into(), Value::Str(Arc::from(best_name.as_str())));
        rec.insert("score".into(), Value::Float(best_score));
        Ok(Value::Record(Arc::new(rec)))
    } else {
        Ok(Value::None)
    }
}
