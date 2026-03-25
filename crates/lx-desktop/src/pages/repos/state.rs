use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnalysisMode {
  Syntactic,
  Semantic,
  Hybrid,
}

impl std::fmt::Display for AnalysisMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Syntactic => write!(f, "SYNTACTIC"),
      Self::Semantic => write!(f, "SEMANTIC"),
      Self::Hybrid => write!(f, "HYBRID"),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChunkResult {
  pub id: String,
  pub score: f64,
  pub description: String,
  pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisResults {
  pub chunks: Vec<ChunkResult>,
  pub total_tokens: usize,
  pub latency_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode {
  pub name: String,
  pub path: String,
  pub is_dir: bool,
  pub depth: u8,
}

#[derive(Clone, Copy)]
pub struct ReposState {
  pub root_path: Signal<String>,
  pub selected_file: Signal<Option<String>>,
  pub mode: Signal<AnalysisMode>,
  pub tree_depth: Signal<f64>,
  pub results: Signal<Option<AnalysisResults>>,
  pub analyzing: Signal<bool>,
}

impl ReposState {
  pub fn provide() -> Self {
    let cwd = std::env::current_dir().ok().map(|p| p.display().to_string()).unwrap_or_else(|| ".".into());
    let ctx = Self {
      root_path: Signal::new(cwd),
      selected_file: Signal::new(None),
      mode: Signal::new(AnalysisMode::Syntactic),
      tree_depth: Signal::new(3.0),
      results: Signal::new(None),
      analyzing: Signal::new(false),
    };
    use_context_provider(|| ctx);
    ctx
  }
}

pub async fn read_dir_tree(root: &str, max_depth: u8) -> Vec<TreeNode> {
  let mut nodes = Vec::new();
  let mut stack: Vec<(String, u8)> = vec![(root.to_string(), 0)];
  while let Some((path, depth)) = stack.pop() {
    if depth > max_depth {
      continue;
    }
    let Ok(mut entries) = tokio::fs::read_dir(&path).await else { continue };
    let mut children = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
      let name = entry.file_name().to_string_lossy().to_string();
      if name.starts_with('.') {
        continue;
      }
      let full_path = entry.path().display().to_string();
      let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
      children.push((name, full_path, is_dir));
    }
    children.sort_by(|a, b| match (a.2, b.2) {
      (true, false) => std::cmp::Ordering::Less,
      (false, true) => std::cmp::Ordering::Greater,
      _ => a.0.cmp(&b.0),
    });
    for (name, full_path, is_dir) in children {
      nodes.push(TreeNode { name, path: full_path.clone(), is_dir, depth });
      if is_dir {
        stack.push((full_path, depth + 1));
      }
    }
  }
  nodes
}

pub async fn run_analysis(root: &str) -> AnalysisResults {
  let start = std::time::Instant::now();
  let mut file_count = 0usize;
  let mut total_bytes = 0usize;
  let mut ext_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
  let mut stack = vec![root.to_string()];
  while let Some(path) = stack.pop() {
    let Ok(mut entries) = tokio::fs::read_dir(&path).await else { continue };
    while let Ok(Some(entry)) = entries.next_entry().await {
      let name = entry.file_name().to_string_lossy().to_string();
      if name.starts_with('.') {
        continue;
      }
      let is_dir = entry.metadata().await.map(|m| m.is_dir()).unwrap_or(false);
      if is_dir {
        stack.push(entry.path().display().to_string());
      } else {
        file_count += 1;
        let size = entry.metadata().await.map(|m| m.len() as usize).unwrap_or(0);
        total_bytes += size;
        let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("other").to_string();
        *ext_counts.entry(ext).or_default() += 1;
      }
    }
  }
  let latency = start.elapsed().as_millis() as u64;
  let mut chunks: Vec<ChunkResult> = ext_counts
    .iter()
    .enumerate()
    .map(|(i, (ext, count))| {
      let score = (*count as f64) / (file_count.max(1) as f64);
      ChunkResult { id: format!("#CHUNK_{i:04}"), score, description: format!("{count} .{ext} files found in repository"), tags: vec![ext.to_uppercase()] }
    })
    .collect();
  chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
  let token_estimate = total_bytes / 4;
  AnalysisResults { chunks, total_tokens: token_estimate, latency_ms: latency }
}
