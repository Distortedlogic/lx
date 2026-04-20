use serde_json::Value;

use crate::credentials::CredentialStore;

use super::context::ExecutionContext;
use super::types::{NodeItem, now_ts};

#[derive(Clone, Debug, thiserror::Error)]
pub enum ExpressionError {
  #[error("unclosed expression in `{0}`")]
  UnclosedExpression(String),
  #[error("invalid reference `{0}`")]
  InvalidReference(String),
  #[error("undefined: {0}")]
  Undefined(String),
  #[error("parse: {0}")]
  Parse(String),
}

pub struct ExpressionContext<'a> {
  pub exec: &'a ExecutionContext,
  pub current_item: Option<&'a NodeItem>,
  pub current_node_id: &'a str,
}

pub fn evaluate_template(template: &str, ctx: &ExpressionContext<'_>) -> Result<String, ExpressionError> {
  let mut out = String::new();
  let mut rest = template;
  while let Some(start) = rest.find("{{") {
    out.push_str(&rest[..start]);
    let after_open = &rest[start + 2..];
    let end = after_open.find("}}").ok_or_else(|| ExpressionError::UnclosedExpression(template.to_string()))?;
    let expr = after_open[..end].trim();
    let value = evaluate_expression(expr, ctx)?;
    out.push_str(&value_to_string(&value));
    rest = &after_open[end + 2..];
  }
  out.push_str(rest);
  Ok(out)
}

pub fn evaluate_expression(expr: &str, ctx: &ExpressionContext<'_>) -> Result<Value, ExpressionError> {
  let expr = expr.trim();
  if expr == "$now" {
    return Ok(Value::String(now_ts()));
  }
  if let Some(rest) = expr.strip_prefix("$env.") {
    return Ok(std::env::var(rest).map(Value::String).unwrap_or(Value::Null));
  }
  if let Some(rest) = expr.strip_prefix("$json") {
    let item = ctx.current_item.ok_or_else(|| ExpressionError::Undefined("$json requires a current item".to_string()))?;
    return walk_path(&item.json, rest);
  }
  if let Some(after) = expr.strip_prefix("$node[") {
    return evaluate_node_reference(after, ctx);
  }
  if expr.starts_with('"') || expr.starts_with('\'') {
    return parse_string_literal(expr).map(Value::String).ok_or_else(|| ExpressionError::Parse(format!("bad string literal: {expr}")));
  }
  if let Ok(int) = expr.parse::<i64>() {
    return Ok(Value::Number(int.into()));
  }
  if let Ok(float) = expr.parse::<f64>() {
    return serde_json::Number::from_f64(float).map(Value::Number).ok_or_else(|| ExpressionError::Parse(format!("bad number: {expr}")));
  }
  Err(ExpressionError::InvalidReference(expr.to_string()))
}

fn evaluate_node_reference(after: &str, ctx: &ExpressionContext<'_>) -> Result<Value, ExpressionError> {
  let close = after.find(']').ok_or_else(|| ExpressionError::Parse("missing `]` in $node[..]".to_string()))?;
  let key_raw = &after[..close];
  let node_id = parse_string_literal(key_raw).ok_or_else(|| ExpressionError::Parse(format!("$node key must be a string literal: {key_raw}")))?;
  let rest = &after[close + 1..];

  if let Some(after_json) = rest.strip_prefix(".json") {
    let outputs = ctx.exec.node_outputs(&node_id).ok_or_else(|| ExpressionError::Undefined(format!("node `{node_id}` has no outputs")))?;
    let first_port_items = outputs.values().next().ok_or_else(|| ExpressionError::Undefined(format!("node `{node_id}` emitted no ports")))?;
    let first_item = first_port_items.first().ok_or_else(|| ExpressionError::Undefined(format!("node `{node_id}` emitted no items")))?;
    return walk_path(&first_item.json, after_json);
  }

  Err(ExpressionError::Parse(format!("unsupported $node suffix: {rest}")))
}

fn walk_path(root: &Value, path: &str) -> Result<Value, ExpressionError> {
  let null = Value::Null;
  let mut current: &Value = root;
  let mut rest = path;
  loop {
    rest = rest.trim_start();
    if rest.is_empty() {
      return Ok(current.clone());
    }
    if let Some(tail) = rest.strip_prefix('.') {
      let (ident, remainder) = split_ident(tail);
      if ident.is_empty() {
        return Err(ExpressionError::Parse(format!("expected identifier after `.`: {rest}")));
      }
      current = current.get(ident).unwrap_or(&null);
      rest = remainder;
      continue;
    }
    if let Some(after) = rest.strip_prefix('[') {
      let close = after.find(']').ok_or_else(|| ExpressionError::Parse("missing `]`".to_string()))?;
      let key_raw = after[..close].trim();
      if let Some(key) = parse_string_literal(key_raw) {
        current = current.get(&key).unwrap_or(&null);
      } else if let Ok(index) = key_raw.parse::<usize>() {
        current = current.get(index).unwrap_or(&null);
      } else {
        return Err(ExpressionError::Parse(format!("invalid index: {key_raw}")));
      }
      rest = &after[close + 1..];
      continue;
    }
    return Err(ExpressionError::Parse(format!("unexpected token: {rest}")));
  }
}

fn split_ident(input: &str) -> (&str, &str) {
  let end = input.char_indices().find(|(_, ch)| !(ch.is_alphanumeric() || *ch == '_')).map(|(index, _)| index).unwrap_or(input.len());
  (&input[..end], &input[end..])
}

