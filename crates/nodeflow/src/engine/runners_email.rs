use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde_json::{Value, json};

use super::expression::{resolve_field, resolve_string};
use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{first_input_item, make_expr_ctx, properties_lookup};
use super::types::{NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_email_runners(registry: &mut NodeRunnerRegistry) {
  registry.register("smtp_send", Arc::new(SmtpRunner));
}

pub struct SmtpRunner;

#[async_trait]
impl NodeRunner for SmtpRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let credential = resolve_field(&properties_lookup(&props, "credential"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving credential: {error}")))?;

    let host = credential.get("host").and_then(Value::as_str).ok_or_else(|| NodeExecutionError::Runtime("smtp_send: credential missing `host`".to_string()))?;
    let username = credential.get("username").and_then(Value::as_str).unwrap_or_default();
    let password = credential.get("password").and_then(Value::as_str).unwrap_or_default();
    let port = credential.get("port").and_then(Value::as_u64).unwrap_or(587) as u16;

    let from_addr =
      resolve_string(&properties_lookup(&props, "from"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let to_addr =
      resolve_string(&properties_lookup(&props, "to"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let subject =
      resolve_string(&properties_lookup(&props, "subject"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;
    let body =
      resolve_string(&properties_lookup(&props, "body"), &expr_ctx, ctx.credentials).map_err(|error| NodeExecutionError::Runtime(error.to_string()))?;

    let message = Message::builder()
      .from(from_addr.parse().map_err(|error| NodeExecutionError::Runtime(format!("parse from: {error}")))?)
      .to(to_addr.parse().map_err(|error| NodeExecutionError::Runtime(format!("parse to: {error}")))?)
      .subject(&subject)
      .body(body.clone())
      .map_err(|error| NodeExecutionError::Runtime(format!("build message: {error}")))?;

    let mailer = if username.is_empty() {
      AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host).port(port).build()
    } else {
      AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
        .map_err(|error| NodeExecutionError::Runtime(format!("smtp relay: {error}")))?
        .credentials(Credentials::new(username.to_string(), password.to_string()))
        .port(port)
        .build()
    };

    let response = mailer.send(message).await.map_err(|error| NodeExecutionError::Runtime(format!("smtp send: {error}")))?;

    let mut outputs = HashMap::new();
    outputs.insert(
      "out".to_string(),
      vec![NodeItem::from_json(json!({
        "code": response.code().to_string(),
        "message": response.message().collect::<Vec<_>>().join("\n"),
        "to": to_addr,
        "subject": subject,
      }))],
    );
    Ok(NodeRunOutcome { outputs, logs: vec![format!("SMTP send to {to_addr}")] })
  }
}
