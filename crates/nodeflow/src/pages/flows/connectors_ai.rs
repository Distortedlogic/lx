use serde_json::json;

use lx_graph_editor::catalog::{
  GraphCredentialRequirement, GraphExpressionSupport, GraphFieldCapabilities, GraphFieldKind, GraphFieldSchema, GraphNodeTemplate, GraphPortTemplate,
  GraphPortType, PortDirection,
};

pub fn ai_connector_templates() -> Vec<GraphNodeTemplate> {
  vec![anthropic_messages_template(), openai_chat_template(), file_read_template(), file_write_template(), postgres_query_template()]
}

fn anthropic_messages_template() -> GraphNodeTemplate {
  let capabilities = GraphFieldCapabilities {
    expression: Some(GraphExpressionSupport { language: Some("workflow".to_string()), placeholder: Some("{{ $json.text }}".to_string()) }),
    credential: Some(GraphCredentialRequirement {
      namespace: "workflow".to_string(),
      kind: "anthropic".to_string(),
      label: "Anthropic API credential".to_string(),
      allow_key_selection: true,
    }),
  };
  GraphNodeTemplate {
    id: "anthropic_messages".to_string(),
    label: "Anthropic Message".to_string(),
    description: Some("Call Anthropic /v1/messages with a prompt; returns assistant text.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Claude".to_string()),
    ports: vec![port_in("input", "Input"), port_out("response", "Response")],
    fields: vec![
      field_with_caps("model", "Model", GraphFieldKind::Text, true, json!("claude-opus-4-7"), capabilities.clone()),
      field_with_caps("prompt", "Prompt", GraphFieldKind::TextArea, true, json!("Summarize: {{ $json.text }}"), capabilities.clone()),
      field_with_caps("max_tokens", "Max Tokens", GraphFieldKind::Integer, true, json!(1024), capabilities.clone()),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
}

fn openai_chat_template() -> GraphNodeTemplate {
  let capabilities = GraphFieldCapabilities {
    expression: Some(GraphExpressionSupport { language: Some("workflow".to_string()), placeholder: Some("{{ $json.text }}".to_string()) }),
    credential: Some(GraphCredentialRequirement {
      namespace: "workflow".to_string(),
      kind: "openai".to_string(),
      label: "OpenAI API credential".to_string(),
      allow_key_selection: true,
    }),
  };
  GraphNodeTemplate {
    id: "openai_chat".to_string(),
    label: "OpenAI Chat".to_string(),
    description: Some("Call OpenAI chat completions with a prompt.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("GPT".to_string()),
    ports: vec![port_in("input", "Input"), port_out("response", "Response")],
    fields: vec![
      field_with_caps("model", "Model", GraphFieldKind::Text, true, json!("gpt-4o-mini"), capabilities.clone()),
      field_with_caps("prompt", "Prompt", GraphFieldKind::TextArea, true, json!("Summarize: {{ $json.text }}"), capabilities.clone()),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
}

fn file_read_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "file_read".to_string(),
    label: "File Read".to_string(),
    description: Some("Read a file from local disk.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Read".to_string()),
    ports: vec![port_in("input", "Input"), port_out("content", "Content")],
    fields: vec![
      field_with_expression("path", "Path", GraphFieldKind::Text, true, json!("/tmp/example.txt")),
      field(
        "format",
        "Format",
        GraphFieldKind::Select {
          options: [("text", "Text"), ("json", "JSON"), ("bytes_base64", "Bytes (base64)")]
            .iter()
            .map(|(value, label)| lx_graph_editor::catalog::GraphFieldOption { value: value.to_string(), label: label.to_string() })
            .collect(),
        },
        true,
        json!("text"),
      ),
    ],
  }
}

fn file_write_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "file_write".to_string(),
    label: "File Write".to_string(),
    description: Some("Write a string or JSON to a file on disk.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Write".to_string()),
    ports: vec![port_in("input", "Input"), port_out("result", "Result")],
    fields: vec![
      field_with_expression("path", "Path", GraphFieldKind::Text, true, json!("/tmp/out.txt")),
      field_with_expression("content", "Content", GraphFieldKind::TextArea, true, json!("{{ $json }}")),
      field("append", "Append", GraphFieldKind::Boolean, false, json!(false)),
    ],
  }
}

fn postgres_query_template() -> GraphNodeTemplate {
  let capabilities = GraphFieldCapabilities {
    expression: Some(GraphExpressionSupport { language: Some("workflow".to_string()), placeholder: Some("{{ $json.id }}".to_string()) }),
    credential: Some(GraphCredentialRequirement {
      namespace: "workflow".to_string(),
      kind: "postgres".to_string(),
      label: "Postgres credential".to_string(),
      allow_key_selection: true,
    }),
  };
  GraphNodeTemplate {
    id: "postgres_query".to_string(),
    label: "Postgres Query".to_string(),
    description: Some("Placeholder for Postgres queries (runner not yet implemented).".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Postgres".to_string()),
    ports: vec![port_in("input", "Input"), port_out("rows", "Rows")],
    fields: vec![
      field_with_caps("query", "SQL", GraphFieldKind::TextArea, true, json!("select now()"), capabilities.clone()),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
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

fn field_with_caps(
  id: &str,
  label: &str,
  kind: GraphFieldKind,
  required: bool,
  default_value: serde_json::Value,
  capabilities: GraphFieldCapabilities,
) -> GraphFieldSchema {
  GraphFieldSchema { id: id.to_string(), label: label.to_string(), description: None, kind, required, default_value: Some(default_value), capabilities }
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
