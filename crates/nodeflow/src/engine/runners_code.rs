use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{merged_inputs, properties_lookup};
use super::types::{NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_code_runner(registry: &mut NodeRunnerRegistry) {
  registry.register("control_code", Arc::new(CodeRunner));
}

pub struct CodeRunner;

#[async_trait]
impl NodeRunner for CodeRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let code = properties_lookup(&ctx.node.properties, "code").as_str().map(ToOwned::to_owned).unwrap_or_default();
    if code.trim().is_empty() {
      return Err(NodeExecutionError::Runtime("code node: script is empty".to_string()));
    }
    let items = merged_inputs(&ctx.inputs);

    let mut engine = rhai::Engine::new();
    engine.set_max_operations(1_000_000);
    engine.set_max_expr_depths(64, 64);

    let mut scope = rhai::Scope::new();
    let items_dynamic: rhai::Array = items.iter().map(|item| json_to_dynamic(&item.json)).collect();
    scope.push("items", items_dynamic);

    let result = engine.eval_with_scope::<rhai::Dynamic>(&mut scope, &code).map_err(|error| NodeExecutionError::Runtime(format!("rhai error: {error}")))?;

    let output_items = dynamic_to_items(result);
    let mut outputs = HashMap::new();
    let count = output_items.len();
    outputs.insert("out".to_string(), output_items);
    Ok(NodeRunOutcome { outputs, logs: vec![format!("Code produced {count} items")] })
  }
}

fn json_to_dynamic(value: &Value) -> rhai::Dynamic {
  match value {
    Value::Null => rhai::Dynamic::UNIT,
    Value::Bool(flag) => rhai::Dynamic::from_bool(*flag),
    Value::Number(number) => {
      if let Some(int) = number.as_i64() {
        rhai::Dynamic::from_int(int)
      } else if let Some(float) = number.as_f64() {
        rhai::Dynamic::from_float(float)
      } else {
        rhai::Dynamic::from(number.to_string())
      }
    },
    Value::String(text) => rhai::Dynamic::from(text.clone()),
    Value::Array(items) => rhai::Dynamic::from(items.iter().map(json_to_dynamic).collect::<rhai::Array>()),
    Value::Object(map) => {
      let mut object = rhai::Map::new();
      for (key, value) in map {
        object.insert(key.into(), json_to_dynamic(value));
      }
      rhai::Dynamic::from(object)
    },
  }
}

fn dynamic_to_value(dynamic: rhai::Dynamic) -> Value {
  if dynamic.is_unit() {
    return Value::Null;
  }
  if let Ok(flag) = dynamic.as_bool() {
    return Value::Bool(flag);
  }
  if let Ok(int) = dynamic.as_int() {
    return Value::Number(int.into());
  }
  if let Ok(float) = dynamic.as_float() {
    return serde_json::Number::from_f64(float).map(Value::Number).unwrap_or(Value::Null);
  }
  if dynamic.is::<rhai::Array>() {
    let array: rhai::Array = dynamic.cast();
    return Value::Array(array.into_iter().map(dynamic_to_value).collect());
  }
  if dynamic.is::<rhai::Map>() {
    let map: rhai::Map = dynamic.cast();
    let mut object = serde_json::Map::new();
    for (key, value) in map {
      object.insert(key.to_string(), dynamic_to_value(value));
    }
    return Value::Object(object);
  }
  Value::String(dynamic.to_string())
}

fn dynamic_to_items(dynamic: rhai::Dynamic) -> Vec<NodeItem> {
  if dynamic.is::<rhai::Array>() {
    let array: rhai::Array = dynamic.cast();
    return array.into_iter().map(|item| NodeItem::from_json(dynamic_to_value(item))).collect();
  }
  vec![NodeItem::from_json(dynamic_to_value(dynamic))]
}
