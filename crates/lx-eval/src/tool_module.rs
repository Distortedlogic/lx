use std::sync::atomic::{AtomicU64, Ordering};

use indexmap::IndexMap;

use lx_span::sym::intern;
use lx_value::{EventStream, LxError, LxVal};

use crate::mcp_client::McpClient;

pub struct ToolModule {
  pub command: String,
  pub alias: String,
  client: tokio::sync::Mutex<McpClient>,
  call_counter: AtomicU64,
}

impl std::fmt::Debug for ToolModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ToolModule").field("command", &self.command).field("alias", &self.alias).finish()
  }
}

impl ToolModule {
  pub async fn new(command: &str, alias: &str) -> Result<Self, String> {
    let client = McpClient::spawn(command).await?;
    Ok(Self { command: command.to_string(), alias: alias.to_string(), client: tokio::sync::Mutex::new(client), call_counter: AtomicU64::new(1) })
  }

  pub async fn call_tool(&self, method: &str, args: LxVal, event_stream: &EventStream, agent_name: &str) -> Result<LxVal, LxError> {
    let span = miette::SourceSpan::new(0.into(), 0);

    {
      let mut client = self.client.lock().await;
      if !client.is_alive() {
        return Err(LxError::runtime(format!("tool '{}' process exited", self.alias), span));
      }
    }

    let call_id = self.call_counter.fetch_add(1, Ordering::Relaxed);

    let arguments: serde_json::Value = match &args {
      LxVal::Record(_) => serde_json::to_value(&args).unwrap_or(serde_json::json!({})),
      LxVal::Str(s) => serde_json::json!({"input": s.as_ref()}),
      LxVal::Unit => serde_json::json!({}),
      other => serde_json::to_value(other).unwrap_or(serde_json::json!({})),
    };

    let mut call_fields = IndexMap::new();
    call_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
    call_fields.insert(intern("tool"), LxVal::str(&self.alias));
    call_fields.insert(intern("method"), LxVal::str(method));
    call_fields.insert(intern("args"), args);
    event_stream.xadd("tool/call", agent_name, None, call_fields);

    let mut client = self.client.lock().await;
    match client.tools_call(method, arguments).await {
      Ok(result) => {
        let result_lxval = LxVal::from(result);

        let mut result_fields = IndexMap::new();
        result_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
        result_fields.insert(intern("tool"), LxVal::str(&self.alias));
        result_fields.insert(intern("method"), LxVal::str(method));
        result_fields.insert(intern("result"), result_lxval.clone());
        event_stream.xadd("tool/result", agent_name, None, result_fields);

        Ok(result_lxval)
      },
      Err(error_msg) => {
        let mut error_fields = IndexMap::new();
        error_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
        error_fields.insert(intern("tool"), LxVal::str(&self.alias));
        error_fields.insert(intern("method"), LxVal::str(method));
        error_fields.insert(intern("error"), LxVal::str(&error_msg));
        event_stream.xadd("tool/error", agent_name, None, error_fields);

        Err(LxError::runtime(format!("tool '{}' method '{}': {}", self.alias, method, error_msg), span))
      },
    }
  }

  pub async fn shutdown(&self) {
    let mut client = self.client.lock().await;
    client.shutdown().await;
  }
}

