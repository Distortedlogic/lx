use std::io::Write;

use indexmap::IndexMap;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;

use crate::error::LxError;
use crate::record;
use crate::value::LxVal;
use miette::SourceSpan;

use super::{EmitBackend, HttpBackend, HttpOpts, LogBackend, LogLevel, YieldBackend};

pub struct StdoutEmitBackend;

impl EmitBackend for StdoutEmitBackend {
  fn emit(&self, value: &LxVal, _span: SourceSpan) -> Result<(), LxError> {
    println!("{value}");
    Ok(())
  }
}

pub struct ReqwestHttpBackend;

impl HttpBackend for ReqwestHttpBackend {
  fn request(&self, method: &str, url: &str, opts: &HttpOpts, span: SourceSpan) -> Result<LxVal, LxError> {
    tokio::task::block_in_place(|| {
      tokio::runtime::Handle::current().block_on(async {
        let c = Client::builder().build().map_err(|e| LxError::runtime(format!("http: client: {e}"), span))?;
        let mut builder = match method {
          "GET" => c.get(url),
          "POST" => c.post(url),
          "PUT" => c.put(url),
          "DELETE" => c.delete(url),
          _ => {
            return Err(LxError::runtime(format!("http: unknown method '{method}'"), span));
          },
        };
        if let Some(ref hdrs) = opts.headers {
          for (k, v) in hdrs {
            builder = builder.header(k.as_str(), v.as_str());
          }
        }
        if let Some(ref query) = opts.query {
          let pairs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
          builder = builder.query(&pairs);
        }
        if let Some(ref body) = opts.body {
          builder = builder.header(CONTENT_TYPE, "application/json").json(body);
        }
        match builder.send().await {
          Ok(resp) => response_to_value(resp, span).await,
          Err(e) => Ok(LxVal::err_str(e.to_string())),
        }
      })
    })
  }
}

async fn response_to_value(resp: reqwest::Response, span: SourceSpan) -> Result<LxVal, LxError> {
  let status = resp.status().as_u16();
  let mut headers = IndexMap::new();
  for (name, value) in resp.headers() {
    let v = value.to_str().unwrap_or("").to_string();
    headers.insert(name.to_string(), LxVal::str(v));
  }
  let body_str = resp.text().await.map_err(|e| LxError::runtime(format!("http: body: {e}"), span))?;
  let body = if let Ok(jv) = serde_json::from_str::<serde_json::Value>(&body_str) { LxVal::from(jv) } else { LxVal::str(body_str) };
  Ok(LxVal::ok(record! {
      "status" => LxVal::int(status),
      "body" => body,
      "headers" => LxVal::record(headers),
  }))
}

pub struct StdinStdoutYieldBackend;

impl YieldBackend for StdinStdoutYieldBackend {
  fn yield_value(&self, value: LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
    use std::io::BufRead;
    let json = serde_json::Value::from(&value);
    let msg = serde_json::json!({"__yield": json});
    println!("{msg}");
    std::io::stdout().flush().map_err(|e| LxError::runtime(format!("yield: stdout: {e}"), span))?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).map_err(|e| LxError::runtime(format!("yield: stdin: {e}"), span))?;
    if line.trim().is_empty() {
      return Err(LxError::runtime("yield: orchestrator closed stdin", span));
    }
    let response: serde_json::Value = serde_json::from_str(line.trim()).map_err(|e| LxError::runtime(format!("yield: JSON parse: {e}"), span))?;
    Ok(LxVal::from(response))
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

pub struct NoopEmitBackend;

impl EmitBackend for NoopEmitBackend {
  fn emit(&self, _value: &LxVal, _span: SourceSpan) -> Result<(), LxError> {
    Ok(())
  }
}

pub struct NoopLogBackend;

impl LogBackend for NoopLogBackend {
  fn log(&self, _level: LogLevel, _msg: &str) {}
}
