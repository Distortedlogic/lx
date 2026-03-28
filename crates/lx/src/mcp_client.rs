use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, ChildStdin, ChildStdout};

pub struct McpClient {
  child: Child,
  stdin: Option<BufWriter<ChildStdin>>,
  stdout: BufReader<ChildStdout>,
  next_id: AtomicU64,
  command: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ToolInfo {
  pub name: String,
  pub description: Option<String>,
}

impl McpClient {
  pub async fn spawn(command: &str) -> Result<Self, String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let (program, args) = parts.split_first().ok_or_else(|| format!("command '{command}' is empty"))?;

    let mut child = tokio::process::Command::new(program)
      .args(args)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|_| format!("command '{command}' not found"))?;

    let stdin = child.stdin.take().ok_or_else(|| "failed to capture stdin".to_string())?;
    let stdout = child.stdout.take().ok_or_else(|| "failed to capture stdout".to_string())?;

    let mut client =
      Self { child, stdin: Some(BufWriter::new(stdin)), stdout: BufReader::new(stdout), next_id: AtomicU64::new(1), command: command.to_string() };

    client.initialize().await?;
    Ok(client)
  }

  async fn initialize(&mut self) -> Result<(), String> {
    let resp = self
      .send_request(
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "lx", "version": "0.1.0"}
        }),
      )
      .await?;

    if resp.get("error").is_some() {
      return Err(format!("MCP initialize failed: {}", resp.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("unknown error")));
    }

    let notif = json!({"jsonrpc": "2.0", "method": "notifications/initialized"});
    let mut line = serde_json::to_string(&notif).map_err(|e| e.to_string())?;
    line.push('\n');
    let stdin = self.stdin.as_mut().ok_or_else(|| "stdin already closed".to_string())?;
    stdin.write_all(line.as_bytes()).await.map_err(|e| e.to_string())?;
    stdin.flush().await.map_err(|e| e.to_string())?;

    Ok(())
  }

  pub async fn tools_list(&mut self) -> Result<Vec<ToolInfo>, String> {
    let resp = self.send_request("tools/list", json!({})).await?;

    let tools_val = resp.get("result").and_then(|r| r.get("tools")).ok_or_else(|| "tools/list: no result.tools in response".to_string())?;

    serde_json::from_value(tools_val.clone()).map_err(|e| format!("tools/list parse: {e}"))
  }

  pub async fn tools_call(&mut self, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value, String> {
    let resp = self.send_request("tools/call", json!({"name": tool_name, "arguments": arguments})).await?;

    if let Some(err) = resp.get("error") {
      let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error");
      return Err(msg.to_string());
    }

    if let Some(result) = resp.get("result") {
      if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
        for block in content {
          if block.get("type").and_then(|t| t.as_str()) == Some("text")
            && let Some(text) = block.get("text").and_then(|t| t.as_str())
          {
            if let Ok(parsed) = serde_json::from_str(text) {
              return Ok(parsed);
            }
            return Ok(serde_json::Value::String(text.to_string()));
          }
        }
      }
      return Ok(result.clone());
    }

    Err("tools/call: no result or error in response".to_string())
  }

  pub async fn shutdown(&mut self) {
    let _ = self.send_request("shutdown", json!({})).await;

    drop(self.stdin.take());

    let wait_result = tokio::time::timeout(Duration::from_secs(2), self.child.wait()).await;

    if wait_result.is_err() {
      let _ = self.child.kill().await;
    }
  }

  async fn send_request(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    let id = self.next_id.fetch_add(1, Ordering::Relaxed);
    let request = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });

    let mut line = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    line.push('\n');
    let stdin = self.stdin.as_mut().ok_or_else(|| format!("MCP process '{}' stdin already closed", self.command))?;
    stdin.write_all(line.as_bytes()).await.map_err(|e| format!("MCP write failed for '{}': {e}", self.command))?;
    stdin.flush().await.map_err(|e| format!("MCP flush failed for '{}': {e}", self.command))?;

    loop {
      let mut response_line = String::new();
      let bytes_read = self.stdout.read_line(&mut response_line).await.map_err(|e| format!("MCP read failed for '{}': {e}", self.command))?;

      if bytes_read == 0 {
        return Err(format!("MCP process '{}' closed stdout unexpectedly", self.command));
      }

      let parsed: serde_json::Value = serde_json::from_str(response_line.trim()).map_err(|e| format!("MCP invalid JSON from '{}': {e}", self.command))?;

      if parsed.get("id").is_some() {
        let resp_id = parsed.get("id").and_then(|v| v.as_u64());
        if resp_id == Some(id) {
          return Ok(parsed);
        }
      }
    }
  }

  pub fn is_alive(&mut self) -> bool {
    matches!(self.child.try_wait(), Ok(None))
  }
}
