use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use super::expression::{ExpressionContext, resolve_field, resolve_string};
use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::types::{NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_builtin_runners(registry: &mut NodeRunnerRegistry) {
  registry.register("http_request", Arc::new(HttpRequestRunner::new()));
  registry.register("slack_post", Arc::new(SlackPostRunner::new()));
  registry.register("control_if", Arc::new(IfRunner));
  registry.register("control_merge", Arc::new(MergeRunner));
  registry.register("control_wait", Arc::new(WaitRunner));
  registry.register("control_set", Arc::new(SetRunner));
}

pub struct HttpRequestRunner {
  client: reqwest::Client,
}

impl HttpRequestRunner {
  pub fn new() -> Self {
    Self { client: reqwest::Client::new() }
  }
}

impl Default for HttpRequestRunner {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl NodeRunner for HttpRequestRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let first_input_item = ctx.inputs.values().flat_map(|items| items.iter()).next();
    let expr_ctx = ExpressionContext { exec: ctx.exec, current_item: first_input_item, current_node_id: &ctx.node.id };

    let url = resolve_string(&properties_lookup(&props, "url"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving url: {error}")))?;
    if url.is_empty() {
      return Err(NodeExecutionError::Runtime("http_request: `url` is empty".to_string()));
    }
    let credential_data = resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving credential: {error}")))?;
    let method = resolve_string(&properties_lookup(&props, "method"), &expr_ctx, ctx.credentials).unwrap_or_else(|_| String::new());

    let http_method = parse_http_method(&method).unwrap_or(reqwest::Method::GET);
    let mut request = self.client.request(http_method, &url);

    if let Some(api_key) = credential_data.get("api_key").and_then(Value::as_str) {
      request = request.bearer_auth(api_key);
    }
    if let Some(auth) = credential_data.get("bearer_token").and_then(Value::as_str) {
      request = request.bearer_auth(auth);
    }
    if let Some(basic) = credential_data.get("basic").and_then(Value::as_object) {
      let user = basic.get("username").and_then(Value::as_str).unwrap_or_default();
      let pass = basic.get("password").and_then(Value::as_str).map(ToOwned::to_owned);
      request = request.basic_auth(user, pass);
    }

    let response = request.send().await.map_err(|error| NodeExecutionError::Runtime(format!("http request failed: {error}")))?;
    let status = response.status().as_u16();
    let headers: HashMap<String, String> =
      response.headers().iter().filter_map(|(name, value)| value.to_str().ok().map(|text| (name.as_str().to_string(), text.to_string()))).collect();
    let body_bytes = response.bytes().await.map_err(|error| NodeExecutionError::Runtime(format!("reading http body: {error}")))?;
    let body_text = String::from_utf8_lossy(&body_bytes).to_string();
    let body_value: Value = serde_json::from_str(&body_text).unwrap_or(Value::String(body_text.clone()));

    let mut outputs = HashMap::new();
    outputs.insert("response".to_string(), vec![NodeItem::from_json(json!({ "status": status, "headers": headers, "body": body_value }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("HTTP {} {url} -> {status}", method_label(&method))] })
  }
}

pub struct SlackPostRunner {
  client: reqwest::Client,
}

impl SlackPostRunner {
  pub fn new() -> Self {
    Self { client: reqwest::Client::new() }
  }
}

impl Default for SlackPostRunner {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl NodeRunner for SlackPostRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let first_input_item = ctx.inputs.values().flat_map(|items| items.iter()).next();
    let expr_ctx = ExpressionContext { exec: ctx.exec, current_item: first_input_item, current_node_id: &ctx.node.id };

    let channel = resolve_string(&properties_lookup(&props, "channel"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving channel: {error}")))?;
    let credential_data = resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving credential: {error}")))?;
    let text = resolve_string(&properties_lookup(&props, "text"), &expr_ctx, ctx.credentials)
      .ok()
      .filter(|text| !text.is_empty())
      .unwrap_or_else(|| first_input_item.map(|item| item.json.to_string()).unwrap_or_else(|| "(empty)".to_string()));

    let webhook_url = credential_data.get("webhook_url").and_then(Value::as_str);
    let bot_token = credential_data.get("bot_token").and_then(Value::as_str);

    if let Some(url) = webhook_url {
      let payload = json!({ "text": text, "channel": channel });
      let response =
        self.client.post(url).json(&payload).send().await.map_err(|error| NodeExecutionError::Runtime(format!("slack webhook failed: {error}")))?;
      let status = response.status().as_u16();
      let body = response.text().await.unwrap_or_default();
      let mut outputs = HashMap::new();
      outputs.insert("out".to_string(), vec![NodeItem::from_json(json!({ "mode": "webhook", "status": status, "response": body, "channel": channel }))]);
      return Ok(NodeRunOutcome { outputs, logs: vec![format!("Slack webhook -> {status}")] });
    }

    if let Some(token) = bot_token {
      let payload = json!({ "channel": channel, "text": text });
      let response = self
        .client
        .post("https://slack.com/api/chat.postMessage")
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .map_err(|error| NodeExecutionError::Runtime(format!("slack api failed: {error}")))?;
      let status = response.status().as_u16();
      let body: Value = response.json().await.unwrap_or(Value::Null);
      let mut outputs = HashMap::new();
      outputs.insert("out".to_string(), vec![NodeItem::from_json(json!({ "mode": "api", "status": status, "response": body, "channel": channel }))]);
      return Ok(NodeRunOutcome { outputs, logs: vec![format!("Slack API -> {status}")] });
    }

    Err(NodeExecutionError::Runtime("slack_post: credential must include `webhook_url` or `bot_token`".to_string()))
  }
}

pub struct IfRunner;

#[async_trait]
impl NodeRunner for IfRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let items: Vec<NodeItem> = ctx.inputs.values().flat_map(|values| values.iter().cloned()).collect();
    let props = ctx.node.properties.clone();

    let mut truthy_items = Vec::new();
    let mut falsy_items = Vec::new();

    for item in items {
      let expr_ctx = ExpressionContext { exec: ctx.exec, current_item: Some(&item), current_node_id: &ctx.node.id };
      let condition = resolve_string(&properties_lookup(&props, "condition"), &expr_ctx, ctx.credentials)
        .map_err(|error| NodeExecutionError::Runtime(format!("resolving condition: {error}")))?;
      if is_truthy(&condition) {
        truthy_items.push(item);
      } else {
        falsy_items.push(item);
      }
    }

    let mut outputs = HashMap::new();
    let truthy_len = truthy_items.len();
    let falsy_len = falsy_items.len();
    outputs.insert("true".to_string(), truthy_items);
    outputs.insert("false".to_string(), falsy_items);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("IF routed {truthy_len} true / {falsy_len} false")] })
  }
}

