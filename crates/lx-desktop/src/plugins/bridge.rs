use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PluginBridgeError {
  pub code: String,
  pub message: String,
  pub details: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginDataResult<T: Clone + PartialEq + 'static> {
  pub data: Option<T>,
  pub loading: bool,
  pub error: Option<PluginBridgeError>,
}

impl<T: Clone + PartialEq + 'static> Default for PluginDataResult<T> {
  fn default() -> Self {
    Self { data: None, loading: true, error: None }
  }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PluginHostContext {
  pub company_id: Option<String>,
  pub company_prefix: Option<String>,
  pub project_id: Option<String>,
  pub entity_id: Option<String>,
  pub entity_type: Option<String>,
  pub user_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginBridgeContextValue {
  pub plugin_id: String,
  pub host_context: PluginHostContext,
}

pub fn use_plugin_bridge() -> Option<PluginBridgeContextValue> {
  use_context::<Signal<Option<PluginBridgeContextValue>>>().read().clone()
}

pub fn provide_plugin_bridge(value: PluginBridgeContextValue) {
  use_context_provider(|| Signal::new(Some(value)));
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginModalBoundsRequest {
  pub bounds: String,
  pub width: Option<u32>,
  pub height: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginRenderCloseEvent {
  pub reason: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginRenderEnvironmentContext {
  pub environment: Option<String>,
  pub launcher_id: Option<String>,
  pub bounds: Option<String>,
}
