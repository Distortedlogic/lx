use serde_json::json;

use lx_graph_editor::catalog::{
  GraphExpressionSupport, GraphFieldCapabilities, GraphFieldKind, GraphFieldSchema, GraphNodeTemplate, GraphPortTemplate, GraphPortType, PortDirection,
};

pub fn control_flow_node_templates() -> Vec<GraphNodeTemplate> {
  vec![if_template(), switch_template(), merge_template(), wait_template(), set_template(), split_in_batches_template(), code_template()]
}

fn switch_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "control_switch".to_string(),
    label: "Switch".to_string(),
    description: Some("Route items by evaluating up to 3 rules; unmatched go to default.".to_string()),
    category: Some("control".to_string()),
    default_label: Some("Switch".to_string()),
    ports: vec![
      port_in("input", "Input"),
      port_out("case_1", "Case 1"),
      port_out("case_2", "Case 2"),
      port_out("case_3", "Case 3"),
      port_out("default", "Default"),
    ],
    fields: vec![
      field_with_expression("rule_1", "Rule 1 (expression)", GraphFieldKind::Text, false, json!("")),
      field_with_expression("rule_2", "Rule 2 (expression)", GraphFieldKind::Text, false, json!("")),
      field_with_expression("rule_3", "Rule 3 (expression)", GraphFieldKind::Text, false, json!("")),
    ],
  }
}

fn split_in_batches_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "control_split_in_batches".to_string(),
    label: "Split In Batches".to_string(),
    description: Some("Group incoming items into fixed-size batches; emits items tagged with batch metadata.".to_string()),
    category: Some("control".to_string()),
    default_label: Some("Batches".to_string()),
    ports: vec![port_in("input", "Input"), port_out("batches", "Batches")],
    fields: vec![field("batch_size", "Batch Size", GraphFieldKind::Integer, true, json!(10))],
  }
}

fn code_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "control_code".to_string(),
    label: "Code".to_string(),
    description: Some("Run a Rhai script against the input items; return a value that becomes the output.".to_string()),
    category: Some("control".to_string()),
    default_label: Some("Code".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![field("code", "Code (Rhai)", GraphFieldKind::TextArea, true, json!("items"))],
  }
}

fn if_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "control_if".to_string(),
    label: "IF".to_string(),
    description: Some("Route items to the true or false branch based on an expression.".to_string()),
    category: Some("control".to_string()),
    default_label: Some("If".to_string()),
    ports: vec![port_in("input", "Input"), port_out("true", "True"), port_out("false", "False")],
    fields: vec![field_with_expression("condition", "Condition", GraphFieldKind::Text, true, json!(""))],
  }
}

fn merge_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "control_merge".to_string(),
    label: "Merge".to_string(),
    description: Some("Merge items from two branches into a single output.".to_string()),
    category: Some("control".to_string()),
    default_label: Some("Merge".to_string()),
    ports: vec![port_in("input_a", "Input A"), port_in("input_b", "Input B"), port_out("out", "Output")],
    fields: Vec::new(),
  }
}

fn wait_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "control_wait".to_string(),
    label: "Wait".to_string(),
    description: Some("Delay execution for the configured number of seconds.".to_string()),
    category: Some("control".to_string()),
    default_label: Some("Wait".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![field("delay_seconds", "Delay (seconds)", GraphFieldKind::Number, true, json!(1))],
  }
}

fn set_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "control_set".to_string(),
    label: "Set".to_string(),
    description: Some("Overwrite or extend the current item with a JSON object.".to_string()),
    category: Some("control".to_string()),
    default_label: Some("Set".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![
      field_with_expression("assignments", "Assignments (JSON)", GraphFieldKind::TextArea, true, json!("{}")),
      field(
        "mode",
        "Mode",
        GraphFieldKind::Select { options: vec![select_option("merge", "Merge"), select_option("replace", "Replace")] },
        true,
        json!("merge"),
      ),
    ],
  }
}

fn select_option(value: &str, label: &str) -> lx_graph_editor::catalog::GraphFieldOption {
  lx_graph_editor::catalog::GraphFieldOption { value: value.to_string(), label: label.to_string() }
}

fn port_in(id: &str, label: &str) -> GraphPortTemplate {
  GraphPortTemplate {
    id: id.to_string(),
    label: label.to_string(),
    description: None,
    direction: PortDirection::Input,
    data_type: Some(GraphPortType::workflow("any")),
    required: false,
    allow_multiple: true,
  }
}

fn port_out(id: &str, label: &str) -> GraphPortTemplate {
  GraphPortTemplate {
    id: id.to_string(),
    label: label.to_string(),
    description: None,
    direction: PortDirection::Output,
    data_type: Some(GraphPortType::workflow("any")),
    required: false,
    allow_multiple: true,
  }
}

fn field(id: &str, label: &str, kind: GraphFieldKind, required: bool, default_value: serde_json::Value) -> GraphFieldSchema {
  GraphFieldSchema {
    id: id.to_string(),
    label: label.to_string(),
    description: None,
    kind,
    required,
    default_value: Some(default_value),
    capabilities: GraphFieldCapabilities::default(),
  }
}

fn field_with_expression(id: &str, label: &str, kind: GraphFieldKind, required: bool, default_value: serde_json::Value) -> GraphFieldSchema {
  GraphFieldSchema {
    id: id.to_string(),
    label: label.to_string(),
    description: None,
    kind,
    required,
    default_value: Some(default_value),
    capabilities: GraphFieldCapabilities {
      expression: Some(GraphExpressionSupport { language: Some("workflow".to_string()), placeholder: Some("{{ $json.value }}".to_string()) }),
      credential: None,
    },
  }
}
