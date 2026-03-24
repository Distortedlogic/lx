use common_voice::AgentBackend;

pub struct ClaudeCliBackend;

#[async_trait::async_trait]
impl AgentBackend for ClaudeCliBackend {
  async fn query(&self, text: &str) -> anyhow::Result<String> {
    let output = tokio::process::Command::new("claude").args(["-p", text, "--output-format", "text"]).output().await?;
    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      anyhow::bail!("claude cli failed: {stderr}");
    }
    let response = String::from_utf8(output.stdout)?;
    Ok(response.trim().to_owned())
  }
}
