use serde_json::json;

use lx_graph_editor::catalog::{GraphFieldCapabilities, GraphFieldKind, GraphFieldSchema, GraphNodeTemplate, GraphPortTemplate, GraphPortType, PortDirection};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LxNodeKind {
  GoalInput,
  EvidenceIngest,
  SensemakingPass,
  DecisionRouter,
  AgentTask,
  ArtifactOutput,
}

pub fn lx_node_templates() -> Vec<GraphNodeTemplate> {
  vec![
    GraphNodeTemplate {
      id: LxNodeKind::GoalInput.template_id().to_string(),
      label: "Goal Input".to_string(),
      description: Some("Introduces an lx goal frame and success constraints into the graph.".to_string()),
      category: Some("lx input".to_string()),
      default_label: Some("Goal".to_string()),
      ports: vec![GraphPortTemplate {
        id: "goal".to_string(),
        label: "Goal".to_string(),
        description: Some("Structured goal context".to_string()),
        direction: PortDirection::Output,
        data_type: Some(goal_context_type()),
        required: true,
        allow_multiple: true,
      }],
      fields: vec![
        GraphFieldSchema {
          id: "mission".to_string(),
          label: "Mission".to_string(),
          description: Some("What this graph is trying to accomplish.".to_string()),
          kind: GraphFieldKind::TextArea,
          required: true,
          default_value: Some(json!("Track a domain, produce evidence-backed conclusions, and emit a reusable artifact.")),
          capabilities: GraphFieldCapabilities::default(),
        },
        GraphFieldSchema {
          id: "success_criteria".to_string(),
          label: "Success Criteria".to_string(),
          description: Some("What a good outcome must include.".to_string()),
          kind: GraphFieldKind::StringList,
          required: true,
          default_value: Some(json!(["Evidence-backed claims", "Clear next actions", "Reusable artifact"])),
          capabilities: GraphFieldCapabilities::default(),
        },
      ],
    },
    GraphNodeTemplate {
      id: LxNodeKind::EvidenceIngest.template_id().to_string(),
      label: "Evidence Ingest".to_string(),
      description: Some("Collects and normalizes source material into an evidence bundle.".to_string()),
      category: Some("lx ingest".to_string()),
      default_label: Some("Evidence".to_string()),
      ports: vec![
        GraphPortTemplate {
          id: "goal".to_string(),
          label: "Goal".to_string(),
          description: Some("Upstream goal frame".to_string()),
          direction: PortDirection::Input,
          data_type: Some(goal_context_type()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "evidence".to_string(),
          label: "Evidence".to_string(),
          description: Some("Normalized evidence bundle".to_string()),
          direction: PortDirection::Output,
          data_type: Some(evidence_bundle_type()),
          required: true,
          allow_multiple: true,
        },
      ],
      fields: vec![
        GraphFieldSchema {
          id: "sources".to_string(),
          label: "Sources".to_string(),
          description: Some("Feeds, files, or search scopes to ingest.".to_string()),
          kind: GraphFieldKind::StringList,
          required: true,
          default_value: Some(json!(["company filings", "press releases", "domain watchlist"])),
          capabilities: GraphFieldCapabilities::default(),
        },
        GraphFieldSchema {
          id: "max_items".to_string(),
          label: "Max Items".to_string(),
          description: Some("Maximum evidence items to normalize.".to_string()),
          kind: GraphFieldKind::Integer,
          required: true,
          default_value: Some(json!(24)),
          capabilities: GraphFieldCapabilities::default(),
        },
      ],
    },
    GraphNodeTemplate {
      id: LxNodeKind::SensemakingPass.template_id().to_string(),
      label: "Sensemaking Pass".to_string(),
      description: Some("Transforms evidence into claims and structured artifacts.".to_string()),
      category: Some("lx transform".to_string()),
      default_label: Some("Sensemake".to_string()),
      ports: vec![
        GraphPortTemplate {
          id: "goal".to_string(),
          label: "Goal".to_string(),
          description: Some("Goal frame".to_string()),
          direction: PortDirection::Input,
          data_type: Some(goal_context_type()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "evidence".to_string(),
          label: "Evidence".to_string(),
          description: Some("Evidence bundle".to_string()),
          direction: PortDirection::Input,
          data_type: Some(evidence_bundle_type()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "artifact".to_string(),
          label: "Artifact".to_string(),
          description: Some("Generalized lx artifact output".to_string()),
          direction: PortDirection::Output,
          data_type: Some(research_brief_type()),
          required: true,
          allow_multiple: true,
        },
      ],
      fields: vec![
        GraphFieldSchema {
          id: "analysis_frame".to_string(),
          label: "Analysis Frame".to_string(),
          description: Some("What lens to use when turning evidence into claims.".to_string()),
          kind: GraphFieldKind::TextArea,
          required: true,
          default_value: Some(json!("Surface concrete changes, implications, and unresolved uncertainty.")),
          capabilities: GraphFieldCapabilities::default(),
        },
        GraphFieldSchema {
          id: "include_citations".to_string(),
          label: "Include Citations".to_string(),
          description: Some("Whether each claim should carry evidence references.".to_string()),
          kind: GraphFieldKind::Boolean,
          required: true,
          default_value: Some(json!(true)),
          capabilities: GraphFieldCapabilities::default(),
        },
      ],
    },
    GraphNodeTemplate {
      id: LxNodeKind::DecisionRouter.template_id().to_string(),
      label: "Decision Router".to_string(),
      description: Some("Routes an artifact toward action, escalation, or archive.".to_string()),
      category: Some("lx control".to_string()),
      default_label: Some("Route".to_string()),
      ports: vec![
        GraphPortTemplate {
          id: "artifact".to_string(),
          label: "Artifact".to_string(),
          description: Some("Artifact to route".to_string()),
          direction: PortDirection::Input,
          data_type: Some(artifact_type()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "actionable".to_string(),
          label: "Actionable".to_string(),
          description: Some("Artifacts that should trigger execution.".to_string()),
          direction: PortDirection::Output,
          data_type: Some(action_packet_type()),
          required: true,
          allow_multiple: true,
        },
        GraphPortTemplate {
          id: "archive".to_string(),
          label: "Archive".to_string(),
          description: Some("Artifacts retained for later use.".to_string()),
          direction: PortDirection::Output,
          data_type: Some(artifact_type()),
          required: true,
          allow_multiple: true,
        },
      ],
      fields: vec![GraphFieldSchema {
        id: "routing_policy".to_string(),
        label: "Routing Policy".to_string(),
        description: Some("How the node decides whether to act or archive.".to_string()),
        kind: GraphFieldKind::TextArea,
        required: true,
        default_value: Some(json!("Route items with concrete owners and immediate next actions to execution; archive the rest.")),
        capabilities: GraphFieldCapabilities::default(),
      }],
    },
    GraphNodeTemplate {
      id: LxNodeKind::AgentTask.template_id().to_string(),
      label: "Agent Task".to_string(),
      description: Some("Turns an actionable packet into an executable lx task.".to_string()),
      category: Some("lx execution".to_string()),
      default_label: Some("Agent Task".to_string()),
      ports: vec![
        GraphPortTemplate {
          id: "actionable".to_string(),
          label: "Actionable".to_string(),
          description: Some("Packet ready for execution.".to_string()),
          direction: PortDirection::Input,
          data_type: Some(action_packet_type()),
          required: true,
          allow_multiple: false,
        },
        GraphPortTemplate {
          id: "artifact".to_string(),
          label: "Artifact".to_string(),
          description: Some("Execution artifact produced by the task.".to_string()),
          direction: PortDirection::Output,
          data_type: Some(execution_artifact_type()),
          required: true,
          allow_multiple: true,
        },
      ],
      fields: vec![
        GraphFieldSchema {
          id: "owner".to_string(),
          label: "Owner".to_string(),
          description: Some("Intended executor or agent role.".to_string()),
          kind: GraphFieldKind::Text,
          required: true,
          default_value: Some(json!("research-agent")),
          capabilities: GraphFieldCapabilities::default(),
        },
        GraphFieldSchema {
          id: "task_brief".to_string(),
          label: "Task Brief".to_string(),
          description: Some("Execution framing passed to the owning agent.".to_string()),
          kind: GraphFieldKind::TextArea,
          required: true,
          default_value: Some(json!("Produce a concise execution artifact with explicit next actions and evidence links.")),
          capabilities: GraphFieldCapabilities::default(),
        },
      ],
    },
    GraphNodeTemplate {
      id: LxNodeKind::ArtifactOutput.template_id().to_string(),
      label: "Artifact Output".to_string(),
      description: Some("Publishes a finalized lx artifact for downstream consumption.".to_string()),
      category: Some("lx output".to_string()),
      default_label: Some("Artifact".to_string()),
      ports: vec![GraphPortTemplate {
        id: "artifact".to_string(),
        label: "Artifact".to_string(),
        description: Some("Artifact to persist or emit.".to_string()),
        direction: PortDirection::Input,
        data_type: Some(artifact_type()),
        required: true,
        allow_multiple: false,
      }],
      fields: vec![GraphFieldSchema {
        id: "destination".to_string(),
        label: "Destination".to_string(),
        description: Some("Where the artifact should land.".to_string()),
        kind: GraphFieldKind::Text,
        required: true,
        default_value: Some(json!("project-artifacts")),
        capabilities: GraphFieldCapabilities::default(),
      }],
    },
  ]
}

impl LxNodeKind {
  pub fn template_id(self) -> &'static str {
    match self {
      LxNodeKind::GoalInput => "lx_goal_input",
      LxNodeKind::EvidenceIngest => "lx_evidence_ingest",
      LxNodeKind::SensemakingPass => "lx_sensemaking_pass",
      LxNodeKind::DecisionRouter => "lx_decision_router",
      LxNodeKind::AgentTask => "lx_agent_task",
      LxNodeKind::ArtifactOutput => "lx_artifact_output",
    }
  }
}

pub fn goal_context_type() -> GraphPortType {
  GraphPortType::qualified("lx", "context", ["goal"])
}

pub fn evidence_bundle_type() -> GraphPortType {
  GraphPortType::qualified("lx", "artifact", ["evidence_bundle"])
}

pub fn artifact_type() -> GraphPortType {
  GraphPortType::lx("artifact")
}

pub fn research_brief_type() -> GraphPortType {
  GraphPortType::qualified("lx", "artifact", ["research_brief"])
}

pub fn action_packet_type() -> GraphPortType {
  GraphPortType::qualified("lx", "artifact", ["action_packet"])
}

pub fn execution_artifact_type() -> GraphPortType {
  GraphPortType::qualified("lx", "artifact", ["execution_artifact"])
}

#[cfg(test)]
mod tests {
  use lx_graph_editor::catalog::{node_template, port_template};
  use lx_graph_editor::commands::{GraphCommand, apply_graph_command};
  use lx_graph_editor::model::{GraphDocument, GraphPoint, GraphPortRef};

  use super::{LxNodeKind, artifact_type, lx_node_templates};

  #[test]
  fn lx_registry_exposes_typed_artifact_outputs() {
    let templates = lx_node_templates();
    let sensemaking = node_template(&templates, LxNodeKind::SensemakingPass.template_id()).expect("sensemaking template");
    let artifact = port_template(&templates, &sensemaking.id, "artifact").expect("artifact port");

    assert_eq!(artifact.data_type, Some(super::research_brief_type()));
  }

  #[test]
  fn lx_artifact_output_accepts_specialized_artifacts() {
    let templates = lx_node_templates();
    let mut document = GraphDocument::new("lx-flow", "LX Flow");

    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode {
        node_id: "goal".to_string(),
        template_id: LxNodeKind::GoalInput.template_id().to_string(),
        position: GraphPoint { x: 0.0, y: 0.0 },
        label: None,
      },
    )
    .expect("add goal");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode {
        node_id: "evidence".to_string(),
        template_id: LxNodeKind::EvidenceIngest.template_id().to_string(),
        position: GraphPoint { x: 240.0, y: 0.0 },
        label: None,
      },
    )
    .expect("add evidence");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode {
        node_id: "sense".to_string(),
        template_id: LxNodeKind::SensemakingPass.template_id().to_string(),
        position: GraphPoint { x: 480.0, y: 0.0 },
        label: None,
      },
    )
    .expect("add sensemaking");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode {
        node_id: "out".to_string(),
        template_id: LxNodeKind::ArtifactOutput.template_id().to_string(),
        position: GraphPoint { x: 720.0, y: 0.0 },
        label: None,
      },
    )
    .expect("add output");

    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::ConnectPorts {
        edge_id: "goal-evidence".to_string(),
        from: GraphPortRef { node_id: "goal".to_string(), port_id: "goal".to_string() },
        to: GraphPortRef { node_id: "evidence".to_string(), port_id: "goal".to_string() },
        label: None,
      },
    )
    .expect("connect goal to evidence");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::ConnectPorts {
        edge_id: "goal-sense".to_string(),
        from: GraphPortRef { node_id: "goal".to_string(), port_id: "goal".to_string() },
        to: GraphPortRef { node_id: "sense".to_string(), port_id: "goal".to_string() },
        label: None,
      },
    )
    .expect("connect goal to sensemaking");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::ConnectPorts {
        edge_id: "evidence-sense".to_string(),
        from: GraphPortRef { node_id: "evidence".to_string(), port_id: "evidence".to_string() },
        to: GraphPortRef { node_id: "sense".to_string(), port_id: "evidence".to_string() },
        label: None,
      },
    )
    .expect("connect evidence to sensemaking");
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::ConnectPorts {
        edge_id: "sense-out".to_string(),
        from: GraphPortRef { node_id: "sense".to_string(), port_id: "artifact".to_string() },
        to: GraphPortRef { node_id: "out".to_string(), port_id: "artifact".to_string() },
        label: None,
      },
    )
    .expect("specialized research brief should satisfy generic artifact input");

    let output = port_template(&templates, LxNodeKind::ArtifactOutput.template_id(), "artifact").expect("output artifact port");
    assert_eq!(output.data_type, Some(artifact_type()));
    assert_eq!(document.edges.len(), 4);
  }
}
