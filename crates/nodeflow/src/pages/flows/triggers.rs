use serde_json::json;

use lx_graph_editor::catalog::{GraphFieldCapabilities, GraphFieldKind, GraphFieldSchema, GraphNodeTemplate, GraphPortTemplate, GraphPortType, PortDirection};

pub fn trigger_node_templates() -> Vec<GraphNodeTemplate> {
  vec![manual_trigger_template(), cron_trigger_template(), webhook_trigger_template(), sub_workflow_template()]
}

fn manual_trigger_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "trigger_manual".to_string(),
    label: "Manual Trigger".to_string(),
    description: Some("Entrypoint fired when the user clicks Run.".to_string()),
    category: Some("trigger".to_string()),
    default_label: Some("Manual".to_string()),
    ports: vec![port_out("out", "Output")],
    fields: vec![field("payload", "Payload (JSON)", GraphFieldKind::TextArea, false, json!("{}"))],
  }
}

fn cron_trigger_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "trigger_cron".to_string(),
    label: "Cron Trigger".to_string(),
    description: Some("Entrypoint fired on a cron schedule.".to_string()),
    category: Some("trigger".to_string()),
    default_label: Some("Cron".to_string()),
    ports: vec![port_out("out", "Output")],
    fields: vec![field("cron_expression", "Cron Expression", GraphFieldKind::Text, true, json!("0 */5 * * * *"))],
  }
}

fn webhook_trigger_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "trigger_webhook".to_string(),
    label: "Webhook Trigger".to_string(),
    description: Some("Entrypoint fired when an HTTP request hits the flow's unique webhook path.".to_string()),
    category: Some("trigger".to_string()),
    default_label: Some("Webhook".to_string()),
    ports: vec![port_out("out", "Output")],
    fields: vec![
      field("path", "Path (after /webhook)", GraphFieldKind::Text, true, json!("hello")),
      field(
        "method",
        "Method",
        GraphFieldKind::Select {
          options: ["ANY", "GET", "POST", "PUT", "PATCH", "DELETE"]
            .iter()
            .map(|value| lx_graph_editor::catalog::GraphFieldOption { value: value.to_string(), label: value.to_string() })
            .collect(),
        },
        true,
        json!("ANY"),
      ),
    ],
  }
}

fn sub_workflow_template() -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: "control_sub_workflow".to_string(),
    label: "Sub-workflow".to_string(),
    description: Some("Run another flow by id; pass the current items as its trigger payload.".to_string()),
    category: Some("control".to_string()),
    default_label: Some("Sub-flow".to_string()),
    ports: vec![port_in("input", "Input"), port_out("out", "Output")],
    fields: vec![field("flow_id", "Flow ID", GraphFieldKind::Text, true, json!(""))],
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
