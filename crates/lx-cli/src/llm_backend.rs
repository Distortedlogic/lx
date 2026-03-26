use std::io::Write;
use std::process::{Command, Stdio};

use lx::error::LxError;
use lx::record;
use lx::runtime::{LlmBackend, LlmOpts};
use lx::value::LxVal;
use miette::SourceSpan;

pub struct ClaudeCodeLlmBackend;

impl LlmBackend for ClaudeCodeLlmBackend {
  fn prompt(&self, text: &str, span: SourceSpan) -> Result<LxVal, LxError> {
    self.prompt_with(&LlmOpts { prompt: text.to_string(), ..Default::default() }, span)
  }

  fn prompt_with(&self, opts: &LlmOpts, span: SourceSpan) -> Result<LxVal, LxError> {
    tokio::task::block_in_place(|| {
      let mut cmd = Command::new("claude");
      cmd.arg("--print").arg("--output-format").arg("stream-json").arg("--verbose");

      if !opts.tools.is_empty() {
        cmd.arg("--allowedTools").arg(opts.tools.join(","));
      }

      if let Some(turns) = opts.max_turns {
        cmd.arg("--max-turns").arg(turns.to_string());
      }

      if let Some(ref schema) = opts.json_schema {
        cmd.arg("--json-schema").arg(schema);
      }

      cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

      let mut child = cmd.spawn().map_err(|e| LxError::runtime(format!("llm: failed to spawn claude: {e}"), span))?;

      if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(opts.prompt.as_bytes()).map_err(|e| LxError::runtime(format!("llm: stdin write: {e}"), span))?;
      }

      let output = child.wait_with_output().map_err(|e| LxError::runtime(format!("llm: wait: {e}"), span))?;

      if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(LxVal::err_str(format!("llm: claude exited {}: {stderr}", output.status)));
      }

      let stdout = String::from_utf8_lossy(&output.stdout);
      parse_ndjson(&stdout, opts.json_schema.is_some(), span)
    })
  }
}

fn recover_json(raw: &str) -> String {
  let mut s = raw.to_string();

  if s.contains("```") {
    let parts: Vec<&str> = s.split("```").collect();
    if parts.len() >= 3 {
      let inner = parts[1];
      let lines: Vec<&str> = inner.lines().collect();
      if lines.len() > 1 {
        s = lines[1..].join("\n");
      } else {
        s = inner.to_string();
      }
    }
  }

  let first_brace = s.find('{');
  let first_bracket = s.find('[');
  let start = match (first_brace, first_bracket) {
    (Some(a), Some(b)) => Some(a.min(b)),
    (Some(a), None) => Some(a),
    (None, Some(b)) => Some(b),
    (None, None) => None,
  };
  let last_brace = s.rfind('}');
  let last_bracket = s.rfind(']');
  let end = match (last_brace, last_bracket) {
    (Some(a), Some(b)) => Some(a.max(b)),
    (Some(a), None) => Some(a),
    (None, Some(b)) => Some(b),
    (None, None) => None,
  };
  if let (Some(start_idx), Some(end_idx)) = (start, end)
    && start_idx <= end_idx
  {
    s = s[start_idx..=end_idx].to_string();
  }

  loop {
    let before = s.len();
    s = s.replace(",}", "}");
    s = s.replace(",]", "]");
    s = s.replace(", }", "}");
    s = s.replace(", ]", "]");
    if s.len() == before {
      break;
    }
  }

  let lines: Vec<String> = s
    .split('\n')
    .map(|line| {
      if let Some(idx) = line.find("//") {
        let before = &line[..idx];
        let quote_count = before.chars().filter(|&c| c == '"').count();
        if quote_count % 2 == 0 { before.to_string() } else { line.to_string() }
      } else {
        line.to_string()
      }
    })
    .collect();
  s = lines.join("\n");

  s
}

fn parse_ndjson(raw: &str, structured: bool, _span: SourceSpan) -> Result<LxVal, LxError> {
  let mut full_text: Vec<String> = Vec::new();
  let mut result_msg: Option<serde_json::Value> = None;

  for line in raw.lines() {
    if line.is_empty() {
      continue;
    }
    let json: serde_json::Value = match serde_json::from_str(line) {
      Ok(v) => v,
      Err(_) => continue,
    };
    match json.get("type").and_then(|t| t.as_str()) {
      Some("assistant") => {
        if let Some(content) = json.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_array()) {
          for item in content {
            if item.get("type").and_then(|t| t.as_str()) == Some("text")
              && let Some(text) = item.get("text").and_then(|t| t.as_str())
            {
              full_text.push(text.to_string());
            }
          }
        }
      },
      Some("result") => {
        result_msg = Some(json);
      },
      _ => {},
    }
  }

  let (response_text, cost, turns, input_tokens, output_tokens, is_error, session_id) = if let Some(ref rm) = result_msg {
    let response_text = rm.get("result").and_then(|r| r.as_str()).map(|s| s.to_string()).unwrap_or_else(|| full_text.join("\n"));
    let cost = rm.get("total_cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let turns = rm.get("num_turns").and_then(|v| v.as_i64()).unwrap_or(0);
    let input_tokens = rm.get("usage").and_then(|u| u.get("input_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
    let output_tokens = rm.get("usage").and_then(|u| u.get("output_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
    let is_error = rm.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
    let session_id = rm.get("session_id").and_then(|v| v.as_str()).map(|s| s.to_string());
    (response_text, cost, turns, input_tokens, output_tokens, is_error, session_id)
  } else {
    (full_text.join("\n"), 0.0, 0_i64, 0_i64, 0_i64, false, None)
  };

  if is_error {
    return Ok(LxVal::err_str(&response_text));
  }

  if structured {
    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&response_text) {
      return Ok(LxVal::ok(LxVal::from(json_val)));
    }
    let recovered = recover_json(&response_text);
    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&recovered) {
      return Ok(LxVal::ok(LxVal::from(json_val)));
    }
  }

  Ok(LxVal::ok(record! {
    "text" => LxVal::str(&response_text),
    "cost_usd" => LxVal::Float(cost),
    "turns" => LxVal::int(turns),
    "input_tokens" => LxVal::int(input_tokens),
    "output_tokens" => LxVal::int(output_tokens),
    "session_id" => session_id.map(LxVal::str).unwrap_or(LxVal::None)
  }))
}
