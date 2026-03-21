use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::{AiOpts, EmbedOpts, RuntimeCtx};
use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::mk;

pub(super) fn register_ai(env: &mut crate::env::Env) {
    let mut ai_fields = IndexMap::new();
    ai_fields.insert("prompt".into(), mk("ai.prompt", 1, bi_prompt));
    ai_fields.insert("prompt_with".into(), mk("ai.prompt_with", 1, bi_prompt_with));
    ai_fields.insert(
        "prompt_structured".into(),
        mk("ai.prompt_structured", 2, bi_prompt_structured),
    );
    ai_fields.insert(
        "prompt_structured_with".into(),
        mk("ai.prompt_structured_with", 2, bi_prompt_structured),
    );
    ai_fields.insert("prompt_json".into(), mk("ai.prompt_json", 2, bi_prompt_json));
    ai_fields.insert("embed".into(), mk("ai.embed", 1, bi_embed));
    ai_fields.insert("embed_with".into(), mk("ai.embed_with", 1, bi_embed_with));
    env.bind("ai".into(), LxVal::Record(Arc::new(ai_fields)));
}

fn opt_str(fields: &IndexMap<String, LxVal>, key: &str) -> Option<String> {
    fields.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn extract_opts(fields: &IndexMap<String, LxVal>) -> AiOpts {
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
        disable_tools: fields
            .get("disable_tools")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        json_schema: opt_str(fields, "json_schema"),
    }
}

fn bi_prompt(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let prompt = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("ai.prompt expects Str", span))?;
    ctx.ai.prompt(prompt, &AiOpts::default(), span)
}

fn bi_prompt_with(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let LxVal::Record(fields) = &args[0] else {
        return Err(LxError::type_err("ai.prompt_with expects Record", span));
    };
    let prompt = fields
        .get("prompt")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            LxError::runtime("ai.prompt_with: record must have 'prompt' field (Str)", span)
        })?;
    ctx.ai.prompt(prompt, &extract_opts(fields), span)
}

fn bi_embed(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let texts = args[0]
        .as_list()
        .ok_or_else(|| LxError::type_err("ai.embed expects List of Str", span))?;
    let strings: Vec<String> = texts
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    ctx.embed.embed(&strings, &EmbedOpts::default(), span)
}

fn bi_embed_with(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let LxVal::Record(fields) = &args[0] else {
        return Err(LxError::type_err("ai.embed_with expects Record", span));
    };
    let texts = fields
        .get("texts")
        .and_then(|v| v.as_list())
        .ok_or_else(|| {
            LxError::runtime("ai.embed_with: record must have 'texts' field (List of Str)", span)
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

fn bi_prompt_structured(
    args: &[LxVal],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let trait_val = &args[0];
    let prompt_text = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("ai.prompt_structured expects Str as second arg", span))?;
    let schema = serde_json::to_string(&serde_json::Value::from(trait_val))
        .unwrap_or_default();
    let opts = AiOpts {
        json_schema: Some(schema),
        ..AiOpts::default()
    };
    ctx.ai.prompt(prompt_text, &opts, span)
}

fn bi_prompt_json(
    args: &[LxVal],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let prompt_text = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("ai.prompt_json expects Str as first arg", span))?;
    let shape = serde_json::to_string(&serde_json::Value::from(&args[1]))
        .unwrap_or_default();
    let opts = AiOpts {
        json_schema: Some(shape),
        ..AiOpts::default()
    };
    ctx.ai.prompt(prompt_text, &opts, span)
}
