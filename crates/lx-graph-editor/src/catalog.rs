use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
  Input,
  Output,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPortType {
  pub namespace: String,
  pub name: String,
  #[serde(default)]
  pub qualifiers: Vec<String>,
}

impl GraphPortType {
  pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
    Self { namespace: namespace.into(), name: name.into(), qualifiers: Vec::new() }
  }

  pub fn qualified(namespace: impl Into<String>, name: impl Into<String>, qualifiers: impl IntoIterator<Item = impl Into<String>>) -> Self {
    Self { namespace: namespace.into(), name: name.into(), qualifiers: qualifiers.into_iter().map(Into::into).collect() }
  }

  pub fn workflow(name: impl Into<String>) -> Self {
    Self::new("workflow", name)
  }

  pub fn lx(name: impl Into<String>) -> Self {
    Self::new("lx", name)
  }

  pub fn accepts(&self, provided: &Self) -> bool {
    self.namespace == provided.namespace
      && self.name == provided.name
      && self.qualifiers.iter().all(|required| provided.qualifiers.iter().any(|candidate| candidate == required))
  }
}

impl std::fmt::Display for GraphPortType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.qualifiers.is_empty() {
      return write!(f, "{}:{}", self.namespace, self.name);
    }
    write!(f, "{}:{}[{}]", self.namespace, self.name, self.qualifiers.join(", "))
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphFieldOption {
  pub value: String,
  pub label: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphFieldValueMode {
  #[default]
  Literal,
  Expression,
  Credential,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphExpressionSupport {
  pub language: Option<String>,
  pub placeholder: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphCredentialRequirement {
  pub namespace: String,
  pub kind: String,
  pub label: String,
  #[serde(default)]
  pub allow_key_selection: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphCredentialOption {
  pub id: String,
  pub namespace: String,
  pub kind: String,
  pub label: String,
  pub detail: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphFieldCapabilities {
  pub expression: Option<GraphExpressionSupport>,
  pub credential: Option<GraphCredentialRequirement>,
}

impl GraphFieldCapabilities {
  pub fn supports_expressions(&self) -> bool {
    self.expression.is_some()
  }

  pub fn supports_credentials(&self) -> bool {
    self.credential.is_some()
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum GraphBoundFieldValue {
  Literal { value: Value },
  Expression { expression: String },
  Credential { credential_id: String, key: Option<String> },
}

impl GraphBoundFieldValue {
  pub fn mode(&self) -> GraphFieldValueMode {
    match self {
      Self::Literal { .. } => GraphFieldValueMode::Literal,
      Self::Expression { .. } => GraphFieldValueMode::Expression,
      Self::Credential { .. } => GraphFieldValueMode::Credential,
    }
  }
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
  pub capabilities: GraphFieldCapabilities,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPortTemplate {
  pub id: String,
  pub label: String,
  pub description: Option<String>,
  pub direction: PortDirection,
  pub data_type: Option<GraphPortType>,
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

pub fn bound_field_value(value: &Value) -> Option<GraphBoundFieldValue> {
  serde_json::from_value::<GraphBoundFieldValue>(value.clone()).ok()
}

pub fn unwrap_literal_field_value(value: &Value) -> Value {
  match bound_field_value(value) {
    Some(GraphBoundFieldValue::Literal { value }) => value,
    _ => value.clone(),
  }
}
