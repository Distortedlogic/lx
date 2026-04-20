use serde_json::Value;

use super::context::ExecutionContext;
use super::expression::ExpressionContext;
use super::types::NodeItem;

pub fn properties_lookup(props: &Value, key: &str) -> Value {
  props.as_object().and_then(|map| map.get(key)).cloned().unwrap_or(Value::Null)
}

pub fn make_expr_ctx<'a>(exec: &'a ExecutionContext, current_item: Option<&'a NodeItem>, current_node_id: &'a str) -> ExpressionContext<'a> {
  ExpressionContext { exec, current_item, current_node_id }
}

pub fn first_input_item(inputs: &std::collections::HashMap<String, Vec<NodeItem>>) -> Option<&NodeItem> {
  inputs.values().flat_map(|items| items.iter()).next()
}

pub fn merged_inputs(inputs: &std::collections::HashMap<String, Vec<NodeItem>>) -> Vec<NodeItem> {
  inputs.values().flat_map(|items| items.iter().cloned()).collect()
}
