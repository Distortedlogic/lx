use serde_json::{Value, json};

use crate::graph_editor::catalog::{GraphFieldKind, GraphFieldOption, GraphFieldSchema, GraphNodeTemplate, GraphPortTemplate, PortDirection, node_template};
use crate::graph_editor::model::{GraphDocument, GraphEdge, GraphNode, GraphPoint, GraphPortRef, GraphSelection, GraphViewport};

pub const DEFAULT_FLOW_ID: &str = "newsfeed-research";

pub fn sample_templates() -> Vec<GraphNodeTemplate> {
  vec![
    GraphNodeTemplate {
      id: "topic_input".to_string(),
      label: "Topic Input".to_string(),
      description: Some("Seeds the workflow with monitored themes.".to_string()),
      category: Some("inputs".to_string()),
      default_label: Some("Topics".to_string()),
      ports: vec![GraphPortTemplate {
        id: "topics".to_string(),
        label: "Topics".to_string(),
        description: Some("Topics to monitor".to_string()),
        direction: PortDirection::Output,
        data_type: Some("topics".to_string()),
        required: true,
        allow_multiple: true,
      }],
      fields: vec![
        GraphFieldSchema {
          id: "topics".to_string(),
          label: "Topics".to_string(),
          description: Some("Theme list for the aggregation pass".to_string()),
          kind: GraphFieldKind::StringList,
          required: true,
          default_value: Some(json!(["AI policy", "chip supply chain", "open models"])),
        },
        GraphFieldSchema {
          id: "refresh_window_hours".to_string(),
          label: "Refresh Window".to_string(),
          description: Some("How far back the fetch step should look".to_string()),
          kind: GraphFieldKind::Integer,
          required: true,
          default_value: Some(json!(12)),
        },
      ],
    },
    GraphNodeTemplate {
      id: "curated_sources".to_string(),
      label: "Curated Sources".to_string(),
      description: Some("Source domains and feeds to prioritize".to_string()),
      category: Some("inputs".to_string()),
      default_label: Some("Sources".to_string()),
      ports: vec![GraphPortTemplate {
        id: "sources".to_string(),
        label: "Sources".to_string(),
        description: Some("Preferred source list".to_string()),
        direction: PortDirection::Output,
        data_type: Some("sources".to_string()),
        required: true,
        allow_multiple: true,
      }],
      fields: vec![GraphFieldSchema {
        id: "domains".to_string(),
        label: "Domains".to_string(),
        description: Some("Preferred source domains".to_string()),
        kind: GraphFieldKind::StringList,
        required: true,
        default_value: Some(json!(["www.ft.com", "www.semafor.com", "www.stratechery.com"])),
      }],
    },
    GraphNodeTemplate {
      id: "web_fetch".to_string(),
      label: "Web Fetch".to_string(),
      description: Some("Fetches candidate articles from sources".to_string()),
      category: Some("fetch".to_string()),
      default_label: Some("Fetch".to_string()),
      ports: vec![
        GraphPortTemplate {
          id: "topics".to_string(),
          label: "Topics".to_string(),
          description: None,
          direction: PortDirection::Input,
          data_type: Some("topics".to_string()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "sources".to_string(),
          label: "Sources".to_string(),
          description: None,
          direction: PortDirection::Input,
          data_type: Some("sources".to_string()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "articles".to_string(),
          label: "Articles".to_string(),
          description: None,
          direction: PortDirection::Output,
          data_type: Some("articles".to_string()),
          required: true,
          allow_multiple: true,
        },
      ],
      fields: vec![
        GraphFieldSchema {
          id: "per_source_limit".to_string(),
          label: "Per-source Limit".to_string(),
          description: Some("Maximum articles per source".to_string()),
          kind: GraphFieldKind::Integer,
          required: true,
          default_value: Some(json!(8)),
        },
        GraphFieldSchema {
          id: "follow_redirects".to_string(),
          label: "Follow Redirects".to_string(),
          description: None,
          kind: GraphFieldKind::Boolean,
          required: true,
          default_value: Some(json!(true)),
        },
      ],
    },
    GraphNodeTemplate {
      id: "extract_signals".to_string(),
      label: "Extract Signals".to_string(),
      description: Some("Pulls structured newsworthy signals from raw articles".to_string()),
      category: Some("transform".to_string()),
      default_label: Some("Extract".to_string()),
      ports: vec![
        GraphPortTemplate {
          id: "articles".to_string(),
          label: "Articles".to_string(),
          description: None,
          direction: PortDirection::Input,
          data_type: Some("articles".to_string()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "signals".to_string(),
          label: "Signals".to_string(),
          description: None,
          direction: PortDirection::Output,
          data_type: Some("signals".to_string()),
          required: true,
          allow_multiple: true,
        },
      ],
      fields: vec![GraphFieldSchema {
        id: "focus".to_string(),
        label: "Focus".to_string(),
        description: Some("What to pull out of each document".to_string()),
        kind: GraphFieldKind::TextArea,
        required: true,
        default_value: Some(json!("Major announcements, partnerships, funding, policy shifts, and credible product launches.")),
      }],
    },
    GraphNodeTemplate {
      id: "dedupe_rank".to_string(),
      label: "Dedupe + Rank".to_string(),
      description: Some("Collapses duplicates and scores urgency".to_string()),
      category: Some("transform".to_string()),
      default_label: Some("Score".to_string()),
      ports: vec![
        GraphPortTemplate {
          id: "signals".to_string(),
          label: "Signals".to_string(),
          description: None,
          direction: PortDirection::Input,
          data_type: Some("signals".to_string()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "ranked".to_string(),
          label: "Ranked".to_string(),
          description: None,
          direction: PortDirection::Output,
          data_type: Some("ranked_items".to_string()),
          required: true,
          allow_multiple: true,
        },
      ],
      fields: vec![
        GraphFieldSchema {
          id: "freshness_bias".to_string(),
          label: "Freshness Bias".to_string(),
          description: None,
          kind: GraphFieldKind::Number,
          required: true,
          default_value: Some(json!(0.72)),
        },
        GraphFieldSchema {
          id: "entity_overlap_penalty".to_string(),
          label: "Entity Overlap Penalty".to_string(),
          description: None,
          kind: GraphFieldKind::Number,
          required: true,
          default_value: Some(json!(0.35)),
        },
      ],
    },
    GraphNodeTemplate {
      id: "summarize_briefs".to_string(),
      label: "Summarize Briefs".to_string(),
      description: Some("Produces digest-ready summaries".to_string()),
      category: Some("transform".to_string()),
      default_label: Some("Summarize".to_string()),
      ports: vec![
        GraphPortTemplate {
          id: "ranked".to_string(),
          label: "Ranked".to_string(),
          description: None,
          direction: PortDirection::Input,
          data_type: Some("ranked_items".to_string()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "briefs".to_string(),
          label: "Briefs".to_string(),
          description: None,
          direction: PortDirection::Output,
          data_type: Some("briefs".to_string()),
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
            GraphFieldOption { value: "bullet".to_string(), label: "Bullet".to_string() },
            GraphFieldOption { value: "briefing".to_string(), label: "Briefing".to_string() },
          ],
        },
        required: true,
        default_value: Some(json!("briefing")),
      }],
    },
    GraphNodeTemplate {
      id: "feed_output".to_string(),
      label: "Feed Output".to_string(),
      description: Some("Publishes the ranked briefs into the reader feed".to_string()),
      category: Some("output".to_string()),
      default_label: Some("Feed".to_string()),
      ports: vec![GraphPortTemplate {
        id: "briefs".to_string(),
        label: "Briefs".to_string(),
        description: None,
        direction: PortDirection::Input,
        data_type: Some("briefs".to_string()),
        required: true,
        allow_multiple: false,
      }],
      fields: vec![
        GraphFieldSchema {
          id: "channel".to_string(),
          label: "Channel".to_string(),
          description: None,
          kind: GraphFieldKind::Text,
          required: true,
          default_value: Some(json!("daily-intel")),
        },
        GraphFieldSchema {
          id: "max_items".to_string(),
          label: "Max Items".to_string(),
          description: None,
          kind: GraphFieldKind::Integer,
          required: true,
          default_value: Some(json!(12)),
        },
      ],
    },
  ]
}

pub fn sample_document(flow_id: &str) -> GraphDocument {
  let templates = sample_templates();
  let mut document = GraphDocument::new(flow_id.to_string(), "Newsfeed Research Flow");
  document.viewport = GraphViewport { pan_x: 96.0, pan_y: 56.0, zoom: 0.9 };
  document.selection = GraphSelection::default();
  document.nodes = vec![
    build_node(
      &templates,
      "topics",
      "topic_input",
      GraphPoint { x: 40.0, y: 80.0 },
      None,
      &[("topics", json!(["AI policy", "open-source models", "semiconductor fabs"]))],
    ),
    build_node(
      &templates,
      "sources",
      "curated_sources",
      GraphPoint { x: 40.0, y: 280.0 },
      None,
      &[("domains", json!(["www.ft.com", "www.theinformation.com", "www.semafor.com"]))],
    ),
    build_node(&templates, "fetch", "web_fetch", GraphPoint { x: 320.0, y: 180.0 }, None, &[("per_source_limit", json!(6))]),
    build_node(&templates, "extract", "extract_signals", GraphPoint { x: 620.0, y: 180.0 }, None, &[]),
    build_node(&templates, "score", "dedupe_rank", GraphPoint { x: 920.0, y: 180.0 }, None, &[]),
    build_node(&templates, "summarize", "summarize_briefs", GraphPoint { x: 1220.0, y: 180.0 }, None, &[("style", json!("briefing"))]),
    build_node(&templates, "feed", "feed_output", GraphPoint { x: 1520.0, y: 180.0 }, None, &[("channel", json!("daily-intel"))]),
  ];
  document.edges = vec![
    GraphEdge {
      id: "edge-topics-fetch".to_string(),
      label: Some("topic scan".to_string()),
      metadata: Default::default(),
      from: GraphPortRef { node_id: "topics".to_string(), port_id: "topics".to_string() },
      to: GraphPortRef { node_id: "fetch".to_string(), port_id: "topics".to_string() },
    },
    GraphEdge {
      id: "edge-sources-fetch".to_string(),
      label: Some("source list".to_string()),
      metadata: Default::default(),
      from: GraphPortRef { node_id: "sources".to_string(), port_id: "sources".to_string() },
      to: GraphPortRef { node_id: "fetch".to_string(), port_id: "sources".to_string() },
    },
    GraphEdge {
      id: "edge-fetch-extract".to_string(),
      label: Some("article stream".to_string()),
      metadata: Default::default(),
      from: GraphPortRef { node_id: "fetch".to_string(), port_id: "articles".to_string() },
      to: GraphPortRef { node_id: "extract".to_string(), port_id: "articles".to_string() },
    },
    GraphEdge {
      id: "edge-extract-score".to_string(),
      label: Some("signal candidates".to_string()),
      metadata: Default::default(),
      from: GraphPortRef { node_id: "extract".to_string(), port_id: "signals".to_string() },
      to: GraphPortRef { node_id: "score".to_string(), port_id: "signals".to_string() },
    },
    GraphEdge {
      id: "edge-score-summarize".to_string(),
      label: Some("ranked feed".to_string()),
      metadata: Default::default(),
      from: GraphPortRef { node_id: "score".to_string(), port_id: "ranked".to_string() },
      to: GraphPortRef { node_id: "summarize".to_string(), port_id: "ranked".to_string() },
    },
    GraphEdge {
      id: "edge-summarize-feed".to_string(),
      label: Some("brief output".to_string()),
      metadata: Default::default(),
      from: GraphPortRef { node_id: "summarize".to_string(), port_id: "briefs".to_string() },
      to: GraphPortRef { node_id: "feed".to_string(), port_id: "briefs".to_string() },
    },
  ];
  document
}

fn build_node(
  templates: &[GraphNodeTemplate],
  id: &str,
  template_id: &str,
  position: GraphPoint,
  label: Option<&str>,
  overrides: &[(&str, Value)],
) -> GraphNode {
  let template = node_template(templates, template_id).expect("sample template should exist");
  let mut properties = template.default_properties();
  if let Some(object) = properties.as_object_mut() {
    for (key, value) in overrides {
      object.insert((*key).to_string(), value.clone());
    }
  }

  GraphNode {
    id: id.to_string(),
    template_id: template_id.to_string(),
    label: label.map(str::to_string).or_else(|| template.default_label.clone()),
    metadata: Default::default(),
    position,
    properties,
  }
}
