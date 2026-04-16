use lx_graph_editor::model::GraphDocument;

pub const DEFAULT_FLOW_ID: &str = "newsfeed-research";
pub const DEFAULT_LX_FLOW_ID: &str = "lx-research-brief";

const NEWSFEED_SAMPLE_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/flows/newsfeed.json"));
const LX_SAMPLE_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/flows/lx-research-brief.json"));

pub fn is_lx_flow_id(flow_id: &str) -> bool {
  flow_id == DEFAULT_LX_FLOW_ID || flow_id.starts_with("lx-")
}

pub fn sample_document(flow_id: &str) -> GraphDocument {
  let mut document: GraphDocument = if is_lx_flow_id(flow_id) {
    serde_json::from_str(LX_SAMPLE_JSON).expect("lx sample should deserialize")
  } else {
    serde_json::from_str(NEWSFEED_SAMPLE_JSON).expect("newsfeed sample should deserialize")
  };
  document.id = flow_id.to_string();
  document
}
