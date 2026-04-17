use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use dioxus::prelude::*;

use lx_graph_editor::model::GraphDocument;

use super::mermaid::{chart_from_graph_document, chart_graph_document, emit_chart, parse_chart};
use super::sample::{DEFAULT_FLOW_ID, DEFAULT_MERMAID_FLOW_ID, sample_document};

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
    if flow_id == DEFAULT_FLOW_ID || flow_id == DEFAULT_MERMAID_FLOW_ID {
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

  fn json_path(&self, flow_id: &str) -> PathBuf {
    self.root.join(format!("{}.json", sanitize_flow_id(flow_id)))
  }

  fn mermaid_path(&self, flow_id: &str) -> PathBuf {
    self.root.join(format!("{}.mmd", sanitize_flow_id(flow_id)))
  }

  fn flow_path(&self, document: &GraphDocument) -> PathBuf {
    if is_mermaid_document(document) { self.mermaid_path(&document.id) } else { self.json_path(&document.id) }
  }
}

impl FlowRepository for FileFlowRepository {
  fn load(&self, flow_id: &str) -> Result<Option<GraphDocument>> {
    let mermaid_path = self.mermaid_path(flow_id);
    if mermaid_path.exists() {
      let source = fs::read_to_string(&mermaid_path).with_context(|| format!("failed to read `{}`", mermaid_path.display()))?;
      let parsed = parse_chart(flow_id, &source);
      let chart = parsed
        .chart
        .ok_or_else(|| anyhow::anyhow!(parsed.diagnostics.into_iter().map(|diagnostic| diagnostic.message).collect::<Vec<_>>().join("; ")))
        .with_context(|| format!("failed to parse `{}`", mermaid_path.display()))?;
      return Ok(Some(chart_graph_document(flow_id, &chart)));
    }

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
    let path = self.flow_path(document);
    if is_mermaid_document(document) {
      let payload = emit_chart(&chart_from_graph_document(document));
      fs::write(&path, payload).with_context(|| format!("failed to write `{}`", path.display()))?;
      let json_path = self.json_path(&document.id);
      if json_path.exists() {
        let _ = fs::remove_file(json_path);
      }
    } else {
      let payload = serde_json::to_string_pretty(document).context("failed to serialize flow document")?;
      fs::write(&path, payload).with_context(|| format!("failed to write `{}`", path.display()))?;
      let mermaid_path = self.mermaid_path(&document.id);
      if mermaid_path.exists() {
        let _ = fs::remove_file(mermaid_path);
      }
    }
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
          if extension != "json" && extension != "mmd" {
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
    while self.json_path(&candidate).exists() || self.mermaid_path(&candidate).exists() {
      candidate = format!("{base}-copy-{index}");
      index += 1;
    }
    Ok(candidate)
  }
}

fn is_mermaid_document(document: &GraphDocument) -> bool {
  document.metadata.tags.iter().any(|tag| tag.eq_ignore_ascii_case("mermaid"))
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::pages::flows::sample::sample_document;

  #[test]
  fn mermaid_documents_save_and_load_as_mmd() {
    let root = std::path::PathBuf::from("/tmp").join(format!("lx-mermaid-storage-{}", uuid::Uuid::new_v4()));
    let repository = FileFlowRepository::new(root.clone());
    let document = sample_document(DEFAULT_MERMAID_FLOW_ID);

    repository.save(&document).expect("mermaid document should save");
    let loaded = repository.load(DEFAULT_MERMAID_FLOW_ID).expect("mermaid document should load").expect("loaded document");

    assert!(root.join(format!("{DEFAULT_MERMAID_FLOW_ID}.mmd")).exists());
    assert!(loaded.metadata.tags.iter().any(|tag| tag == "mermaid"));
    assert_eq!(loaded.edges.len(), document.edges.len());

    let _ = fs::remove_dir_all(root);
  }
}
