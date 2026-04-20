use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::any;
use chrono::Utc;
use cron::Schedule;
use lx_graph_editor::model::{GraphDocument, GraphNode};
use tokio::sync::RwLock;

use crate::credentials::CredentialStore;

use super::executor::execute_flow;
use super::persistence::FlowRunPersistence;
use super::runner::NodeRunnerRegistry;

#[derive(Clone, Debug)]
pub struct FlowActivation {
  pub active: bool,
  pub document: GraphDocument,
  pub templates: Vec<lx_graph_editor::catalog::GraphNodeTemplate>,
}

#[derive(Clone)]
pub struct FlowScheduler {
  pub(crate) inner: Arc<SchedulerInner>,
}

pub(crate) struct SchedulerInner {
  pub flows: RwLock<HashMap<String, FlowActivation>>,
  registry: NodeRunnerRegistry,
  credentials: CredentialStore,
  runs: FlowRunPersistence,
}

impl FlowScheduler {
  pub fn new(registry: NodeRunnerRegistry, credentials: CredentialStore, runs: FlowRunPersistence) -> Self {
    Self { inner: Arc::new(SchedulerInner { flows: RwLock::new(HashMap::new()), registry, credentials, runs }) }
  }

  pub async fn register_flow(&self, flow_id: String, activation: FlowActivation) {
    self.inner.flows.write().await.insert(flow_id, activation);
  }

  pub async fn set_active(&self, flow_id: &str, active: bool) {
    if let Some(entry) = self.inner.flows.write().await.get_mut(flow_id) {
      entry.active = active;
    }
  }

  pub async fn remove(&self, flow_id: &str) {
    self.inner.flows.write().await.remove(flow_id);
  }

  pub async fn active_flow_ids(&self) -> Vec<String> {
    self.inner.flows.read().await.iter().filter(|(_, activation)| activation.active).map(|(id, _)| id.clone()).collect()
  }

  pub fn spawn_cron_loop(&self) {
    let inner = self.inner.clone();
    tokio::spawn(async move {
      let mut ticker = tokio::time::interval(Duration::from_secs(1));
      let mut last_fires: HashMap<(String, String), i64> = HashMap::new();
      loop {
        ticker.tick().await;
        let now = Utc::now();
        let flows = inner.flows.read().await;
        let mut fires_now: Vec<(String, String, GraphDocument, Vec<lx_graph_editor::catalog::GraphNodeTemplate>, chrono::DateTime<Utc>)> = Vec::new();
        for (flow_id, activation) in flows.iter() {
          if !activation.active {
            continue;
          }
          for (node_id, expr) in cron_triggers(&activation.document) {
            let key = (flow_id.clone(), node_id.clone());
            let Ok(schedule) = Schedule::from_str(&expr) else {
              continue;
            };
            let last_anchor = last_fires.get(&key).and_then(|ts| chrono::DateTime::from_timestamp(*ts, 0)).unwrap_or(now - chrono::Duration::seconds(60));
            let Some(next_fire) = schedule.after(&last_anchor).next() else {
              continue;
            };
            if next_fire <= now {
              fires_now.push((flow_id.clone(), node_id, activation.document.clone(), activation.templates.clone(), next_fire));
            }
          }
        }
        drop(flows);

        for (flow_id, node_id, document, templates, fire_time) in fires_now {
          last_fires.insert((flow_id.clone(), node_id.clone()), fire_time.timestamp());
          let registry = inner.registry.clone();
          let credentials = inner.credentials.clone();
          let runs = inner.runs.clone();
          tokio::spawn(async move {
            let (report, context) = execute_flow(&document, &templates, &registry, &credentials).await;
            let run_id = uuid::Uuid::new_v4().to_string();
            let _ = runs.save(&flow_id, &run_id, &report, &context);
          });
        }
      }
    });
  }

  pub fn spawn_webhook_listener(&self, bind_addr: String) {
    let inner = self.inner.clone();
    tokio::spawn(async move {
      let router = Router::new()
        .route("/webhook/{flow_id}/{*path}", any(webhook_handler))
        .route("/webhook/{flow_id}", any(webhook_handler_root))
        .with_state(inner.clone());
      let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(error) => {
          eprintln!("nodeflow webhook listener: failed to bind `{bind_addr}`: {error}");
          return;
        },
      };
      if let Err(error) = axum::serve(listener, router).await {
        eprintln!("nodeflow webhook listener exited: {error}");
      }
    });
  }
}

