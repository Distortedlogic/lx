use common_voice::AgentBackend;
use std::process::Stdio;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::AsyncReadExt;

const SYSTEM_PROMPT: &str = "\
Your final response to the user will be spoken aloud via text-to-speech. \
You may use tools, code, and any formatting you need internally, \
but the text you output to the user must be TTS-friendly: \
no markdown, no bullet points, no numbered lists, no code blocks, no URLs, no special formatting. \
Write abbreviations, acronyms, and numbers as spoken words. Avoid parenthetical asides.";

static SESSION_ID: LazyLock<String> = LazyLock::new(|| uuid::Uuid::new_v4().to_string());
static SESSION_CREATED: AtomicBool = AtomicBool::new(false);

fn build_args(text: &str) -> Vec<&str> {
  let mut args = vec!["-p", text, "--output-format", "text", "--system-prompt", SYSTEM_PROMPT];
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
    let args = build_args(text);
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
  let args = build_args(text);
  let mut child = tokio::process::Command::new("claude").args(&args).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;
  let mut stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?;
  let mut full = Vec::new();
  let mut buf = [0u8; 256];
  loop {
    let n = stdout.read(&mut buf).await?;
    if n == 0 {
      break;
    }
    full.extend_from_slice(&buf[..n]);
    let chunk = String::from_utf8_lossy(&buf[..n]);
    on_chunk(&chunk);
  }
  let status = child.wait().await?;
  if !status.success() {
    anyhow::bail!("claude cli failed");
  }
  SESSION_CREATED.store(true, Ordering::Relaxed);
  Ok(String::from_utf8(full)?.trim().to_owned())
}
