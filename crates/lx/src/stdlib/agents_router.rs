use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::ai;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("route".into(), mk("router.route", 1, bi_route));
    m.insert("quick_route".into(), mk("router.quick_route", 1, bi_quick_route));
    m
}

struct RouteFields {
    prompt: String,
    catalog: Vec<CatalogEntry>,
}

struct CatalogEntry {
    name: String,
    domain: String,
    description: String,
    terminal: bool,
}

fn extract_fields(args: &[Value], span: Span) -> Result<RouteFields, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("router expects Record", span));
    };
    let prompt = fields.get("prompt")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("router: missing 'prompt' (Str)", span))?
        .to_string();
    let catalog_list = fields.get("catalog")
        .and_then(|v| v.as_list())
        .ok_or_else(|| LxError::runtime("router: missing 'catalog' (List)", span))?;
    let mut catalog = Vec::new();
    for entry in catalog_list.iter() {
        let Value::Record(r) = entry else {
            return Err(LxError::type_err("router: catalog entry must be Record", span));
        };
        catalog.push(CatalogEntry {
            name: r.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            domain: r.get("domain").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: r.get("description").and_then(|v| v.as_str())
                .unwrap_or("").to_string(),
            terminal: r.get("terminal").and_then(|v| v.as_bool()).unwrap_or(false),
        });
    }
    Ok(RouteFields { prompt, catalog })
}

fn build_result(domain: &str, agent: &str, confidence: f64, terminal: bool) -> Value {
    let mut r = IndexMap::new();
    r.insert("domain".into(), Value::Str(Arc::from(domain)));
    r.insert("agent".into(), Value::Str(Arc::from(agent)));
    r.insert("confidence".into(), Value::Float(confidence));
    r.insert("terminal".into(), Value::Bool(terminal));
    Value::Record(Arc::new(r))
}

fn no_match() -> Value {
    build_result("none", "", 0.0, false)
}

fn build_system_prompt() -> String {
    String::from(
        "You are a prompt router. Given a user prompt and a catalog of specialist agents, \
         classify which agent should handle the prompt.\n\n\
         Respond with ONLY a JSON object, no markdown fences:\n\
         {\"domain\": \"...\", \"agent\": \"...\", \"confidence\": 0.0-1.0}\n\n\
         If no agent matches well, respond: {\"domain\": \"none\", \"agent\": \"\", \"confidence\": 0.0}\n\
         Confidence 0.8+ means strong match. 0.5-0.8 partial. Below 0.5 means no good match.",
    )
}

fn build_catalog_text(catalog: &[CatalogEntry]) -> String {
    let mut text = String::from("CATALOG:\n");
    for (i, entry) in catalog.iter().enumerate() {
        text.push_str(&format!(
            "{}. name={} domain={} description={}\n",
            i + 1, entry.name, entry.domain, entry.description,
        ));
    }
    text
}

fn build_user_prompt(fields: &RouteFields) -> String {
    let mut p = build_catalog_text(&fields.catalog);
    p.push_str(&format!("\nPROMPT TO ROUTE:\n{}", fields.prompt));
    p
}

fn parse_llm_result(llm_response: &Value, catalog: &[CatalogEntry], span: Span)
    -> Result<Value, LxError>
{
    let Ok(jv) = ai::parse_llm_json(llm_response, "router", span)? else {
        return Ok(no_match());
    };
    let domain = jv.get("domain").and_then(|v| v.as_str()).unwrap_or("none");
    let agent = jv.get("agent").and_then(|v| v.as_str()).unwrap_or("");
    let confidence = jv.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let terminal = catalog.iter()
        .find(|e| e.name == agent || e.domain == domain)
        .map(|e| e.terminal)
        .unwrap_or(false);
    Ok(build_result(domain, agent, confidence, terminal))
}

fn bi_route(args: &[Value], span: Span) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    if fields.catalog.is_empty() {
        return Ok(no_match());
    }
    let system = build_system_prompt();
    let user = build_user_prompt(&fields);
    let opts = ai::Opts {
        system: Some(system),
        max_turns: Some(1),
        ..ai::default_opts()
    };
    let llm_result = ai::run_claude(&user, &opts, span)?;
    parse_llm_result(&llm_result, &fields.catalog, span)
}

fn keyword_score(prompt: &str, entry: &CatalogEntry) -> f64 {
    let prompt_lower = prompt.to_lowercase();
    let mut words = Vec::new();
    for w in entry.description.split_whitespace() {
        if w.len() > 3 {
            words.push(w.to_lowercase());
        }
    }
    for w in entry.domain.split_whitespace() {
        if w.len() > 2 {
            words.push(w.to_lowercase());
        }
    }
    for w in entry.name.split_whitespace() {
        if w.len() > 2 {
            words.push(w.to_lowercase());
        }
    }
    if words.is_empty() {
        return 0.0;
    }
    let hits = words.iter().filter(|w| prompt_lower.contains(w.as_str())).count();
    hits as f64 / words.len() as f64
}

fn bi_quick_route(args: &[Value], span: Span) -> Result<Value, LxError> {
    let fields = extract_fields(args, span)?;
    if fields.catalog.is_empty() {
        return Ok(no_match());
    }
    let mut best_score = 0.0f64;
    let mut best_idx = 0usize;
    for (i, entry) in fields.catalog.iter().enumerate() {
        let score = keyword_score(&fields.prompt, entry);
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    if best_score < 0.1 {
        return Ok(no_match());
    }
    let entry = &fields.catalog[best_idx];
    Ok(build_result(&entry.domain, &entry.name, best_score, entry.terminal))
}
