use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
  Input,
  Output,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphFieldOption {
  pub value: String,
  pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GraphFieldKind {
  Text,
  TextArea,
  Number,
  Integer,
  Boolean,
  Select { options: Vec<GraphFieldOption> },
  StringList,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphFieldSchema {
  pub id: String,
  pub label: String,
  pub description: Option<String>,
  pub kind: GraphFieldKind,
  pub required: bool,
  pub default_value: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPortTemplate {
  pub id: String,
  pub label: String,
  pub description: Option<String>,
  pub direction: PortDirection,
  pub data_type: Option<String>,
  pub required: bool,
  pub allow_multiple: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphNodeTemplate {
  pub id: String,
  pub label: String,
  pub description: Option<String>,
  pub category: Option<String>,
  pub default_label: Option<String>,
  #[serde(default)]
  pub ports: Vec<GraphPortTemplate>,
  #[serde(default)]
  pub fields: Vec<GraphFieldSchema>,
}

impl GraphNodeTemplate {
  pub fn default_properties(&self) -> Value {
    materialize_default_properties(self)
  }
}

pub fn node_template<'a>(templates: &'a [GraphNodeTemplate], template_id: &str) -> Option<&'a GraphNodeTemplate> {
  templates.iter().find(|template| template.id == template_id)
}

pub fn port_template<'a>(templates: &'a [GraphNodeTemplate], template_id: &str, port_id: &str) -> Option<&'a GraphPortTemplate> {
  let template = node_template(templates, template_id)?;
  template.ports.iter().find(|port| port.id == port_id)
}

pub fn field_schema<'a>(templates: &'a [GraphNodeTemplate], template_id: &str, field_id: &str) -> Option<&'a GraphFieldSchema> {
  let template = node_template(templates, template_id)?;
  template.fields.iter().find(|field| field.id == field_id)
}

pub fn materialize_default_properties(template: &GraphNodeTemplate) -> Value {
  let mut object = Map::new();
  for field in &template.fields {
    if let Some(default_value) = &field.default_value {
      object.insert(field.id.clone(), default_value.clone());
    }
  }
  Value::Object(object)
}
