pub mod catalog;
pub mod commands;
pub mod dioxus;
pub mod history;
pub mod inspector;
pub mod model;
pub mod protocol;

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::catalog::{GraphFieldKind, GraphFieldOption, GraphFieldSchema, GraphNodeTemplate, GraphPortTemplate, GraphPortType, PortDirection};
  use super::commands::{GraphCommand, GraphCommandError, apply_graph_command};
  use super::model::{GraphDocument, GraphPoint, GraphPortRef, GraphSelection, GraphViewport};
  use super::protocol::{GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity, GraphWidgetEvent, GraphWidgetSnapshot};

  fn workflow_templates() -> Vec<GraphNodeTemplate> {
    vec![
      GraphNodeTemplate {
        id: "topic_source".to_string(),
        label: "Topic Source".to_string(),
        description: Some("Produces topic strings".to_string()),
        category: Some("input".to_string()),
        default_label: Some("Topics".to_string()),
        ports: vec![GraphPortTemplate {
          id: "topics".to_string(),
          label: "Topics".to_string(),
          description: None,
          direction: PortDirection::Output,
          data_type: Some(GraphPortType::workflow("topic")),
          required: true,
          allow_multiple: true,
        }],
        fields: vec![
          GraphFieldSchema {
            id: "query".to_string(),
            label: "Query".to_string(),
            description: None,
            kind: GraphFieldKind::Text,
            required: true,
            default_value: Some(json!("ai safety")),
            capabilities: Default::default(),
          },
          GraphFieldSchema {
            id: "sources".to_string(),
            label: "Sources".to_string(),
            description: None,
            kind: GraphFieldKind::StringList,
            required: true,
            default_value: Some(json!(["https://example.com"])),
            capabilities: Default::default(),
          },
        ],
      },
      GraphNodeTemplate {
        id: "summarizer".to_string(),
        label: "Summarizer".to_string(),
        description: Some("Summarizes fetched documents".to_string()),
        category: Some("transform".to_string()),
        default_label: Some("Summarize".to_string()),
        ports: vec![
          GraphPortTemplate {
            id: "documents".to_string(),
            label: "Documents".to_string(),
            description: None,
            direction: PortDirection::Input,
            data_type: Some(GraphPortType::workflow("topic")),
            required: true,
            allow_multiple: false,
          },
          GraphPortTemplate {
            id: "summary".to_string(),
            label: "Summary".to_string(),
            description: None,
            direction: PortDirection::Output,
            data_type: Some(GraphPortType::workflow("summary")),
            required: true,
            allow_multiple: true,
          },
        ],
        fields: vec![GraphFieldSchema {
          id: "style".to_string(),
          label: "Style".to_string(),
          description: None,
          kind: GraphFieldKind::Select {
            options: vec![
              GraphFieldOption { value: "brief".to_string(), label: "Brief".to_string() },
              GraphFieldOption { value: "full".to_string(), label: "Full".to_string() },
            ],
          },
          required: true,
          default_value: Some(json!("brief")),
          capabilities: Default::default(),
        }],
      },
    ]
  }

  #[test]
  fn creates_a_node_from_template_defaults() {
    let templates = workflow_templates();
    let mut document = GraphDocument::new("flow-1", "Research Flow");

    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode { node_id: "node-1".to_string(), template_id: "topic_source".to_string(), position: GraphPoint { x: 120.0, y: 80.0 }, label: None },
    )
    .expect("add node");

    let node = document.node("node-1").expect("node exists");
    assert_eq!(node.label.as_deref(), Some("Topics"));
    assert_eq!(node.properties["query"], json!("ai safety"));
    assert_eq!(node.properties["sources"], json!(["https://example.com"]));
    assert_eq!(document.selection, GraphSelection::single_node("node-1"));
  }

  #[test]
  fn moves_a_node() {
    let templates = workflow_templates();
    let mut document = GraphDocument::new("flow-1", "Research Flow");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode {
        node_id: "node-1".to_string(),
        template_id: "topic_source".to_string(),
        position: GraphPoint { x: 0.0, y: 0.0 },
        label: Some("Start".to_string()),
      },
    )
    .expect("add node");

    apply_graph_command(&mut document, &templates, GraphCommand::MoveNode { node_id: "node-1".to_string(), position: GraphPoint { x: 320.0, y: 240.0 } })
      .expect("move node");

    assert_eq!(document.node("node-1").expect("node exists").position, GraphPoint { x: 320.0, y: 240.0 });
  }

  #[test]
  fn connects_and_disconnects_ports() {
    let templates = workflow_templates();
    let mut document = GraphDocument::new("flow-1", "Research Flow");
    for (node_id, template_id, x) in [("node-1", "topic_source", 0.0), ("node-2", "summarizer", 240.0)] {
      apply_graph_command(
        &mut document,
        &templates,
        GraphCommand::AddNode { node_id: node_id.to_string(), template_id: template_id.to_string(), position: GraphPoint { x, y: 0.0 }, label: None },
      )
      .expect("add node");
    }

    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::ConnectPorts {
        edge_id: "edge-1".to_string(),
        from: GraphPortRef { node_id: "node-1".to_string(), port_id: "topics".to_string() },
        to: GraphPortRef { node_id: "node-2".to_string(), port_id: "documents".to_string() },
        label: Some("feeds".to_string()),
      },
    )
    .expect("connect ports");

    assert_eq!(document.edges.len(), 1);
    assert_eq!(document.selection, GraphSelection::single_edge("edge-1"));

    apply_graph_command(&mut document, &templates, GraphCommand::DisconnectEdge { edge_id: "edge-1".to_string() }).expect("disconnect edge");

    assert!(document.edges.is_empty());
    assert!(document.selection.is_empty());
  }

  #[test]
  fn rejects_invalid_connections() {
    let templates = workflow_templates();
    let mut document = GraphDocument::new("flow-1", "Research Flow");
    for (node_id, template_id, x) in [("node-1", "topic_source", 0.0), ("node-2", "summarizer", 240.0)] {
      apply_graph_command(
        &mut document,
        &templates,
        GraphCommand::AddNode { node_id: node_id.to_string(), template_id: template_id.to_string(), position: GraphPoint { x, y: 0.0 }, label: None },
      )
      .expect("add node");
    }

    let error = apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::ConnectPorts {
        edge_id: "edge-1".to_string(),
        from: GraphPortRef { node_id: "node-2".to_string(), port_id: "documents".to_string() },
        to: GraphPortRef { node_id: "node-1".to_string(), port_id: "topics".to_string() },
        label: None,
      },
    )
    .expect_err("reversed ports should fail");

    assert_eq!(error, GraphCommandError::InvalidConnectionDirection { from: PortDirection::Input, to: PortDirection::Output });
  }

  #[test]
  fn accepts_qualified_output_for_more_general_input() {
    let templates = vec![
      GraphNodeTemplate {
        id: "producer".to_string(),
        label: "Producer".to_string(),
        description: None,
        category: Some("lx".to_string()),
        default_label: Some("Producer".to_string()),
        ports: vec![GraphPortTemplate {
          id: "artifact".to_string(),
          label: "Artifact".to_string(),
          description: None,
          direction: PortDirection::Output,
          data_type: Some(GraphPortType::qualified("lx", "artifact", ["research_brief"])),
          required: true,
          allow_multiple: true,
        }],
        fields: vec![],
      },
      GraphNodeTemplate {
        id: "consumer".to_string(),
        label: "Consumer".to_string(),
        description: None,
        category: Some("lx".to_string()),
        default_label: Some("Consumer".to_string()),
        ports: vec![GraphPortTemplate {
          id: "artifact".to_string(),
          label: "Artifact".to_string(),
          description: None,
          direction: PortDirection::Input,
          data_type: Some(GraphPortType::new("lx", "artifact")),
          required: true,
          allow_multiple: false,
        }],
        fields: vec![],
      },
    ];

    let mut document = GraphDocument::new("flow-1", "LX Flow");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode { node_id: "producer-1".to_string(), template_id: "producer".to_string(), position: GraphPoint { x: 0.0, y: 0.0 }, label: None },
    )
    .expect("add producer");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode { node_id: "consumer-1".to_string(), template_id: "consumer".to_string(), position: GraphPoint { x: 240.0, y: 0.0 }, label: None },
    )
    .expect("add consumer");

    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::ConnectPorts {
        edge_id: "edge-1".to_string(),
        from: GraphPortRef { node_id: "producer-1".to_string(), port_id: "artifact".to_string() },
        to: GraphPortRef { node_id: "consumer-1".to_string(), port_id: "artifact".to_string() },
        label: None,
      },
    )
    .expect("qualified output should satisfy general input");

    assert_eq!(document.edges.len(), 1);
  }

  #[test]
  fn deletes_the_current_selection() {
    let templates = workflow_templates();
    let mut document = GraphDocument::new("flow-1", "Research Flow");
    for (node_id, template_id, x) in [("node-1", "topic_source", 0.0), ("node-2", "summarizer", 240.0)] {
      apply_graph_command(
        &mut document,
        &templates,
        GraphCommand::AddNode { node_id: node_id.to_string(), template_id: template_id.to_string(), position: GraphPoint { x, y: 0.0 }, label: None },
      )
      .expect("add node");
    }
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::ConnectPorts {
        edge_id: "edge-1".to_string(),
        from: GraphPortRef { node_id: "node-1".to_string(), port_id: "topics".to_string() },
        to: GraphPortRef { node_id: "node-2".to_string(), port_id: "documents".to_string() },
        label: None,
      },
    )
    .expect("connect ports");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::Select {
        selection: GraphSelection {
          anchor: Some(super::model::GraphEntityRef::Node("node-1".to_string())),
          node_ids: vec!["node-1".to_string()],
          edge_ids: vec!["edge-1".to_string()],
        },
      },
    )
    .expect("select graph entities");

    apply_graph_command(&mut document, &templates, GraphCommand::DeleteSelection).expect("delete selection");

    assert_eq!(document.nodes.len(), 1);
    assert_eq!(document.nodes[0].id, "node-2");
    assert!(document.edges.is_empty());
    assert!(document.selection.is_empty());
  }

  #[test]
  fn serializes_widget_snapshot_and_events() {
    let templates = workflow_templates();
    let mut document = GraphDocument::new("flow-1", "Research Flow");
    document.viewport = GraphViewport { pan_x: 32.0, pan_y: 48.0, zoom: 1.25 };
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode { node_id: "node-1".to_string(), template_id: "topic_source".to_string(), position: GraphPoint { x: 12.0, y: 18.0 }, label: None },
    )
    .expect("add node");

    let snapshot = GraphWidgetSnapshot {
      document,
      templates: templates.clone(),
      diagnostics: vec![GraphWidgetDiagnostic {
        id: "missing-summary".to_string(),
        severity: GraphWidgetDiagnosticSeverity::Warning,
        message: "Missing summary sink".to_string(),
        target: Some(super::model::GraphEntityRef::Node("node-1".to_string())),
      }],
      run_snapshot: None,
    };
    let event = GraphWidgetEvent::ViewportChanged { viewport: GraphViewport { pan_x: 0.0, pan_y: 0.0, zoom: 0.8 } };

    let snapshot_json = serde_json::to_value(&snapshot).expect("serialize snapshot");
    let event_json = serde_json::to_value(&event).expect("serialize event");

    assert_eq!(snapshot_json["document"]["title"], json!("Research Flow"));
    assert_eq!(snapshot_json["diagnostics"][0]["severity"], json!("warning"));
    assert_eq!(event_json["type"], json!("viewport_changed"));
    assert_eq!(event_json["viewport"]["zoom"], json!(0.8));
  }
}