pub fn build_and_start_scheduler(
  credentials: CredentialStore,
  runs: FlowRunPersistence,
  flow_persistence: &crate::pages::flows::storage::FlowPersistence,
) -> FlowScheduler {
  let mut registry = super::default_registry();
  super::subflow::register_sub_workflow_runner(&mut registry, flow_persistence.clone(), credentials.clone());
  let scheduler = FlowScheduler::new(registry, credentials, runs);
  scheduler.spawn_cron_loop();
  scheduler.spawn_webhook_listener("127.0.0.1:5678".to_string());
  scheduler.restore_activations_sync(flow_persistence);
  scheduler
}

impl FlowScheduler {
  pub fn restore_activations_sync(&self, flow_persistence: &crate::pages::flows::storage::FlowPersistence) {
    let state = super::activation::load_activation_state();
    let templates = crate::pages::flows::registry::sample_workflow_registry().templates();
    let entries: Vec<(String, bool)> = state.flows.into_iter().collect();
    for (flow_id, active) in entries {
      if !active {
        continue;
      }
      let Ok(document) = flow_persistence.load_or_seed(&flow_id) else {
        continue;
      };
      let inner = self.inner.clone();
      let templates = templates.clone();
      tokio::spawn(async move {
        inner.flows.write().await.insert(flow_id, FlowActivation { active: true, document, templates });
      });
    }
  }
}

async fn webhook_handler(
  Path((flow_id, path)): Path<(String, String)>,
  State(inner): State<Arc<SchedulerInner>>,
  request_body: axum::body::Bytes,
) -> impl IntoResponse {
  fire_webhook(inner, flow_id, Some(path), request_body).await
}

async fn webhook_handler_root(Path(flow_id): Path<String>, State(inner): State<Arc<SchedulerInner>>, request_body: axum::body::Bytes) -> impl IntoResponse {
  fire_webhook(inner, flow_id, None, request_body).await
}

async fn fire_webhook(inner: Arc<SchedulerInner>, flow_id: String, path: Option<String>, body: axum::body::Bytes) -> axum::response::Response {
  let entry = {
    let flows = inner.flows.read().await;
    flows.get(&flow_id).cloned()
  };
  let Some(activation) = entry else {
    return (StatusCode::NOT_FOUND, "flow not registered").into_response();
  };
  if !activation.active {
    return (StatusCode::FORBIDDEN, "flow not active").into_response();
  }
  let webhook_nodes = webhook_triggers(&activation.document, path.as_deref());
  if webhook_nodes.is_empty() {
    return (StatusCode::NOT_FOUND, "no matching webhook trigger").into_response();
  }
  let registry = inner.registry.clone();
  let credentials = inner.credentials.clone();
  let runs = inner.runs.clone();
  let document = activation.document.clone();
  let templates = activation.templates.clone();
  tokio::spawn(async move {
    let _body_text = String::from_utf8_lossy(&body).to_string();
    let (report, context) = execute_flow(&document, &templates, &registry, &credentials).await;
    let run_id = uuid::Uuid::new_v4().to_string();
    let _ = runs.save(&flow_id, &run_id, &report, &context);
  });
  (StatusCode::ACCEPTED, "triggered").into_response()
}

fn cron_triggers(document: &GraphDocument) -> Vec<(String, String)> {
  document
    .nodes
    .iter()
    .filter(|node| node.template_id == "trigger_cron")
    .filter_map(|node| node.properties.get("cron_expression").and_then(|value| value.as_str()).map(|expr| (node.id.clone(), expr.to_string())))
    .collect()
}

fn webhook_triggers(document: &GraphDocument, path: Option<&str>) -> Vec<GraphNode> {
  document
    .nodes
    .iter()
    .filter(|node| node.template_id == "trigger_webhook")
    .filter(|node| {
      let configured = node.properties.get("path").and_then(|value| value.as_str()).unwrap_or("");
      configured == path.unwrap_or("")
    })
    .cloned()
    .collect()
}
