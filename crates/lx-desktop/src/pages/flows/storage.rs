use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use dioxus::prelude::*;

use crate::graph_editor::model::GraphDocument;

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
    self.repository.save(document)
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

  fn flow_path(&self, flow_id: &str) -> PathBuf {
    self.root.join(format!("{}.json", sanitize_flow_id(flow_id)))
  }
}

impl FlowRepository for FileFlowRepository {
  fn load(&self, flow_id: &str) -> Result<Option<GraphDocument>> {
    let path = self.flow_path(flow_id);
    if !path.exists() {
      return Ok(None);
    }
    let payload = fs::read_to_string(&path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut document: GraphDocument = serde_json::from_str(&payload).with_context(|| format!("failed to parse `{}`", path.display()))?;
    document.id = flow_id.to_string();
    Ok(Some(document))
  }

  fn save(&self, document: &GraphDocument) -> Result<()> {
    fs::create_dir_all(&self.root).with_context(|| format!("failed to create `{}`", self.root.display()))?;
    let path = self.flow_path(&document.id);
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
    while self.flow_path(&candidate).exists() {
      candidate = format!("{base}-copy-{index}");
      index += 1;
    }
    Ok(candidate)
  }
}

fn default_flows_root() -> PathBuf {
  let base = dirs::data_local_dir().or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share"))).unwrap_or_else(|| PathBuf::from("/tmp"));
  base.join("lx").join("flows")
}

fn sanitize_flow_id(flow_id: &str) -> String {
  let sanitized: String = flow_id.chars().map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { ch } else { '-' }).collect();
  let trimmed = sanitized.trim_matches('-');
  if trimmed.is_empty() { "flow".to_string() } else { trimmed.to_string() }
}
