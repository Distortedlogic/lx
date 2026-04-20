use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::context::ExecutionContext;
use super::types::{FlowExecutionReport, NodeExecutionData};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRecord {
  pub run_id: String,
  pub flow_id: String,
  pub report: FlowExecutionReport,
  pub node_outputs: HashMap<String, HashMap<String, NodeExecutionData>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunSummary {
  pub run_id: String,
  pub flow_id: String,
  pub started_at: String,
  pub finished_at: Option<String>,
  pub aborted: bool,
  pub success_count: usize,
  pub failed_count: usize,
  pub skipped_count: usize,
}

#[derive(Clone, Debug)]
pub struct FlowRunPersistence {
  root: PathBuf,
  retention: usize,
}

impl FlowRunPersistence {
  pub fn file_backed() -> Self {
    Self { root: default_runs_root(), retention: 50 }
  }

  pub fn with_root(root: PathBuf) -> Self {
    Self { root, retention: 50 }
  }

  pub fn save(&self, flow_id: &str, run_id: &str, report: &FlowExecutionReport, context: &ExecutionContext) -> Result<PathBuf> {
    let flow_dir = self.root.join(sanitize(flow_id));
    fs::create_dir_all(&flow_dir).with_context(|| format!("failed to create `{}`", flow_dir.display()))?;
    let path = flow_dir.join(format!("{}.json", sanitize(run_id)));
    let record = RunRecord { run_id: run_id.to_string(), flow_id: flow_id.to_string(), report: report.clone(), node_outputs: context.snapshot() };
    let payload = serde_json::to_string_pretty(&record).context("failed to serialize run record")?;
    fs::write(&path, payload).with_context(|| format!("failed to write `{}`", path.display()))?;
    self.enforce_retention(&flow_dir).ok();
    Ok(path)
  }

  pub fn list(&self, flow_id: &str) -> Result<Vec<RunSummary>> {
    let flow_dir = self.root.join(sanitize(flow_id));
    if !flow_dir.exists() {
      return Ok(Vec::new());
    }
    let entries = fs::read_dir(&flow_dir).with_context(|| format!("failed to read `{}`", flow_dir.display()))?;
    let mut summaries: Vec<RunSummary> = entries
      .filter_map(std::result::Result::ok)
      .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
      .filter_map(|entry| fs::read_to_string(entry.path()).ok().and_then(|payload| serde_json::from_str::<RunRecord>(&payload).ok()))
      .map(|record| RunSummary {
        run_id: record.run_id.clone(),
        flow_id: record.flow_id.clone(),
        started_at: record.report.started_at.clone(),
        finished_at: record.report.finished_at.clone(),
        aborted: record.report.aborted,
        success_count: record.report.success_count(),
        failed_count: record.report.failed_count(),
        skipped_count: record.report.skipped_count(),
      })
      .collect();
    summaries.sort_by(|left, right| right.started_at.cmp(&left.started_at));
    Ok(summaries)
  }

  pub fn load(&self, flow_id: &str, run_id: &str) -> Result<Option<RunRecord>> {
    let path = self.root.join(sanitize(flow_id)).join(format!("{}.json", sanitize(run_id)));
    if !path.exists() {
      return Ok(None);
    }
    let payload = fs::read_to_string(&path).with_context(|| format!("failed to read `{}`", path.display()))?;
    serde_json::from_str(&payload).map(Some).with_context(|| format!("failed to parse `{}`", path.display()))
  }

  fn enforce_retention(&self, flow_dir: &PathBuf) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(flow_dir)?
      .filter_map(std::result::Result::ok)
      .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
      .filter_map(|entry| entry.metadata().ok().and_then(|meta| meta.modified().ok()).map(|modified| (modified, entry.path())))
      .collect();
    entries.sort_by(|left, right| right.0.cmp(&left.0));
    for (_, path) in entries.into_iter().skip(self.retention) {
      let _ = fs::remove_file(path);
    }
    Ok(())
  }
}

fn default_runs_root() -> PathBuf {
  let base = dirs::data_local_dir().or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share"))).unwrap_or_else(|| PathBuf::from("/tmp"));
  base.join("nodeflow").join("runs")
}

fn sanitize(value: &str) -> String {
  let sanitized: String = value.chars().map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { ch } else { '-' }).collect();
  let trimmed = sanitized.trim_matches('-').to_string();
  if trimmed.is_empty() { "run".to_string() } else { trimmed }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::engine::types::{FlowExecutionReport, NodeExecutionRecord, NodeExecutionStatus, NodeItem, now_ts};

  fn temp_root(name: &str) -> PathBuf {
    PathBuf::from("/tmp").join(format!("nodeflow-runs-{}-{}", name, uuid::Uuid::new_v4()))
  }

  #[test]
  fn save_and_list_roundtrip() {
    let root = temp_root("roundtrip");
    let persistence = FlowRunPersistence::with_root(root.clone());

    let report = FlowExecutionReport {
      records: vec![NodeExecutionRecord {
        node_id: "n".to_string(),
        template_id: "http_request".to_string(),
        status: NodeExecutionStatus::Success,
        logs: Vec::new(),
        error: None,
        started_at: now_ts(),
        finished_at: Some(now_ts()),
      }],
      aborted: false,
      error: None,
      started_at: now_ts(),
      finished_at: Some(now_ts()),
    };
    let mut context = ExecutionContext::default();
    let mut port_outputs = HashMap::new();
    port_outputs.insert("response".to_string(), vec![NodeItem::from_json(serde_json::json!({ "status": 200 }))]);
    context.set_node_outputs("n", port_outputs);

    persistence.save("my-flow", "run-1", &report, &context).unwrap();

    let summaries = persistence.list("my-flow").unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].success_count, 1);

    let record = persistence.load("my-flow", "run-1").unwrap().unwrap();
    assert_eq!(record.node_outputs.get("n").and_then(|ports| ports.get("response")).map(|items| items.len()), Some(1));

    let _ = fs::remove_dir_all(root);
  }
}