fn parse_string_literal(input: &str) -> Option<String> {
  let trimmed = input.trim();
  if trimmed.len() >= 2 && ((trimmed.starts_with('"') && trimmed.ends_with('"')) || (trimmed.starts_with('\'') && trimmed.ends_with('\''))) {
    Some(trimmed[1..trimmed.len() - 1].to_string())
  } else {
    None
  }
}

fn value_to_string(value: &Value) -> String {
  match value {
    Value::Null => String::new(),
    Value::String(text) => text.clone(),
    Value::Bool(flag) => flag.to_string(),
    Value::Number(number) => number.to_string(),
    Value::Array(_) | Value::Object(_) => value.to_string(),
  }
}

pub fn resolve_field(raw: &Value, expr_ctx: &ExpressionContext<'_>, credentials: &CredentialStore) -> Result<Value, ExpressionError> {
  match raw {
    Value::Object(map) => match map.get("mode").and_then(Value::as_str) {
      Some("expression") => {
        let text =
          map.get("expression").and_then(Value::as_str).ok_or_else(|| ExpressionError::Parse("expression binding missing `expression`".to_string()))?;
        Ok(Value::String(evaluate_template(text, expr_ctx)?))
      },
      Some("credential") => {
        let credential_id =
          map.get("credential_id").and_then(Value::as_str).ok_or_else(|| ExpressionError::Parse("credential binding missing `credential_id`".to_string()))?;
        let record = credentials.get(credential_id).ok_or_else(|| ExpressionError::Undefined(format!("credential `{credential_id}` not found in store")))?;
        Ok(record.data)
      },
      Some("literal") => Ok(map.get("value").cloned().unwrap_or(Value::Null)),
      _ => Ok(raw.clone()),
    },
    other => Ok(other.clone()),
  }
}

pub fn resolve_string(raw: &Value, expr_ctx: &ExpressionContext<'_>, credentials: &CredentialStore) -> Result<String, ExpressionError> {
  let resolved = resolve_field(raw, expr_ctx, credentials)?;
  Ok(match resolved {
    Value::String(text) => text,
    Value::Null => String::new(),
    other => other.to_string(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  use std::collections::HashMap;

  fn make_context_with_node_output(node_id: &str, port_id: &str, payload: Value) -> ExecutionContext {
    let mut context = ExecutionContext::default();
    let mut outputs = HashMap::new();
    outputs.insert(port_id.to_string(), vec![NodeItem::from_json(payload)]);
    context.set_node_outputs(node_id, outputs);
    context
  }

  #[test]
  fn literal_template_passes_through() {
    let exec = ExecutionContext::default();
    let ctx = ExpressionContext { exec: &exec, current_item: None, current_node_id: "x" };
    assert_eq!(evaluate_template("hello world", &ctx).unwrap(), "hello world");
  }

  #[test]
  fn json_access_with_nested_fields() {
    let exec = ExecutionContext::default();
    let item = NodeItem::from_json(json!({ "user": { "name": "Ada", "age": 36 } }));
    let ctx = ExpressionContext { exec: &exec, current_item: Some(&item), current_node_id: "x" };
    assert_eq!(evaluate_template("hi {{ $json.user.name }}, {{ $json.user.age }}", &ctx).unwrap(), "hi Ada, 36");
  }

  #[test]
  fn node_reference_resolves_json_path() {
    let exec = make_context_with_node_output("fetch", "out", json!({ "title": "Headline" }));
    let ctx = ExpressionContext { exec: &exec, current_item: None, current_node_id: "consumer" };
    assert_eq!(evaluate_template("{{ $node[\"fetch\"].json.title }}", &ctx).unwrap(), "Headline");
  }

  #[test]
  fn bracket_indexing_supports_strings_and_indexes() {
    let exec = ExecutionContext::default();
    let item = NodeItem::from_json(json!({ "items": [ { "id": "a" }, { "id": "b" } ], "with space": "ok" }));
    let ctx = ExpressionContext { exec: &exec, current_item: Some(&item), current_node_id: "x" };
    assert_eq!(evaluate_template("{{ $json.items[1].id }}", &ctx).unwrap(), "b");
    assert_eq!(evaluate_template("{{ $json[\"with space\"] }}", &ctx).unwrap(), "ok");
  }

  #[test]
  fn now_resolves_to_numeric_timestamp() {
    let exec = ExecutionContext::default();
    let ctx = ExpressionContext { exec: &exec, current_item: None, current_node_id: "x" };
    let now_value = evaluate_template("{{ $now }}", &ctx).unwrap();
    assert!(!now_value.is_empty() && now_value.chars().all(|ch| ch.is_ascii_digit()));
  }

  #[test]
  fn env_missing_variable_yields_empty() {
    let exec = ExecutionContext::default();
    let ctx = ExpressionContext { exec: &exec, current_item: None, current_node_id: "x" };
    assert_eq!(evaluate_template("{{ $env.NODEFLOW_DEFINITELY_NOT_SET_VAR_XYZ }}", &ctx).unwrap(), "");
  }

  #[test]
  fn missing_field_yields_empty_string() {
    let exec = ExecutionContext::default();
    let item = NodeItem::from_json(json!({ "a": 1 }));
    let ctx = ExpressionContext { exec: &exec, current_item: Some(&item), current_node_id: "x" };
    assert_eq!(evaluate_template("{{ $json.missing }}", &ctx).unwrap(), "");
  }

  #[test]
  fn unclosed_expression_errors() {
    let exec = ExecutionContext::default();
    let ctx = ExpressionContext { exec: &exec, current_item: None, current_node_id: "x" };
    assert!(evaluate_template("hi {{ $json.x", &ctx).is_err());
  }
}
