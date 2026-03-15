use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;

use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv::{json_to_lx, lx_to_json};
use crate::value::Value;

use super::{
    AiBackend, AiOpts, EmitBackend, HttpBackend, HttpOpts, LogBackend, LogLevel,
    ShellBackend, YieldBackend,
};

pub struct ClaudeCodeAiBackend;

impl AiBackend for ClaudeCodeAiBackend {
    fn prompt(&self, text: &str, opts: &AiOpts, span: Span) -> Result<Value, LxError> {
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
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return Ok(Value::Err(Box::new(Value::Str(
                Arc::from(format!("ai: cannot run 'claude': {e}").as_str()),
            )))),
        };
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())
                .map_err(|e| LxError::runtime(format!("ai: stdin write: {e}"), span))?;
        }
        let output = child.wait_with_output()
            .map_err(|e| LxError::runtime(format!("ai: wait: {e}"), span))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() && stdout.trim().is_empty() {
            return Ok(Value::Err(Box::new(Value::Str(
                Arc::from(format!("ai: claude exited {}: {stderr}", output.status).as_str()),
            ))));
        }
        let jv: serde_json::Value = serde_json::from_str(stdout.trim())
            .map_err(|e| LxError::runtime(
                format!("ai: JSON parse: {e}\nstdout: {stdout}\nstderr: {stderr}"), span,
            ))?;
        parse_ai_response(&jv)
    }
}

fn parse_ai_response(jv: &serde_json::Value) -> Result<Value, LxError> {
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

pub struct StdoutEmitBackend;

impl EmitBackend for StdoutEmitBackend {
    fn emit(&self, value: &Value, _span: Span) -> Result<(), LxError> {
        println!("{value}");
        Ok(())
    }
}

pub struct ReqwestHttpBackend;

impl HttpBackend for ReqwestHttpBackend {
    fn request(
        &self,
        method: &str,
        url: &str,
        opts: &HttpOpts,
        span: Span,
    ) -> Result<Value, LxError> {
        let c = Client::builder().build()
            .map_err(|e| LxError::runtime(format!("http: client: {e}"), span))?;
        let mut builder = match method {
            "GET" => c.get(url),
            "POST" => c.post(url),
            "PUT" => c.put(url),
            "DELETE" => c.delete(url),
            _ => return Err(LxError::runtime(format!("http: unknown method '{method}'"), span)),
        };
        if let Some(ref hdrs) = opts.headers {
            for (k, v) in hdrs {
                builder = builder.header(k.as_str(), v.as_str());
            }
        }
        if let Some(ref query) = opts.query {
            let pairs: Vec<(&str, &str)> = query.iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            builder = builder.query(&pairs);
        }
        if let Some(ref body) = opts.body {
            builder = builder.header(CONTENT_TYPE, "application/json").json(body);
        }
        match builder.send() {
            Ok(resp) => response_to_value(resp, span),
            Err(e) => Ok(Value::Err(Box::new(Value::Str(
                Arc::from(e.to_string().as_str()),
            )))),
        }
    }
}

fn response_to_value(resp: reqwest::blocking::Response, span: Span) -> Result<Value, LxError> {
    let status = resp.status().as_u16();
    let mut headers = IndexMap::new();
    for (name, value) in resp.headers() {
        let v = value.to_str().unwrap_or("").to_string();
        headers.insert(name.to_string(), Value::Str(Arc::from(v.as_str())));
    }
    let body_str = resp.text()
        .map_err(|e| LxError::runtime(format!("http: body: {e}"), span))?;
    let body = if let Ok(jv) = serde_json::from_str::<serde_json::Value>(&body_str) {
        json_to_lx(jv)
    } else {
        Value::Str(Arc::from(body_str.as_str()))
    };
    let mut fields = IndexMap::new();
    fields.insert("status".into(), Value::Int(BigInt::from(status)));
    fields.insert("body".into(), body);
    fields.insert("headers".into(), Value::Record(Arc::new(headers)));
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(fields)))))
}

pub struct ProcessShellBackend;

impl ShellBackend for ProcessShellBackend {
    fn exec(&self, cmd: &str, _span: Span) -> Result<Value, LxError> {
        match Command::new("sh").arg("-c").arg(cmd).output() {
            Ok(output) => {
                let out = String::from_utf8_lossy(&output.stdout).into_owned();
                let err = String::from_utf8_lossy(&output.stderr).into_owned();
                let code = output.status.code().unwrap_or(-1);
                let mut fields = IndexMap::new();
                fields.insert("out".into(), Value::Str(Arc::from(out.as_str())));
                fields.insert("err".into(), Value::Str(Arc::from(err.as_str())));
                fields.insert("code".into(), Value::Int(code.into()));
                Ok(Value::Ok(Box::new(Value::Record(Arc::new(fields)))))
            }
            Err(e) => {
                let mut fields = IndexMap::new();
                fields.insert("cmd".into(), Value::Str(Arc::from(cmd)));
                fields.insert("msg".into(), Value::Str(Arc::from(e.to_string().as_str())));
                Ok(Value::Err(Box::new(Value::Record(Arc::new(fields)))))
            }
        }
    }

    fn exec_capture(&self, cmd: &str, span: Span) -> Result<Value, LxError> {
        match Command::new("sh").arg("-c").arg(cmd).output() {
            Ok(output) => {
                let code = output.status.code().unwrap_or(-1);
                if code == 0 {
                    let out = String::from_utf8_lossy(&output.stdout).into_owned();
                    Ok(Value::Str(Arc::from(out.as_str())))
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).into_owned();
                    let mut fields = IndexMap::new();
                    fields.insert("cmd".into(), Value::Str(Arc::from(cmd)));
                    fields.insert("msg".into(), Value::Str(Arc::from(err.as_str())));
                    let shell_err = Value::Err(Box::new(Value::Record(Arc::new(fields))));
                    Err(LxError::propagate(shell_err, span))
                }
            }
            Err(e) => {
                let mut fields = IndexMap::new();
                fields.insert("cmd".into(), Value::Str(Arc::from(cmd)));
                fields.insert("msg".into(), Value::Str(Arc::from(e.to_string().as_str())));
                let shell_err = Value::Err(Box::new(Value::Record(Arc::new(fields))));
                Err(LxError::propagate(shell_err, span))
            }
        }
    }
}

pub struct StdinStdoutYieldBackend;

impl YieldBackend for StdinStdoutYieldBackend {
    fn yield_value(&self, value: Value, span: Span) -> Result<Value, LxError> {
        use std::io::BufRead;
        let json = lx_to_json(&value, span)
            .map_err(|e| LxError::runtime(format!("yield: {e}"), span))?;
        let msg = serde_json::json!({"__yield": json});
        println!("{msg}");
        std::io::stdout().flush()
            .map_err(|e| LxError::runtime(format!("yield: stdout: {e}"), span))?;
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)
            .map_err(|e| LxError::runtime(format!("yield: stdin: {e}"), span))?;
        if line.trim().is_empty() {
            return Err(LxError::runtime("yield: orchestrator closed stdin", span));
        }
        let response: serde_json::Value = serde_json::from_str(line.trim())
            .map_err(|e| LxError::runtime(format!("yield: JSON parse: {e}"), span))?;
        Ok(json_to_lx(response))
    }
}

pub struct StderrLogBackend;

impl LogBackend for StderrLogBackend {
    fn log(&self, level: LogLevel, msg: &str) {
        let tag = match level {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Err => "ERR",
            LogLevel::Debug => "DEBUG",
        };
        eprintln!("[{tag}] {msg}");
    }
}
