use std::io::Write;
use std::process::{Command, Stdio};

use lx::error::LxError;
use lx::record;
use lx::runtime::{AiBackend, AiOpts};
use lx::value::LxVal;
use miette::SourceSpan;

pub struct ClaudeCodeAiBackend;

impl AiBackend for ClaudeCodeAiBackend {
  fn prompt(&self, text: &str, span: SourceSpan) -> Result<LxVal, LxError> {
    self.prompt_with(&AiOpts { prompt: text.to_string(), ..Default::default() }, span)
  }

  fn prompt_with(&self, opts: &AiOpts, span: SourceSpan) -> Result<LxVal, LxError> {
    tokio::task::block_in_place(|| {
      let mut cmd = Command::new("claude");
      cmd.arg("--print").arg("--output-format").arg("json");

      if !opts.tools.is_empty() {
        cmd.arg("--allowedTools").arg(opts.tools.join(","));
      }

      if let Some(turns) = opts.max_turns {
        cmd.arg("--max-turns").arg(turns.to_string());
      }

      cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

      let mut child = cmd.spawn().map_err(|e| LxError::runtime(format!("ai: failed to spawn claude: {e}"), span))?;

      if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(opts.prompt.as_bytes()).map_err(|e| LxError::runtime(format!("ai: stdin write: {e}"), span))?;
      }

      let output = child.wait_with_output().map_err(|e| LxError::runtime(format!("ai: wait: {e}"), span))?;

      if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(LxVal::err_str(format!("ai: claude exited {}: {stderr}", output.status)));
      }

      let stdout = String::from_utf8_lossy(&output.stdout);
      parse_response(&stdout, span)
    })
  }
}

fn parse_response(raw: &str, _span: SourceSpan) -> Result<LxVal, LxError> {
  let text = match serde_json::from_str::<serde_json::Value>(raw) {
    Ok(json) => {
      if let Some(result) = json.get("result").and_then(|r| r.as_str()) {
        result.to_string()
      } else if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
        text.to_string()
      } else {
        raw.to_string()
      }
    },
    Err(_) => raw.to_string(),
  };
  Ok(LxVal::ok(record! {
    "text" => LxVal::str(&text)
  }))
}
