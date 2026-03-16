use std::sync::Arc;

use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::{PromptState, get_state, prompt_id, store_and_handle};

pub(super) fn render_state(state: &PromptState) -> String {
    let mut parts = Vec::new();
    if let Some(ref sys) = state.system {
        parts.push(sys.clone());
    }
    for sec in &state.sections {
        parts.push(format!("{}:\n{}", capitalize(&sec.name), sec.content));
    }
    if !state.constraints.is_empty() {
        let items: Vec<String> = state.constraints.iter().map(|c| format!("- {c}")).collect();
        parts.push(format!("Constraints:\n{}", items.join("\n")));
    }
    if !state.instructions.is_empty() {
        let items: Vec<String> = state
            .instructions
            .iter()
            .map(|i| format!("- {i}"))
            .collect();
        parts.push(format!("Instructions:\n{}", items.join("\n")));
    }
    if !state.examples.is_empty() {
        let mut ex_parts = Vec::new();
        for (i, ex) in state.examples.iter().enumerate() {
            ex_parts.push(format!(
                "Example {}:\nInput: {}\nOutput: {}",
                i + 1,
                ex.input,
                ex.output
            ));
        }
        parts.push(format!("Examples:\n{}", ex_parts.join("\n\n")));
    }
    parts.join("\n\n")
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub(super) fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

pub(super) fn bi_render(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = prompt_id(&args[0], span)?;
    let state = get_state(id, span)?;
    let text = render_state(&state);
    Ok(Value::Str(Arc::from(text.as_str())))
}

pub(super) fn bi_render_within(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = prompt_id(&args[0], span)?;
    let max_tokens: usize = match &args[1] {
        Value::Int(n) => n
            .try_into()
            .map_err(|_| LxError::type_err("prompt.render_within: bad token limit", span))?,
        Value::Float(f) => *f as usize,
        _ => {
            return Err(LxError::type_err(
                "prompt.render_within: expects Int token limit",
                span,
            ));
        }
    };
    let mut state = get_state(id, span)?;
    let text = render_state(&state);
    if estimate_tokens(&text) <= max_tokens {
        return Ok(Value::Str(Arc::from(text.as_str())));
    }
    while !state.examples.is_empty() {
        state.examples.pop();
        let text = render_state(&state);
        if estimate_tokens(&text) <= max_tokens {
            return Ok(Value::Str(Arc::from(text.as_str())));
        }
    }
    while !state.constraints.is_empty() {
        state.constraints.pop();
        let text = render_state(&state);
        if estimate_tokens(&text) <= max_tokens {
            return Ok(Value::Str(Arc::from(text.as_str())));
        }
    }
    let text = render_state(&state);
    Ok(Value::Str(Arc::from(text.as_str())))
}

pub(super) fn bi_estimate(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = prompt_id(&args[0], span)?;
    let state = get_state(id, span)?;
    let text = render_state(&state);
    Ok(Value::Int(BigInt::from(estimate_tokens(&text))))
}

pub(super) fn bi_sections(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = prompt_id(&args[0], span)?;
    let state = get_state(id, span)?;
    let mut names = Vec::new();
    if state.system.is_some() {
        names.push(Value::Str(Arc::from("system")));
    }
    for sec in &state.sections {
        names.push(Value::Str(Arc::from(sec.name.as_str())));
    }
    if !state.constraints.is_empty() {
        names.push(Value::Str(Arc::from("constraints")));
    }
    if !state.instructions.is_empty() {
        names.push(Value::Str(Arc::from("instructions")));
    }
    if !state.examples.is_empty() {
        names.push(Value::Str(Arc::from("examples")));
    }
    Ok(Value::List(Arc::new(names)))
}

pub(super) fn bi_without(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = prompt_id(&args[0], span)?;
    let mut state = get_state(id, span)?;
    let names: Vec<String> = match &args[1] {
        Value::Str(s) => vec![s.to_string()],
        Value::List(items) => items
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => {
            return Err(LxError::type_err(
                "prompt.without: expects Str or List of Str",
                span,
            ));
        }
    };
    for name in &names {
        match name.as_str() {
            "system" => state.system = None,
            "constraints" => state.constraints.clear(),
            "instructions" => state.instructions.clear(),
            "examples" => state.examples.clear(),
            other => state.sections.retain(|s| s.name != other),
        }
    }
    Ok(store_and_handle(state))
}
