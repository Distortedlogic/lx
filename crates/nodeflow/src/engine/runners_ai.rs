use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use super::expression::{resolve_field, resolve_string};
use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{first_input_item, make_expr_ctx, properties_lookup};
use super::types::{NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_ai_runners(registry: &mut NodeRunnerRegistry) {
  registry.register("anthropic_messages", Arc::new(AnthropicRunner::new()));
  registry.register("openai_chat", Arc::new(OpenAiRunner::new()));
}

pub struct AnthropicRunner {
  client: reqwest::Client,
}

impl AnthropicRunner {
  pub fn new() -> Self {
    Self { client: reqwest::Client::new() }
  }
}

impl Default for AnthropicRunner {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl NodeRunner for AnthropicRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);

    let model = resolve_string(&properties_lookup(&props, "model"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving model: {error}")))?;
    let prompt = resolve_string(&properties_lookup(&props, "prompt"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving prompt: {error}")))?;
    let max_tokens = properties_lookup(&props, "max_tokens").as_u64().unwrap_or(1024);
    let credential_data = resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving credential: {error}")))?;
    let api_key = credential_data
      .get("api_key")
      .and_then(Value::as_str)
      .ok_or_else(|| NodeExecutionError::Runtime("anthropic_messages: credential missing `api_key`".to_string()))?;

    let body = json!({
      "model": model,
      "max_tokens": max_tokens,
      "messages": [ { "role": "user", "content": prompt } ]
    });
    let response = self
      .client
      .post("https://api.anthropic.com/v1/messages")
      .header("x-api-key", api_key)
      .header("anthropic-version", "2023-06-01")
      .json(&body)
      .send()
      .await
      .map_err(|error| NodeExecutionError::Runtime(format!("anthropic request failed: {error}")))?;
    let status = response.status().as_u16();
    let response_body: Value = response.json().await.unwrap_or(Value::Null);
    let text = response_body
      .get("content")
      .and_then(Value::as_array)
      .and_then(|blocks| blocks.first())
      .and_then(|block| block.get("text"))
      .and_then(Value::as_str)
      .map(ToOwned::to_owned);

    let mut outputs = HashMap::new();
    outputs.insert("response".to_string(), vec![NodeItem::from_json(json!({ "status": status, "text": text, "raw": response_body }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Anthropic `{model}` -> {status}")] })
  }
}

pub struct OpenAiRunner {
  client: reqwest::Client,
}

impl OpenAiRunner {
  pub fn new() -> Self {
    Self { client: reqwest::Client::new() }
  }
}

impl Default for OpenAiRunner {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl NodeRunner for OpenAiRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);

    let model = resolve_string(&properties_lookup(&props, "model"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving model: {error}")))?;
    let prompt = resolve_string(&properties_lookup(&props, "prompt"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving prompt: {error}")))?;
    let credential_data = resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving credential: {error}")))?;
    let api_key = credential_data
      .get("api_key")
      .and_then(Value::as_str)
      .ok_or_else(|| NodeExecutionError::Runtime("openai_chat: credential missing `api_key`".to_string()))?;

    let body = json!({
      "model": model,
      "messages": [ { "role": "user", "content": prompt } ],
    });
    let response = self
      .client
      .post("https://api.openai.com/v1/chat/completions")
      .bearer_auth(api_key)
      .json(&body)
      .send()
      .await
      .map_err(|error| NodeExecutionError::Runtime(format!("openai request failed: {error}")))?;
    let status = response.status().as_u16();
    let response_body: Value = response.json().await.unwrap_or(Value::Null);
    let text = response_body
      .get("choices")
      .and_then(Value::as_array)
      .and_then(|choices| choices.first())
      .and_then(|choice| choice.get("message"))
      .and_then(|message| message.get("content"))
      .and_then(Value::as_str)
      .map(ToOwned::to_owned);

    let mut outputs = HashMap::new();
    outputs.insert("response".to_string(), vec![NodeItem::from_json(json!({ "status": status, "text": text, "raw": response_body }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("OpenAI `{model}` -> {status}")] })
  }
}
