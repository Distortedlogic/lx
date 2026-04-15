use crate::graph_editor::model::GraphDocument;

pub const DEFAULT_FLOW_ID: &str = "newsfeed-research";

const NEWSFEED_SAMPLE_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/flows/newsfeed.json"));

pub fn sample_document(flow_id: &str) -> GraphDocument {
  let mut document: GraphDocument = serde_json::from_str(NEWSFEED_SAMPLE_JSON).expect("newsfeed sample should deserialize");
  document.id = flow_id.to_string();
  document
}
