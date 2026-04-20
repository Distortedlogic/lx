use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use dioxus::prelude::*;

use lx_graph_editor::model::GraphDocument;

use super::sample::{DEFAULT_FLOW_ID, sample_document};

pub trait FlowRepository: Send + Sync {
  fn load(&self, flow_id: &str) -> Result<Option<GraphDocument>>;
  fn save(&self, document: &GraphDocument) -> Result<()>;
  fn most_recent_flow_id(&self) -> Result<Option<String>>;
  fn next_flow_id(&self, base_flow_id: &str) -> Result<String>;
}

#[derive(Clone)]
pub struct FlowPersistence {
  repository: Arc<dyn FlowRepository>,
}

impl FlowPersistence {
  pub fn file_backed() -> Self {
    Self { repository: Arc::new(FileFlowRepository::new(default_flows_root())) }
  }

  pub fn resolve_default_flow_id(&self) -> String {
    self.repository.most_recent_flow_id().ok().flatten().unwrap_or_else(|| DEFAULT_FLOW_ID.to_string())
  }

  pub fn load_or_seed(&self, flow_id: &str) -> Result<GraphDocument> {
    if let Some(document) = self.repository.load(flow_id)? {
      return Ok(document);
    }

    let document = sample_document(flow_id);
    if flow_id == DEFAULT_FLOW_ID {
      self.repository.save(&document)?;
    }
    Ok(document)
  }

  pub fn save(&self, document: &GraphDocument) -> Result<()> {
    self.repository.save(document)?;
    self.write_snapshot(document).ok();
    Ok(())
  }

  fn write_snapshot(&self, document: &GraphDocument) -> Result<()> {
    let versions_root = default_versions_root().join(sanitize_flow_id(&document.id));
    fs::create_dir_all(&versions_root).with_context(|| format!("failed to create `{}`", versions_root.display()))?;
    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string();
    let path = versions_root.join(format!("{timestamp}.json"));
    let payload = serde_json::to_string_pretty(document).context("failed to serialize flow snapshot")?;
    fs::write(&path, payload).with_context(|| format!("failed to write `{}`", path.display()))?;
    enforce_version_retention(&versions_root, 20).ok();
    Ok(())
  }

  pub fn list_snapshots(&self, flow_id: &str) -> Result<Vec<String>> {
    let versions_root = default_versions_root().join(sanitize_flow_id(flow_id));
    if !versions_root.exists() {
      return Ok(Vec::new());
    }
    let mut entries: Vec<_> = fs::read_dir(&versions_root)
      .with_context(|| format!("failed to read `{}`", versions_root.display()))?
      .filter_map(std::result::Result::ok)
      .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
      .filter_map(|entry| entry.file_name().into_string().ok())
      .collect();
    entries.sort();
    entries.reverse();
    Ok(entries)
  }

  pub fn save_as_new(&self, document: &GraphDocument) -> Result<GraphDocument> {
    let mut next_document = document.clone();
    next_document.id = self.repository.next_flow_id(&document.id)?;
    self.repository.save(&next_document)?;
    Ok(next_document)
  }

  pub fn reset_to_sample(&self, flow_id: &str) -> Result<GraphDocument> {
    let document = sample_document(flow_id);
    self.repository.save(&document)?;
    Ok(document)
  }
}

pub fn provide_flow_persistence() -> FlowPersistence {
  let persistence = use_hook(FlowPersistence::file_backed);
  use_context_provider({
    let persistence = persistence.clone();
    move || persistence.clone()
  });
  persistence
}

pub fn use_flow_persistence() -> FlowPersistence {
  use_context()
}

struct FileFlowRepository {
  root: PathBuf,
}

impl FileFlowRepository {
  fn new(root: PathBuf) -> Self {
    Self { root }
  }

  fn json_path(&self, flow_id: &str) -> PathBuf {
    self.root.join(format!("{}.json", sanitize_flow_id(flow_id)))
  }
}

impl FlowRepository for FileFlowRepository {
  fn load(&self, flow_id: &str) -> Result<Option<GraphDocument>> {
    let json_path = self.json_path(flow_id);
    if !json_path.exists() {
      return Ok(None);
    }
    let payload = fs::read_to_string(&json_path).with_context(|| format!("failed to read `{}`", json_path.display()))?;
    let mut document: GraphDocument = serde_json::from_str(&payload).with_context(|| format!("failed to parse `{}`", json_path.display()))?;
    document.id = flow_id.to_string();
    Ok(Some(document))
  }

  fn save(&self, document: &GraphDocument) -> Result<()> {
    fs::create_dir_all(&self.root).with_context(|| format!("failed to create `{}`", self.root.display()))?;
    let path = self.json_path(&document.id);
    let payload = serde_json::to_string_pretty(document).context("failed to serialize flow document")?;
    fs::write(&path, payload).with_context(|| format!("failed to write `{}`", path.display()))?;
    Ok(())
  }

  fn most_recent_flow_id(&self) -> Result<Option<String>> {
    let entries = match fs::read_dir(&self.root) {
      Ok(entries) => entries,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
      Err(error) => return Err(error).with_context(|| format!("failed to read `{}`", self.root.display())),
    };

    Ok(
      entries
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
          let path = entry.path();
          let extension = path.extension()?.to_str()?;
          if extension != "json" {
            return None;
          }
          let metadata = entry.metadata().ok()?;
          let modified = metadata.modified().ok()?;
          let stem = path.file_stem()?.to_str()?.to_string();
          Some((modified, stem))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, flow_id)| flow_id),
    )
  }

  fn next_flow_id(&self, base_flow_id: &str) -> Result<String> {
    let base = sanitize_flow_id(base_flow_id);
    let mut candidate = format!("{base}-copy");
    let mut index = 2usize;
    while self.json_path(&candidate).exists() {
      candidate = format!("{base}-copy-{index}");
      index += 1;
    }
    Ok(candidate)
  }
}

fn default_flows_root() -> PathBuf {
  let base = dirs::data_local_dir().or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share"))).unwrap_or_else(|| PathBuf::from("/tmp"));
  base.join("nodeflow").join("flows")
}

fn default_versions_root() -> PathBuf {
  let base = dirs::data_local_dir().or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share"))).unwrap_or_else(|| PathBuf::from("/tmp"));
  base.join("nodeflow").join("versions")
}

fn enforce_version_retention(root: &PathBuf, retention: usize) -> Result<()> {
  let mut entries: Vec<_> = fs::read_dir(root)?
    .filter_map(std::result::Result::ok)
    .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
    .filter_map(|entry| entry.metadata().ok().and_then(|meta| meta.modified().ok()).map(|modified| (modified, entry.path())))
    .collect();
  entries.sort_by(|left, right| right.0.cmp(&left.0));
  for (_, path) in entries.into_iter().skip(retention) {
    let _ = fs::remove_file(path);
  }
  Ok(())
}

fn sanitize_flow_id(flow_id: &str) -> String {
  let sanitized: String = flow_id.chars().map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { ch } else { '-' }).collect();
  let trimmed = sanitized.trim_matches('-');
  if trimmed.is_empty() { "flow".to_string() } else { trimmed.to_string() }
}
