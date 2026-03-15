use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("prompt".into(), mk("ai.prompt", 1, bi_prompt));
    m.insert("prompt_with".into(), mk("ai.prompt_with", 1, bi_prompt_with));
    m
}

pub(crate) struct Opts {
    pub(crate) system: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) max_turns: Option<i64>,
    pub(crate) resume: Option<String>,
    pub(crate) tools: Option<Vec<String>>,
    pub(crate) append_system: Option<String>,
}

fn build_command(opts: &Opts) -> Command {
    let mut cmd = Command::new("claude");
    cmd.arg("-p").arg("--output-format").arg("json");
    if let Some(ref s) = opts.system {
        cmd.arg("--system-prompt").arg(s);
    }
    if let Some(ref m) = opts.model {
        cmd.arg("--model").arg(m);
    }
    if let Some(n) = opts.max_turns {
        cmd.arg("--max-turns").arg(n.to_string());
    }
    if let Some(ref id) = opts.resume {
        cmd.arg("--resume").arg(id);
    }
    if let Some(ref t) = opts.tools {
        cmd.arg("--allowedTools").arg(t.join(","));
    }
    if let Some(ref s) = opts.append_system {
        cmd.arg("--append-system-prompt").arg(s);
    }
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd
}

pub(crate) fn run_claude(prompt: &str, opts: &Opts, span: Span) -> Result<Value, LxError> {
    let mut cmd = build_command(opts);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return Ok(Value::Err(Box::new(Value::Str(
            Arc::from(format!("ai: cannot run 'claude': {e}").as_str())
        )))),
    };
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes())
            .map_err(|e| LxError::runtime(format!("ai: stdin write: {e}"), span))?;
    }
    let output = child.wait_with_output()
        .map_err(|e| LxError::runtime(format!("ai: wait: {e}"), span))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() && stdout.trim().is_empty() {
        return Ok(Value::Err(Box::new(Value::Str(
            Arc::from(format!("ai: claude exited {}: {stderr}", output.status).as_str())
        ))));
    }
    let jv: serde_json::Value = serde_json::from_str(stdout.trim())
        .map_err(|e| LxError::runtime(
            format!("ai: JSON parse: {e}\nstdout: {stdout}\nstderr: {stderr}"), span,
        ))?;
    parse_response(&jv)
}

fn parse_response(jv: &serde_json::Value) -> Result<Value, LxError> {
    let is_error = jv.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
    let result_text = jv.get("result").and_then(|v| v.as_str()).unwrap_or("");
    if is_error {
        let mut fields = IndexMap::new();
        fields.insert("msg".into(), Value::Str(Arc::from(result_text)));
        if let Some(sub) = jv.get("subtype").and_then(|v| v.as_str()) {
            fields.insert("subtype".into(), Value::Str(Arc::from(sub)));
        }
        return Ok(Value::Err(Box::new(Value::Record(Arc::new(fields)))));
    }
    let mut fields = IndexMap::new();
    fields.insert("text".into(), Value::Str(Arc::from(result_text)));
    if let Some(sid) = jv.get("session_id").and_then(|v| v.as_str()) {
        fields.insert("session_id".into(), Value::Str(Arc::from(sid)));
    }
    if let Some(cost) = jv.get("cost_usd").and_then(|v| v.as_f64()) {
        fields.insert("cost".into(), Value::Float(cost));
    }
    if let Some(turns) = jv.get("num_turns").and_then(|v| v.as_i64()) {
        fields.insert("turns".into(), Value::Int(BigInt::from(turns)));
    }
    if let Some(ms) = jv.get("duration_ms").and_then(|v| v.as_i64()) {
        fields.insert("duration_ms".into(), Value::Int(BigInt::from(ms)));
    }
    if let Some(model) = jv.get("model").and_then(|v| v.as_str()) {
        fields.insert("model".into(), Value::Str(Arc::from(model)));
    }
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(fields)))))
}

pub(crate) fn extract_llm_text(response: &Value) -> Result<String, String> {
    match response {
        Value::Ok(inner) => match inner.as_ref() {
            Value::Record(f) => Ok(f.get("text")
                .and_then(|v| v.as_str()).unwrap_or("").to_string()),
            Value::Str(s) => Ok(s.to_string()),
            _ => Err("LLM returned unexpected format".to_string()),
        },
        Value::Err(e) => {
            let msg = match e.as_ref() {
                Value::Str(s) => s.to_string(),
                Value::Record(r) => r.get("msg")
                    .and_then(|v| v.as_str()).unwrap_or("unknown error").to_string(),
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
        .strip_prefix("```json").or_else(|| trimmed.strip_prefix("```"))
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
    let jv = serde_json::from_str::<serde_json::Value>(text.trim())
        .or_else(|_| serde_json::from_str(strip_json_fences(&text)))
        .map_err(|e| LxError::runtime(format!("{context}: JSON parse: {e}"), span))?;
    Ok(Ok(jv))
}

pub(crate) fn default_opts() -> Opts {
    Opts {
        system: None,
        model: None,
        max_turns: None,
        resume: None,
        tools: None,
        append_system: None,
    }
}

fn str_field(fields: &IndexMap<String, Value>, key: &str) -> Option<String> {
    fields.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn extract_opts(fields: &IndexMap<String, Value>) -> Opts {
    Opts {
        system: str_field(fields, "system"),
        model: str_field(fields, "model"),
        max_turns: fields.get("max_turns")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok()),
        resume: str_field(fields, "resume"),
        tools: fields.get("tools").and_then(|v| v.as_list()).map(|l| {
            l.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
        }),
        append_system: str_field(fields, "append_system"),
    }
}

fn bi_prompt(args: &[Value], span: Span) -> Result<Value, LxError> {
    let prompt = args[0].as_str()
        .ok_or_else(|| LxError::type_err("ai.prompt expects Str", span))?;
    let result = run_claude(prompt, &default_opts(), span)?;
    match result {
        Value::Ok(inner) => {
            if let Value::Record(ref fields) = *inner {
                let text = fields.get("text").cloned()
                    .unwrap_or(Value::Str(Arc::from("")));
                Ok(Value::Ok(Box::new(text)))
            } else {
                Ok(Value::Ok(inner))
            }
        }
        other => Ok(other),
    }
}

fn bi_prompt_with(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("ai.prompt_with expects Record", span));
    };
    let prompt = fields.get("prompt")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime(
            "ai.prompt_with: record must have 'prompt' field (Str)", span,
        ))?;
    run_claude(prompt, &extract_opts(fields), span)
}
