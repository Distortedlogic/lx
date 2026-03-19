use std::sync::Arc;

use indexmap::IndexMap;
use num_traits::ToPrimitive;

use crate::backends::{AiOpts, RuntimeCtx};
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::{ProtoFieldDef, Value};

pub fn mk_prompt_structured() -> Value {
    mk("ai.prompt_structured", 2, bi_prompt_structured)
}

pub fn mk_prompt_structured_with() -> Value {
    mk("ai.prompt_structured_with", 2, bi_prompt_structured_with)
}

pub fn mk_prompt_json() -> Value {
    mk("ai.prompt_json", 2, bi_prompt_json)
}

fn bi_prompt_json(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let prompt = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("ai.prompt_json: first arg must be prompt Str", span))?;
    let Value::Record(shape) = &args[1] else {
        return Err(LxError::type_err(
            "ai.prompt_json: second arg must be shape Record",
            span,
        ));
    };
    let fields = record_to_fields(shape);
    let augmented = augment_prompt(prompt, "json", &fields);
    run_structured(ctx, &augmented, &AiOpts::default(), &fields, 2, span)
}

fn record_to_fields(rec: &IndexMap<String, Value>) -> Vec<ProtoFieldDef> {
    rec.iter()
        .map(|(name, val)| {
            let type_name = match val {
                Value::List(_) => "List".to_string(),
                _ => val.type_name().to_string(),
            };
            ProtoFieldDef {
                name: name.clone(),
                type_name,
                default: None,
                constraint: None,
            }
        })
        .collect()
}

fn bi_prompt_structured(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let (proto_name, fields) = extract_protocol(&args[0], span)?;
    let prompt = args[1].as_str().ok_or_else(|| {
        LxError::type_err("ai.prompt_structured: second arg must be prompt Str", span)
    })?;
    let augmented = augment_prompt(prompt, &proto_name, &fields);
    run_structured(ctx, &augmented, &AiOpts::default(), &fields, 2, span)
}

fn bi_prompt_structured_with(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let (proto_name, fields) = extract_protocol(&args[0], span)?;
    let Value::Record(opts) = &args[1] else {
        return Err(LxError::type_err(
            "ai.prompt_structured_with: second arg must be options Record",
            span,
        ));
    };
    let prompt = opts.get("prompt").and_then(|v| v.as_str()).ok_or_else(|| {
        LxError::runtime(
            "ai.prompt_structured_with: opts must have 'prompt' (Str)",
            span,
        )
    })?;
    let max_retries = opts
        .get("max_retries")
        .and_then(|v| v.as_int())
        .and_then(|n| n.to_usize())
        .unwrap_or(2);
    let ai_opts = super::ai::extract_opts(opts);
    let augmented = augment_prompt(prompt, &proto_name, &fields);
    run_structured(ctx, &augmented, &ai_opts, &fields, max_retries, span)
}

fn extract_protocol(val: &Value, span: Span) -> Result<(String, Arc<Vec<ProtoFieldDef>>), LxError> {
    match val {
        Value::Trait { name, fields, .. } if !fields.is_empty() => {
            Ok((name.to_string(), Arc::clone(fields)))
        }
        _ => Err(LxError::type_err(
            "ai.prompt_structured: first arg must be a Protocol",
            span,
        )),
    }
}

fn augment_prompt(prompt: &str, _name: &str, fields: &[ProtoFieldDef]) -> String {
    let mut schema = String::from("{\n");
    for f in fields {
        let ts = type_to_schema(&f.type_name);
        schema.push_str(&format!("  \"{}\": {}", f.name, ts));
        if f.default.is_some() {
            schema.push_str(" (optional)");
        }
        schema.push('\n');
    }
    schema.push('}');
    format!("{prompt}\n\nRespond with a JSON object matching this exact schema:\n{schema}")
}

fn type_to_schema(type_name: &str) -> String {
    match type_name {
        "Str" => "string".into(),
        "Int" => "integer".into(),
        "Float" => "number (float)".into(),
        "Bool" => "boolean".into(),
        "List" => "[any]".into(),
        "Any" => "any".into(),
        other => format!("{other} (object)"),
    }
}

fn run_structured(
    ctx: &Arc<RuntimeCtx>,
    prompt: &str,
    opts: &AiOpts,
    fields: &[ProtoFieldDef],
    max_retries: usize,
    span: Span,
) -> Result<Value, LxError> {
    let mut current_prompt = prompt.to_string();
    for attempt in 0..=max_retries {
        let response = ctx.ai.prompt(&current_prompt, opts, span)?;
        match try_parse_and_validate(&response, fields, span) {
            Ok(record) => return Ok(Value::Ok(Box::new(record))),
            Err(reason) => {
                if attempt == max_retries {
                    let raw_text = super::ai::extract_llm_text(&response).unwrap_or_else(|e| e);
                    return Ok(Value::Err(Box::new(record! {
                        "reason" => Value::Str(Arc::from(reason.as_str())),
                        "raw" => Value::Str(Arc::from(raw_text.as_str())),
                        "attempts" => Value::Int((attempt + 1).into()),
                    })));
                }
                current_prompt = format!(
                    "{prompt}\n\nYour previous response did not match the required schema.\nError: {reason}\nPlease respond again with a valid JSON object matching the schema."
                );
            }
        }
    }
    unreachable!()
}

fn try_parse_and_validate(
    response: &Value,
    fields: &[ProtoFieldDef],
    span: Span,
) -> Result<Value, String> {
    let text = super::ai::extract_llm_text(response)?;
    let stripped = super::ai::strip_json_fences(&text);
    let jv: serde_json::Value =
        serde_json::from_str(stripped).map_err(|e| format!("invalid JSON: {e}"))?;
    let lx_val = json_conv::json_to_lx(jv);
    let Value::Record(rec) = &lx_val else {
        return Err("response is not a JSON object".into());
    };
    validate_fields(rec, fields, span)?;
    Ok(lx_val)
}

fn validate_fields(
    rec: &IndexMap<String, Value>,
    fields: &[ProtoFieldDef],
    _span: Span,
) -> Result<(), String> {
    for f in fields {
        match rec.get(&f.name) {
            Some(val) => {
                if f.type_name != "Any" && val.type_name() != f.type_name {
                    return Err(format!(
                        "field '{}' expected {}, got {}",
                        f.name,
                        f.type_name,
                        val.type_name()
                    ));
                }
            }
            None => {
                if f.default.is_none() {
                    return Err(format!("missing required field '{}'", f.name));
                }
            }
        }
    }
    Ok(())
}
