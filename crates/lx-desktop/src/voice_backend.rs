use common_voice::AgentBackend;
use std::process::Stdio;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

const SYSTEM_PROMPT: &str = "\
Your final response to the user will be spoken aloud via text-to-speech. \
You may use tools, code, and any formatting you need internally, \
but the text you output to the user must be TTS-friendly: \
no markdown, no bullet points, no numbered lists, no code blocks, no URLs, no special formatting. \
Write abbreviations, acronyms, and numbers as spoken words. Avoid parenthetical asides.";

static SESSION_ID: LazyLock<String> = LazyLock::new(|| uuid::Uuid::new_v4().to_string());
static SESSION_CREATED: AtomicBool = AtomicBool::new(false);

fn build_args<'a>(text: &'a str, format_args: &[&'a str]) -> Vec<&'a str> {
  let mut args = vec!["-p", text];
  args.extend_from_slice(format_args);
  args.extend(["--system-prompt", SYSTEM_PROMPT]);
  if SESSION_CREATED.load(Ordering::Relaxed) {
    args.extend(["--resume", &SESSION_ID]);
  } else {
    args.extend(["--session-id", &SESSION_ID]);
  }
  args
}

pub struct ClaudeCliBackend;

#[async_trait::async_trait]
impl AgentBackend for ClaudeCliBackend {
  async fn query(&self, text: &str) -> anyhow::Result<String> {
    let args = build_args(text, &["--output-format", "text"]);
    let output = tokio::process::Command::new("claude").args(&args).output().await?;
    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      anyhow::bail!("claude cli failed: {stderr}");
    }
    SESSION_CREATED.store(true, Ordering::Relaxed);
    let response = String::from_utf8(output.stdout)?;
    Ok(response.trim().to_owned())
  }
}

pub async fn query_streaming(text: &str, mut on_chunk: impl FnMut(&str)) -> anyhow::Result<String> {
  use tokio::io::AsyncBufReadExt;
  let args = build_args(text, &["--output-format", "stream-json", "--verbose", "--include-partial-messages"]);
  let mut child = tokio::process::Command::new("claude").args(&args).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;
  let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?;
  let mut lines = tokio::io::BufReader::new(stdout).lines();
  let mut result_text = String::new();
  while let Some(line) = lines.next_line().await? {
    let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
    match event["type"].as_str() {
      Some("stream_event") => {
        if event["event"]["type"].as_str() == Some("content_block_delta")
          && event["event"]["delta"]["type"].as_str() == Some("text_delta")
          && let Some(text) = event["event"]["delta"]["text"].as_str()
        {
          on_chunk(text);
        }
      },
      Some("result") => {
        if let Some(text) = event["result"].as_str() {
          result_text = text.trim().to_owned();
        }
      },
      _ => {},
    }
  }
  let status = child.wait().await?;
  if !status.success() && result_text.is_empty() {
    anyhow::bail!("claude cli failed");
  }
  SESSION_CREATED.store(true, Ordering::Relaxed);
  Ok(result_text)
}
