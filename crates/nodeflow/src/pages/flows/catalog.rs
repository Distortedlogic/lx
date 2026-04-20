use serde_json::json;

use lx_graph_editor::catalog::{
  GraphCredentialRequirement, GraphExpressionSupport, GraphFieldCapabilities, GraphFieldKind, GraphFieldOption, GraphFieldSchema, GraphNodeTemplate,
  GraphPortTemplate, GraphPortType, PortDirection,
};

pub fn sample_workflow_pack_templates() -> Vec<GraphNodeTemplate> {
  workflow_node_templates()
}

pub fn connector_node_templates() -> Vec<GraphNodeTemplate> {
  vec![http_request_template(), slack_post_template()]
}

pub fn workflow_node_templates() -> Vec<GraphNodeTemplate> {
  vec![
    node(
      "topic_input",
      "Topic Input",
      Some("input"),
      Some("Topics"),
      vec![out("topics", "Topics", GraphPortType::workflow("topic"), true)],
      vec![
        field("topics", "Topics", GraphFieldKind::StringList, true, json!(["AI policy"])),
        field("refresh_window_hours", "Refresh Window Hours", GraphFieldKind::Integer, true, json!(12)),
      ],
    ),
    node(
      "curated_sources",
      "Curated Sources",
      Some("input"),
      Some("Sources"),
      vec![out("sources", "Sources", GraphPortType::workflow("source"), true)],
      vec![field("domains", "Domains", GraphFieldKind::StringList, true, json!(["www.ft.com"]))],
    ),
    node(
      "web_fetch",
      "Web Fetch",
      Some("collect"),
      Some("Fetch"),
      vec![
        input("topics", "Topics", GraphPortType::workflow("topic"), true),
        input("sources", "Sources", GraphPortType::workflow("source"), true),
        out("articles", "Articles", GraphPortType::workflow("article"), true),
      ],
      vec![
        field("follow_redirects", "Follow Redirects", GraphFieldKind::Boolean, true, json!(true)),
        field("per_source_limit", "Per Source Limit", GraphFieldKind::Integer, true, json!(6)),
      ],
    ),
    node(
      "extract_signals",
      "Extract Signals",
      Some("analyze"),
      Some("Extract"),
      vec![input("articles", "Articles", GraphPortType::workflow("article"), true), out("signals", "Signals", GraphPortType::workflow("signal"), true)],
      vec![field(
        "focus",
        "Focus",
        GraphFieldKind::TextArea,
        true,
        json!("Major announcements, partnerships, funding, policy shifts, and credible product launches."),
      )],
    ),
    node(
      "dedupe_rank",
      "Dedupe Rank",
      Some("analyze"),
      Some("Score"),
      vec![input("signals", "Signals", GraphPortType::workflow("signal"), true), out("ranked", "Ranked", GraphPortType::workflow("ranked_signal"), true)],
      vec![
        field("entity_overlap_penalty", "Entity Overlap Penalty", GraphFieldKind::Number, true, json!(0.35)),
        field("freshness_bias", "Freshness Bias", GraphFieldKind::Number, true, json!(0.72)),
      ],
    ),
    node(
      "summarize_briefs",
      "Summarize Briefs",
      Some("output"),
      Some("Summarize"),
      vec![input("ranked", "Ranked", GraphPortType::workflow("ranked_signal"), true), out("briefs", "Briefs", GraphPortType::workflow("brief"), true)],
      vec![select_field("style", "Style", true, json!("briefing"), vec![("briefing", "Briefing")])],
    ),
    node(
      "feed_output",
      "Feed Output",
      Some("output"),
      Some("Feed"),
      vec![input("briefs", "Briefs", GraphPortType::workflow("brief"), true)],
      vec![
        field("channel", "Channel", GraphFieldKind::Text, true, json!("daily-intel")),
        field("max_items", "Max Items", GraphFieldKind::Integer, true, json!(12)),
      ],
    ),
  ]
}

fn http_request_template() -> GraphNodeTemplate {
  let capabilities = GraphFieldCapabilities {
    expression: Some(GraphExpressionSupport { language: Some("workflow".to_string()), placeholder: Some("{{ steps.fetch.output }}".to_string()) }),
    credential: Some(GraphCredentialRequirement {
      namespace: "workflow".to_string(),
      kind: "http_api".to_string(),
      label: "HTTP API credential".to_string(),
      allow_key_selection: true,
    }),
  };
  node(
    "http_request",
    "HTTP Request",
    Some("connector"),
    Some("HTTP"),
    vec![out("response", "Response", GraphPortType::workflow("http_response"), true)],
    vec![
      field_with_caps("url", "URL", GraphFieldKind::Text, true, json!("https://example.com"), capabilities.clone()),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, false, json!(null), capabilities),
    ],
  )
}

fn slack_post_template() -> GraphNodeTemplate {
  let capabilities = GraphFieldCapabilities {
    expression: Some(GraphExpressionSupport { language: Some("workflow".to_string()), placeholder: Some("{{ steps.summarize.output }}".to_string()) }),
    credential: Some(GraphCredentialRequirement {
      namespace: "workflow".to_string(),
      kind: "slack_bot".to_string(),
      label: "Slack bot credential".to_string(),
      allow_key_selection: false,
    }),
  };
  node(
    "slack_post",
    "Slack Post",
    Some("connector"),
    Some("Slack"),
    vec![input("briefs", "Briefs", GraphPortType::workflow("brief"), false)],
    vec![
      field("channel", "Channel", GraphFieldKind::Text, true, json!("#research-intel")),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  )
}

fn node(
  id: &str,
  label: &str,
  category: Option<&str>,
  default_label: Option<&str>,
  ports: Vec<GraphPortTemplate>,
  fields: Vec<GraphFieldSchema>,
) -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: id.to_string(),
    label: label.to_string(),
    description: None,
    category: category.map(ToOwned::to_owned),
    default_label: default_label.map(ToOwned::to_owned),
    ports,
    fields,
  }
}

fn field(id: &str, label: &str, kind: GraphFieldKind, required: bool, default_value: serde_json::Value) -> GraphFieldSchema {
  field_with_caps(id, label, kind, required, default_value, GraphFieldCapabilities::default())
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

fn select_field(id: &str, label: &str, required: bool, default_value: serde_json::Value, options: Vec<(&str, &str)>) -> GraphFieldSchema {
  field(
    id,
    label,
    GraphFieldKind::Select {
      options: options.into_iter().map(|(value, label)| GraphFieldOption { value: value.to_string(), label: label.to_string() }).collect(),
    },
    required,
    default_value,
  )
}

fn input(id: &str, label: &str, data_type: GraphPortType, required: bool) -> GraphPortTemplate {
  GraphPortTemplate {
    id: id.to_string(),
    label: label.to_string(),
    description: None,
    direction: PortDirection::Input,
    data_type: Some(data_type),
    required,
    allow_multiple: true,
  }
}

fn out(id: &str, label: &str, data_type: GraphPortType, required: bool) -> GraphPortTemplate {
  GraphPortTemplate {
    id: id.to_string(),
    label: label.to_string(),
    description: None,
    direction: PortDirection::Output,
    data_type: Some(data_type),
    required,
    allow_multiple: true,
  }
}
