use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;

use super::expression::resolve_string;
use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{first_input_item, make_expr_ctx, properties_lookup};
use super::types::{NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_filesystem_runners(registry: &mut NodeRunnerRegistry) {
  registry.register("file_read", Arc::new(FileReadRunner));
  registry.register("file_write", Arc::new(FileWriteRunner));
}

pub struct FileReadRunner;

#[async_trait]
impl NodeRunner for FileReadRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let path = resolve_string(&properties_lookup(&props, "path"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving path: {error}")))?;
    let format = properties_lookup(&props, "format").as_str().unwrap_or("text").to_string();

    let bytes = tokio::fs::read(&path).await.map_err(|error| NodeExecutionError::Runtime(format!("read `{path}`: {error}")))?;
    let content = match format.as_str() {
      "bytes_base64" => {
        use base64::Engine as _;
        Value::String(base64::engine::general_purpose::STANDARD.encode(&bytes))
      },
      "json" => serde_json::from_slice(&bytes).map_err(|error| NodeExecutionError::Runtime(format!("parse json: {error}")))?,
      _ => Value::String(String::from_utf8_lossy(&bytes).to_string()),
    };

    let mut outputs = HashMap::new();
    outputs.insert("content".to_string(), vec![NodeItem::from_json(json!({ "path": path, "content": content, "bytes": bytes.len() }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Read {} bytes from {path}", bytes.len())] })
  }
}

pub struct FileWriteRunner;

#[async_trait]
impl NodeRunner for FileWriteRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let path = resolve_string(&properties_lookup(&props, "path"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving path: {error}")))?;
    let content = resolve_string(&properties_lookup(&props, "content"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving content: {error}")))?;
    let append = properties_lookup(&props, "append").as_bool().unwrap_or(false);

    if let Some(parent) = PathBuf::from(&path).parent() {
      tokio::fs::create_dir_all(parent).await.map_err(|error| NodeExecutionError::Runtime(format!("mkdir `{}`: {error}", parent.display())))?;
    }

    let bytes_written = if append {
      let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|error| NodeExecutionError::Runtime(format!("open `{path}`: {error}")))?;
      file.write_all(content.as_bytes()).await.map_err(|error| NodeExecutionError::Runtime(format!("append to `{path}`: {error}")))?;
      file.flush().await.map_err(|error| NodeExecutionError::Runtime(format!("flush `{path}`: {error}")))?;
      content.len()
    } else {
      tokio::fs::write(&path, &content).await.map_err(|error| NodeExecutionError::Runtime(format!("write `{path}`: {error}")))?;
      content.len()
    };

    let mut outputs = HashMap::new();
    outputs.insert("result".to_string(), vec![NodeItem::from_json(json!({ "path": path, "bytes_written": bytes_written, "appended": append }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Wrote {bytes_written} bytes to {path}")] })
  }
}
