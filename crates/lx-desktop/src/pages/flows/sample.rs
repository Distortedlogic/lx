use lx_graph_editor::model::GraphDocument;

use super::mermaid::{chart_graph_document, parse_chart};

pub const DEFAULT_FLOW_ID: &str = "newsfeed-research";
pub const DEFAULT_LX_FLOW_ID: &str = "lx-research-brief";
pub const DEFAULT_MERMAID_FLOW_ID: &str = "mermaid-mock-lx";

const NEWSFEED_SAMPLE_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/flows/newsfeed.json"));
const LX_SAMPLE_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/flows/lx-research-brief.json"));
const MERMAID_SAMPLE_MMD: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/flows/mermaid-mock-lx.mmd"));

pub fn is_lx_flow_id(flow_id: &str) -> bool {
  flow_id == DEFAULT_LX_FLOW_ID || flow_id.starts_with("lx-")
}

pub fn is_mermaid_flow_id(flow_id: &str) -> bool {
  flow_id == DEFAULT_MERMAID_FLOW_ID || flow_id.starts_with("mermaid-")
}

pub fn sample_document(flow_id: &str) -> GraphDocument {
  let mut document: GraphDocument = if is_lx_flow_id(flow_id) {
    serde_json::from_str(LX_SAMPLE_JSON).expect("lx sample should deserialize")
  } else if is_mermaid_flow_id(flow_id) {
    let chart = parse_chart(flow_id, MERMAID_SAMPLE_MMD).chart.expect("mermaid sample should parse");
    chart_graph_document(flow_id, &chart)
  } else {
    serde_json::from_str(NEWSFEED_SAMPLE_JSON).expect("newsfeed sample should deserialize")
  };
  document.id = flow_id.to_string();
  document
}
