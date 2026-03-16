use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

#[derive(Clone)]
struct Section {
    name: String,
    content: String,
}

#[derive(Clone)]
struct Example {
    input: String,
    output: String,
}

#[derive(Clone)]
struct PromptState {
    system: Option<String>,
    sections: Vec<Section>,
    constraints: Vec<String>,
    instructions: Vec<String>,
    examples: Vec<Example>,
}

impl PromptState {
    fn new() -> Self {
        Self {
            system: None,
            sections: Vec::new(),
            constraints: Vec::new(),
            instructions: Vec::new(),
            examples: Vec::new(),
        }
    }
}

static PROMPTS: LazyLock<DashMap<u64, PromptState>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("prompt.create", 1, bi_create));
    m.insert("system".into(), mk("prompt.system", 2, bi_system));
    m.insert("section".into(), mk("prompt.section", 3, bi_section));
    m.insert("constraint".into(), mk("prompt.constraint", 2, bi_constraint));
    m.insert("instruction".into(), mk("prompt.instruction", 2, bi_instruction));
    m.insert("example".into(), mk("prompt.example", 2, bi_example));
    m.insert("compose".into(), mk("prompt.compose", 1, bi_compose));
    m.insert("render".into(), mk("prompt.render", 1, bi_render));
    m.insert("render_within".into(), mk("prompt.render_within", 2, bi_render_within));
    m.insert("estimate".into(), mk("prompt.estimate", 1, bi_estimate));
    m.insert("sections".into(), mk("prompt.sections", 1, bi_sections));
    m.insert("without".into(), mk("prompt.without", 2, bi_without));
    m
}

fn prompt_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__prompt_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("prompt: expected prompt handle", span)),
        _ => Err(LxError::type_err("prompt: expected prompt Record", span)),
    }
}

fn store_and_handle(state: PromptState) -> Value {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    PROMPTS.insert(id, state);
    let mut rec = IndexMap::new();
    rec.insert("__prompt_id".into(), Value::Int(BigInt::from(id)));
    Value::Record(Arc::new(rec))
}

fn get_state(id: u64, span: Span) -> Result<PromptState, LxError> {
    PROMPTS
        .get(&id)
        .map(|s| s.clone())
        .ok_or_else(|| LxError::runtime("prompt: not found", span))
}

fn bi_create(_args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    Ok(store_and_handle(PromptState::new()))
}

fn bi_system(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let text = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("prompt.system: expects Str", span))?;
    let id = prompt_id(&args[1], span)?;
    let mut state = get_state(id, span)?;
    state.system = Some(text.to_string());
    Ok(store_and_handle(state))
}

fn bi_section(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("prompt.section: name must be Str", span))?;
    let content = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("prompt.section: content must be Str", span))?;
    let id = prompt_id(&args[2], span)?;
    let mut state = get_state(id, span)?;
    state.sections.push(Section {
        name: name.to_string(),
        content: content.to_string(),
    });
    Ok(store_and_handle(state))
}

fn bi_constraint(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let text = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("prompt.constraint: expects Str", span))?;
    let id = prompt_id(&args[1], span)?;
    let mut state = get_state(id, span)?;
    state.constraints.push(text.to_string());
    Ok(store_and_handle(state))
}

fn bi_instruction(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let text = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("prompt.instruction: expects Str", span))?;
    let id = prompt_id(&args[1], span)?;
    let mut state = get_state(id, span)?;
    state.instructions.push(text.to_string());
    Ok(store_and_handle(state))
}

fn bi_example(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(ex) = &args[0] else {
        return Err(LxError::type_err("prompt.example: expects Record {input output}", span));
    };
    let input = ex
        .get("input")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::type_err("prompt.example: input must be Str", span))?;
    let output = ex
        .get("output")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::type_err("prompt.example: output must be Str", span))?;
    let id = prompt_id(&args[1], span)?;
    let mut state = get_state(id, span)?;
    state.examples.push(Example {
        input: input.to_string(),
        output: output.to_string(),
    });
    Ok(store_and_handle(state))
}

fn bi_compose(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::List(prompts) = &args[0] else {
        return Err(LxError::type_err("prompt.compose: expects List of prompts", span));
    };
    let mut merged = PromptState::new();
    for p in prompts.iter() {
        let id = prompt_id(p, span)?;
        let state = get_state(id, span)?;
        if let Some(sys) = &state.system {
            merged.system = Some(match &merged.system {
                Some(existing) => format!("{existing}\n{sys}"),
                None => sys.clone(),
            });
        }
        merged.sections.extend(state.sections);
        merged.constraints.extend(state.constraints);
        merged.instructions.extend(state.instructions);
        merged.examples.extend(state.examples);
    }
    Ok(store_and_handle(merged))
}

fn render_state(state: &PromptState) -> String {
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

fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

fn bi_render(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = prompt_id(&args[0], span)?;
    let state = get_state(id, span)?;
    let text = render_state(&state);
    Ok(Value::Str(Arc::from(text.as_str())))
}

fn bi_render_within(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
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
            ))
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

fn bi_estimate(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = prompt_id(&args[0], span)?;
    let state = get_state(id, span)?;
    let text = render_state(&state);
    Ok(Value::Int(BigInt::from(estimate_tokens(&text))))
}

fn bi_sections(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
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

fn bi_without(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
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
            ))
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
