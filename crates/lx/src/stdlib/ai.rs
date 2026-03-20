use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::{AiOpts, EmbedOpts, RuntimeCtx};
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("prompt".into(), mk("ai.prompt", 1, bi_prompt));
    m.insert(
        "prompt_with".into(),
        mk("ai.prompt_with", 1, bi_prompt_with),
    );
    m.insert(
        "prompt_structured".into(),
        super::ai_structured::mk_prompt_structured(),
    );
    m.insert(
        "prompt_structured_with".into(),
        super::ai_structured::mk_prompt_structured_with(),
    );
    m.insert("prompt_json".into(), super::ai_structured::mk_prompt_json());
    m.insert("embed".into(), mk("ai.embed", 1, bi_embed));
    m.insert("embed_with".into(), mk("ai.embed_with", 1, bi_embed_with));
    m
}

pub(crate) fn extract_llm_text(response: &Value) -> Result<String, String> {
    match response {
        Value::Ok(inner) => match inner.as_ref() {
            Value::Record(f) => Ok(f
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()),
            Value::Str(s) => Ok(s.to_string()),
            _ => Err("LLM returned unexpected format".to_string()),
        },
        Value::Err(e) => {
            let msg = match e.as_ref() {
                Value::Str(s) => s.to_string(),
                Value::Record(r) => r
                    .get("msg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string(),
                _ => "LLM error".to_string(),
            };
            Err(format!("LLM error: {msg}"))
        }
        _ => Err("LLM returned unexpected value".to_string()),
    }
}

pub(crate) fn strip_json_fences(text: &str) -> &str {
    let trimmed = text.trim();
    trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(|s| s.trim())
        .unwrap_or(trimmed)
}

pub(crate) fn parse_llm_json(
    response: &Value,
    context: &str,
    span: Span,
) -> Result<Result<serde_json::Value, String>, LxError> {
    let text = match extract_llm_text(response) {
        Ok(t) => t,
        Err(msg) => return Ok(Err(msg)),
    };
    if text.trim().is_empty() {
        return Ok(Err(format!("{context}: empty LLM response")));
    }
    let jv = serde_json::from_str::<serde_json::Value>(text.trim())
        .or_else(|_| serde_json::from_str(strip_json_fences(&text)))
        .map_err(|e| LxError::runtime(format!("{context}: JSON parse: {e}"), span))?;
    Ok(Ok(jv))
}

fn opt_str(fields: &IndexMap<String, Value>, key: &str) -> Option<String> {
    fields
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub(crate) fn extract_opts(fields: &IndexMap<String, Value>) -> AiOpts {
    AiOpts {
        system: opt_str(fields, "system"),
        model: opt_str(fields, "model"),
        max_turns: fields
            .get("max_turns")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok()),
        resume: opt_str(fields, "resume"),
        tools: fields.get("tools").and_then(|v| v.as_list()).map(|l| {
            l.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        }),
        append_system: opt_str(fields, "append_system"),
    }
}

fn bi_prompt(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let prompt = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("ai.prompt expects Str", span))?;
    let result = ctx.ai.prompt(prompt, &AiOpts::default(), span)?;
    match result {
        Value::Ok(inner) => {
            if let Value::Record(ref fields) = *inner {
                let text = fields
                    .get("text")
                    .cloned()
                    .unwrap_or(Value::Str(Arc::from("")));
                Ok(Value::Ok(Box::new(text)))
            } else {
                Ok(Value::Ok(inner))
            }
        }
        other => Ok(other),
    }
}

fn bi_prompt_with(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("ai.prompt_with expects Record", span));
    };
    let prompt = fields
        .get("prompt")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            LxError::runtime(
                "ai.prompt_with: record must have 'prompt' field (Str)",
                span,
            )
        })?;
    ctx.ai.prompt(prompt, &extract_opts(fields), span)
}

fn bi_embed(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let texts = args[0]
        .as_list()
        .ok_or_else(|| LxError::type_err("ai.embed expects List of Str", span))?;
    let strings: Vec<String> = texts
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    ctx.embed.embed(&strings, &EmbedOpts::default(), span)
}

fn bi_embed_with(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("ai.embed_with expects Record", span));
    };
    let texts = fields
        .get("texts")
        .and_then(|v| v.as_list())
        .ok_or_else(|| {
            LxError::runtime(
                "ai.embed_with: record must have 'texts' field (List of Str)",
                span,
            )
        })?;
    let strings: Vec<String> = texts
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let opts = EmbedOpts {
        model: opt_str(fields, "model"),
        dimensions: fields
            .get("dimensions")
            .and_then(|v| v.as_int())
            .and_then(|n| usize::try_from(n).ok()),
    };
    ctx.embed.embed(&strings, &opts, span)
}
