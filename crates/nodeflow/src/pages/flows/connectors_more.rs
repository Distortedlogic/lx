use serde_json::json;

use lx_graph_editor::catalog::{
  GraphCredentialRequirement, GraphExpressionSupport, GraphFieldCapabilities, GraphFieldKind, GraphFieldSchema, GraphNodeTemplate, GraphPortTemplate,
  GraphPortType, PortDirection,
};

pub fn more_connector_templates() -> Vec<GraphNodeTemplate> {
  vec![
    discord_template(),
    telegram_template(),
    github_issue_template(),
    notion_append_template(),
    airtable_create_template(),
    google_sheets_append_template(),
    smtp_send_template(),
    sqlite_query_template(),
    split_out_template(),
    sticky_note_template(),
    error_trigger_template(),
  ]
}

fn discord_template() -> GraphNodeTemplate {
  let capabilities = credential_caps("discord", "Discord webhook credential");
  GraphNodeTemplate {
    id: "discord_webhook".to_string(),
    label: "Discord Webhook".to_string(),
    description: Some("Post a message to a Discord channel via webhook URL.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Discord".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![
      field_with_expression("content", "Message", GraphFieldKind::TextArea, true, json!("Hello from nodeflow")),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
}

fn telegram_template() -> GraphNodeTemplate {
  let capabilities = credential_caps("telegram", "Telegram bot credential");
  GraphNodeTemplate {
    id: "telegram_send".to_string(),
    label: "Telegram Send".to_string(),
    description: Some("Send a message via a Telegram bot.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Telegram".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![
      field_with_expression("chat_id", "Chat ID", GraphFieldKind::Text, true, json!("")),
      field_with_expression("text", "Text", GraphFieldKind::TextArea, true, json!("Hello from nodeflow")),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
}

fn github_issue_template() -> GraphNodeTemplate {
  let capabilities = credential_caps("github", "GitHub token credential");
  GraphNodeTemplate {
    id: "github_issue_create".to_string(),
    label: "GitHub Create Issue".to_string(),
    description: Some("Create an issue on a GitHub repo.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("GitHub Issue".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![
      field_with_expression("owner", "Owner", GraphFieldKind::Text, true, json!("")),
      field_with_expression("repo", "Repo", GraphFieldKind::Text, true, json!("")),
      field_with_expression("title", "Title", GraphFieldKind::Text, true, json!("Issue from nodeflow")),
      field_with_expression("body", "Body", GraphFieldKind::TextArea, false, json!("")),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
}

fn notion_append_template() -> GraphNodeTemplate {
  let capabilities = credential_caps("notion", "Notion token credential");
  GraphNodeTemplate {
    id: "notion_page_append".to_string(),
    label: "Notion Append".to_string(),
    description: Some("Append a paragraph block to a Notion page.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Notion".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![
      field_with_expression("page_id", "Page ID", GraphFieldKind::Text, true, json!("")),
      field_with_expression("text", "Text", GraphFieldKind::TextArea, true, json!("Logged from nodeflow")),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
}

fn airtable_create_template() -> GraphNodeTemplate {
  let capabilities = credential_caps("airtable", "Airtable api_key credential");
  GraphNodeTemplate {
    id: "airtable_record_create".to_string(),
    label: "Airtable Create Record".to_string(),
    description: Some("Create a record in an Airtable base/table.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Airtable".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![
      field_with_expression("base_id", "Base ID", GraphFieldKind::Text, true, json!("")),
      field_with_expression("table", "Table", GraphFieldKind::Text, true, json!("")),
      field_with_expression("fields_json", "Fields JSON", GraphFieldKind::TextArea, true, json!("{}")),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
}

fn google_sheets_append_template() -> GraphNodeTemplate {
  let capabilities = credential_caps("google_oauth", "Google OAuth access_token credential");
  GraphNodeTemplate {
    id: "google_sheets_append".to_string(),
    label: "Google Sheets Append".to_string(),
    description: Some("Append rows to a Google Sheet.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Sheets".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![
      field_with_expression("spreadsheet_id", "Spreadsheet ID", GraphFieldKind::Text, true, json!("")),
      field_with_expression("range", "Range", GraphFieldKind::Text, true, json!("Sheet1!A:B")),
      field_with_expression("values_json", "Values JSON (array of arrays)", GraphFieldKind::TextArea, true, json!("[[\"hello\", \"world\"]]")),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
}

fn smtp_send_template() -> GraphNodeTemplate {
  let capabilities = credential_caps("smtp", "SMTP credential");
  GraphNodeTemplate {
    id: "smtp_send".to_string(),
    label: "SMTP Send".to_string(),
    description: Some("Send an email via SMTP.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("Email".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![
      field_with_expression("from", "From", GraphFieldKind::Text, true, json!("noreply@example.com")),
      field_with_expression("to", "To", GraphFieldKind::Text, true, json!("you@example.com")),
      field_with_expression("subject", "Subject", GraphFieldKind::Text, true, json!("Hello")),
      field_with_expression("body", "Body", GraphFieldKind::TextArea, true, json!("Message body")),
      field_with_caps("credential", "Credential", GraphFieldKind::Text, true, json!(null), capabilities),
    ],
  }
}

fn sqlite_query_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "sqlite_query".to_string(),
    label: "SQLite Query".to_string(),
    description: Some("Run a SQL statement against a SQLite database file.".to_string()),
    category: Some("connector".to_string()),
    default_label: Some("SQLite".to_string()),
    ports: vec![port_in("input", "Input"), port_out("rows", "Rows")],
    fields: vec![
      field_with_expression("database_path", "Database Path", GraphFieldKind::Text, true, json!("/tmp/nodeflow.sqlite")),
      field_with_expression("query", "Query", GraphFieldKind::TextArea, true, json!("SELECT 1 as hello")),
    ],
  }
}

fn split_out_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "control_split_out".to_string(),
    label: "Split Out".to_string(),
    description: Some("Expand an array field on each input item into individual items.".to_string()),
    category: Some("control".to_string()),
    default_label: Some("Split Out".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![field("field", "Array Field", GraphFieldKind::Text, true, json!("items"))],
  }
}

fn sticky_note_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "sticky_note".to_string(),
    label: "Sticky Note".to_string(),
    description: Some("Visual-only note on the canvas; skipped at runtime.".to_string()),
    category: Some("annotation".to_string()),
    default_label: Some("Note".to_string()),
    ports: Vec::new(),
    fields: vec![field("text", "Text", GraphFieldKind::TextArea, false, json!("Drop a note here."))],
  }
}

fn error_trigger_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "trigger_error".to_string(),
    label: "Error Trigger".to_string(),
    description: Some("Entrypoint activated when another flow aborts; receives the failed execution report.".to_string()),
    category: Some("trigger".to_string()),
    default_label: Some("On Error".to_string()),
    ports: vec![port_out("out", "Output")],
    fields: vec![field_with_expression("target_flow_id", "Target Flow ID", GraphFieldKind::Text, false, json!(""))],
  }
}

fn credential_caps(kind: &str, label: &str) -> GraphFieldCapabilities {
  GraphFieldCapabilities {
    expression: Some(GraphExpressionSupport { language: Some("workflow".to_string()), placeholder: Some("{{ $json.value }}".to_string()) }),
    credential: Some(GraphCredentialRequirement {
      namespace: "workflow".to_string(),
      kind: kind.to_string(),
      label: label.to_string(),
      allow_key_selection: true,
    }),
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