pub struct MergeRunner;

#[async_trait]
impl NodeRunner for MergeRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let mut merged: Vec<NodeItem> = Vec::new();
    for port_id in ["input_a", "input_b"] {
      if let Some(items) = ctx.inputs.get(port_id) {
        merged.extend(items.iter().cloned());
      }
    }
    let mut outputs = HashMap::new();
    let count = merged.len();
    outputs.insert("out".to_string(), merged);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Merged {count} items")] })
  }
}

pub struct WaitRunner;

#[async_trait]
impl NodeRunner for WaitRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let delay_raw = properties_lookup(&ctx.node.properties, "delay_seconds");
    let delay_value = match delay_raw {
      Value::Number(number) => number.as_f64().unwrap_or(1.0),
      Value::String(text) => text.parse::<f64>().unwrap_or(1.0),
      _ => {
        let first_input_item = ctx.inputs.values().flat_map(|items| items.iter()).next();
        let expr_ctx = ExpressionContext { exec: ctx.exec, current_item: first_input_item, current_node_id: &ctx.node.id };
        let rendered = resolve_string(&delay_raw, &expr_ctx, ctx.credentials).unwrap_or_default();
        rendered.parse::<f64>().unwrap_or(1.0)
      },
    };
    let millis = (delay_value.max(0.0) * 1000.0) as u64;
    tokio::time::sleep(tokio::time::Duration::from_millis(millis)).await;

    let items: Vec<NodeItem> = ctx.inputs.values().flat_map(|values| values.iter().cloned()).collect();
    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), items);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Waited {delay_value} seconds")] })
  }
}

pub struct SetRunner;

#[async_trait]
impl NodeRunner for SetRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let mode = properties_lookup(&props, "mode").as_str().map(ToOwned::to_owned).unwrap_or_else(|| "merge".to_string());
    let items: Vec<NodeItem> = ctx.inputs.values().flat_map(|values| values.iter().cloned()).collect();
    let items = if items.is_empty() { vec![NodeItem::from_json(Value::Object(Default::default()))] } else { items };

    let mut out_items = Vec::with_capacity(items.len());
    for item in items {
      let expr_ctx = ExpressionContext { exec: ctx.exec, current_item: Some(&item), current_node_id: &ctx.node.id };
      let assignments_text = resolve_string(&properties_lookup(&props, "assignments"), &expr_ctx, ctx.credentials)
        .map_err(|error| NodeExecutionError::Runtime(format!("resolving assignments: {error}")))?;
      let patch: Value = serde_json::from_str(&assignments_text).map_err(|error| NodeExecutionError::Runtime(format!("parsing assignments json: {error}")))?;
      let new_json = match mode.as_str() {
        "replace" => patch,
        _ => merge_json(item.json.clone(), patch),
      };
      out_items.push(NodeItem::from_json(new_json));
    }

    let mut outputs = HashMap::new();
    let count = out_items.len();
    outputs.insert("out".to_string(), out_items);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Set applied to {count} items ({mode})")] })
  }
}

fn is_truthy(value: &str) -> bool {
  let trimmed = value.trim().to_lowercase();
  !matches!(trimmed.as_str(), "" | "false" | "0" | "no" | "null" | "undefined")
}

fn merge_json(base: Value, patch: Value) -> Value {
  match (base, patch) {
    (Value::Object(mut left), Value::Object(right)) => {
      for (key, value) in right {
        let merged = match left.remove(&key) {
          Some(existing) => merge_json(existing, value),
          None => value,
        };
        left.insert(key, merged);
      }
      Value::Object(left)
    },
    (_, patch) => patch,
  }
}

fn properties_lookup(props: &Value, key: &str) -> Value {
  props.as_object().and_then(|map| map.get(key)).cloned().unwrap_or(Value::Null)
}

fn parse_http_method(method: &str) -> Option<reqwest::Method> {
  let upper = method.trim().to_uppercase();
  match upper.as_str() {
    "" | "GET" => Some(reqwest::Method::GET),
    "POST" => Some(reqwest::Method::POST),
    "PUT" => Some(reqwest::Method::PUT),
    "PATCH" => Some(reqwest::Method::PATCH),
    "DELETE" => Some(reqwest::Method::DELETE),
    "HEAD" => Some(reqwest::Method::HEAD),
    "OPTIONS" => Some(reqwest::Method::OPTIONS),
    _ => None,
  }
}

fn method_label(method: &str) -> &str {
  if method.trim().is_empty() { "GET" } else { method.trim() }
}
