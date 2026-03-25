use common_voice::AgentBackend;
use std::sync::LazyLock;

const SYSTEM_PROMPT: &str = "\
Your final response to the user will be spoken aloud via text-to-speech. \
You may use tools, code, and any formatting you need internally, \
but the text you output to the user must be TTS-friendly: \
no markdown, no bullet points, no numbered lists, no code blocks, no URLs, no special formatting. \
Write abbreviations, acronyms, and numbers as spoken words. Avoid parenthetical asides.";

pub static SESSION_ID: LazyLock<String> = LazyLock::new(|| uuid::Uuid::new_v4().to_string());

pub struct ClaudeCliBackend;

#[async_trait::async_trait]
impl AgentBackend for ClaudeCliBackend {
  async fn query(&self, text: &str) -> anyhow::Result<String> {
    let output = tokio::process::Command::new("claude")
      .args(["-p", text, "--output-format", "text", "--system-prompt", SYSTEM_PROMPT, "--session-id", &SESSION_ID])
      .output()
      .await?;
    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      anyhow::bail!("claude cli failed: {stderr}");
    }
    let response = String::from_utf8(output.stdout)?;
    Ok(response.trim().to_owned())
  }
}
