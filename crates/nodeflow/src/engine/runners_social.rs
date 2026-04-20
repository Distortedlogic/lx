use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use super::expression::{resolve_field, resolve_string};
use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{first_input_item, make_expr_ctx, properties_lookup};
use super::types::{NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_social_runners(registry: &mut NodeRunnerRegistry) {
  registry.register("discord_webhook", Arc::new(DiscordWebhookRunner::new()));
  registry.register("telegram_send", Arc::new(TelegramRunner::new()));
  registry.register("github_issue_create", Arc::new(GitHubIssueRunner::new()));
  registry.register("notion_page_append", Arc::new(NotionRunner::new()));
  registry.register("airtable_record_create", Arc::new(AirtableRunner::new()));
  registry.register("google_sheets_append", Arc::new(GoogleSheetsRunner::new()));
}

macro_rules! reqwest_runner {
  ($name:ident) => {
    pub struct $name {
      client: reqwest::Client,
    }
    impl $name {
      pub fn new() -> Self {
        Self { client: reqwest::Client::new() }
      }
    }
    impl Default for $name {
      fn default() -> Self {
        Self::new()
      }
    }
  };
}

reqwest_runner!(DiscordWebhookRunner);
reqwest_runner!(TelegramRunner);
reqwest_runner!(GitHubIssueRunner);
reqwest_runner!(NotionRunner);
reqwest_runner!(AirtableRunner);
reqwest_runner!(GoogleSheetsRunner);

#[async_trait]
impl NodeRunner for DiscordWebhookRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let credential =
      resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let webhook_url = credential
      .get("webhook_url")
      .and_then(Value::as_str)
      .ok_or_else(|| NodeExecutionError::Runtime("discord_webhook: credential missing `webhook_url`".to_string()))?;
    let content =
      resolve_string(&properties_lookup(&props, "content"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;

    let response = self
      .client
      .post(webhook_url)
      .json(&json!({ "content": content }))
      .send()
      .await
      .map_err(|error| NodeExecutionError::Runtime(format!("discord: {error}")))?;
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();

    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), vec![NodeItem::from_json(json!({ "status": status, "response": body }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Discord webhook -> {status}")] })
  }
}

#[async_trait]
impl NodeRunner for TelegramRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let credential =
      resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let bot_token = credential
      .get("bot_token")
      .and_then(Value::as_str)
      .ok_or_else(|| NodeExecutionError::Runtime("telegram_send: credential missing `bot_token`".to_string()))?;
    let chat_id =
      resolve_string(&properties_lookup(&props, "chat_id"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let text =
      resolve_string(&properties_lookup(&props, "text"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;

    let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
    let response = self
      .client
      .post(&url)
      .json(&json!({ "chat_id": chat_id, "text": text }))
      .send()
      .await
      .map_err(|error| NodeExecutionError::Runtime(format!("telegram: {error}")))?;
    let status = response.status().as_u16();
    let body: Value = response.json().await.unwrap_or(Value::Null);
    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), vec![NodeItem::from_json(json!({ "status": status, "response": body }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Telegram send -> {status}")] })
  }
}

#[async_trait]
impl NodeRunner for GitHubIssueRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let credential =
      resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let token = credential
      .get("token")
      .and_then(Value::as_str)
      .ok_or_else(|| NodeExecutionError::Runtime("github_issue_create: credential missing `token`".to_string()))?;
    let owner =
      resolve_string(&properties_lookup(&props, "owner"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let repo =
      resolve_string(&properties_lookup(&props, "repo"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let title =
      resolve_string(&properties_lookup(&props, "title"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let body = resolve_string(&properties_lookup(&props, "body"), &expr_ctx, ctx.credentials).unwrap_or_default();

    let url = format!("https://api.github.com/repos/{owner}/{repo}/issues");
    let response = self
      .client
      .post(&url)
      .header("User-Agent", "nodeflow")
      .header("Accept", "application/vnd.github+json")
      .bearer_auth(token)
      .json(&json!({ "title": title, "body": body }))
      .send()
      .await
      .map_err(|error| NodeExecutionError::Runtime(format!("github: {error}")))?;
    let status = response.status().as_u16();
    let payload: Value = response.json().await.unwrap_or(Value::Null);
    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), vec![NodeItem::from_json(json!({ "status": status, "issue": payload }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("GitHub create issue `{title}` -> {status}")] })
  }
}

#[async_trait]
impl NodeRunner for NotionRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let credential =
      resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let token = credential
      .get("token")
      .and_then(Value::as_str)
      .ok_or_else(|| NodeExecutionError::Runtime("notion_page_append: credential missing `token`".to_string()))?;
    let page_id =
      resolve_string(&properties_lookup(&props, "page_id"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let text =
      resolve_string(&properties_lookup(&props, "text"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;

    let url = format!("https://api.notion.com/v1/blocks/{page_id}/children");
    let body = json!({
      "children": [
        { "object": "block", "type": "paragraph", "paragraph": { "rich_text": [ { "type": "text", "text": { "content": text } } ] } }
      ]
    });
    let response = self
      .client
      .patch(&url)
      .bearer_auth(token)
      .header("Notion-Version", "2022-06-28")
      .json(&body)
      .send()
      .await
      .map_err(|error| NodeExecutionError::Runtime(format!("notion: {error}")))?;
    let status = response.status().as_u16();
    let payload: Value = response.json().await.unwrap_or(Value::Null);
    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), vec![NodeItem::from_json(json!({ "status": status, "response": payload }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Notion append -> {status}")] })
  }
}

#[async_trait]
impl NodeRunner for AirtableRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let credential =
      resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let token =
      credential.get("api_key").and_then(Value::as_str).ok_or_else(|| NodeExecutionError::Runtime("airtable: credential missing `api_key`".to_string()))?;
    let base_id =
      resolve_string(&properties_lookup(&props, "base_id"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let table =
      resolve_string(&properties_lookup(&props, "table"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let fields_raw = resolve_string(&properties_lookup(&props, "fields_json"), &expr_ctx, ctx.credentials).unwrap_or_else(|_| "{}".to_string());
    let fields: Value = serde_json::from_str(&fields_raw).unwrap_or(Value::Object(Default::default()));

    let url = format!("https://api.airtable.com/v0/{base_id}/{table}");
    let response = self
      .client
      .post(&url)
      .bearer_auth(token)
      .json(&json!({ "fields": fields }))
      .send()
      .await
      .map_err(|error| NodeExecutionError::Runtime(format!("airtable: {error}")))?;
    let status = response.status().as_u16();
    let payload: Value = response.json().await.unwrap_or(Value::Null);
    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), vec![NodeItem::from_json(json!({ "status": status, "record": payload }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Airtable create -> {status}")] })
  }
}

#[async_trait]
impl NodeRunner for GoogleSheetsRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let credential =
      resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let access_token = credential
      .get("access_token")
      .and_then(Value::as_str)
      .ok_or_else(|| NodeExecutionError::Runtime("google_sheets_append: credential missing `access_token`".to_string()))?;
    let spreadsheet_id = resolve_string(&properties_lookup(&props, "spreadsheet_id"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let range =
      resolve_string(&properties_lookup(&props, "range"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let values_raw = resolve_string(&properties_lookup(&props, "values_json"), &expr_ctx, ctx.credentials).unwrap_or_else(|_| "[]".to_string());
    let values: Value = serde_json::from_str(&values_raw).unwrap_or(Value::Array(Vec::new()));

    let url = format!("https://sheets.googleapis.com/v4/spreadsheets/{spreadsheet_id}/values/{range}:append?valueInputOption=RAW");
    let response = self
      .client
      .post(&url)
      .bearer_auth(access_token)
      .json(&json!({ "values": values }))
      .send()
      .await
      .map_err(|error| NodeExecutionError::Runtime(format!("google sheets: {error}")))?;
    let status = response.status().as_u16();
    let payload: Value = response.json().await.unwrap_or(Value::Null);
    let mut outputs = HashMap::new();
    outputs.insert("out".to_string(), vec![NodeItem::from_json(json!({ "status": status, "response": payload }))]);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Google Sheets append -> {status}")] })
  }
}
