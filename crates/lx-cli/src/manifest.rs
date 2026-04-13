use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RootManifest {
  pub workspace: Option<WorkspaceSection>,
  pub package: Option<PackageSection>,
  pub test: Option<TestSection>,
  pub backends: Option<BackendsSection>,
  pub stream: Option<StreamSection>,
  pub dependencies: Option<HashMap<String, DepSpec>>,
  #[serde(rename = "deps")]
  pub deps_table: Option<DepsTable>,
  pub tools: Option<HashMap<String, ToolSpec>>,
}

impl RootManifest {
  pub fn validate(&self, path: &Path) -> Result<(), String> {
    if let Some(ref pkg) = self.package {
      match &pkg.version {
        None => {
          return Err(format!("{}: [package] requires a 'version' field", path.display()));
        },
        Some(v) if v.is_empty() => {
          return Err(format!("{}: [package].version must not be empty", path.display()));
        },
        _ => {},
      }
    }
    Ok(())
  }
}

#[derive(Deserialize)]
pub struct WorkspaceSection {
  pub members: Vec<String>,
}

#[derive(Deserialize)]
pub struct PackageSection {
  pub name: String,
  pub version: Option<String>,
  pub entry: Option<String>,
  pub description: Option<String>,
  pub authors: Option<Vec<String>>,
  pub license: Option<String>,
  pub lx: Option<String>,
}

#[derive(Deserialize)]
pub struct TestSection {
  pub dir: Option<String>,
  pub pattern: Option<String>,
  pub threshold: Option<f64>,
  pub runs: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum YieldBackend {
  StdinStdout,
}

#[derive(Deserialize)]
pub struct BackendsSection {
  #[serde(rename = "yield")]
  pub yield_backend: Option<YieldBackend>,
}

#[derive(Deserialize)]
pub struct StreamSection {
  pub command: String,
}

#[derive(Deserialize)]
pub struct DepsTable {
  pub dev: Option<HashMap<String, DepSpec>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DepSpec {
  Git { git: String, branch: Option<String>, tag: Option<String>, rev: Option<String> },
  Path { path: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolSpec {
  Lx { path: String },
  Mcp { command: String },
}

#[derive(Debug)]
pub enum WorkspaceRootLookupError {
  NotFound { start: PathBuf },
  Read { path: PathBuf, error: std::io::Error },
  Parse { path: PathBuf, error: toml::de::Error },
  NotWorkspace { path: PathBuf },
}

impl std::fmt::Display for WorkspaceRootLookupError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      WorkspaceRootLookupError::NotFound { start } => write!(f, "no workspace {} found starting from {}", lx_span::LX_MANIFEST, start.display()),
      WorkspaceRootLookupError::Read { path, error } => write!(f, "cannot read {}: {error}", path.display()),
      WorkspaceRootLookupError::Parse { path, error } => write!(f, "invalid {}: {error}", path.display()),
      WorkspaceRootLookupError::NotWorkspace { path } => write!(f, "{} exists but is not a workspace root", path.display()),
    }
  }
}

impl std::error::Error for WorkspaceRootLookupError {}

pub fn find_manifest_root(start: &Path) -> Option<PathBuf> {
  let mut dir = start.to_path_buf();
  loop {
    let candidate = dir.join(lx_span::LX_MANIFEST);
    if candidate.exists() {
      return Some(dir);
    }
    if !dir.pop() {
      return None;
    }
  }
}

pub fn load_manifest(root: &Path) -> Result<RootManifest, String> {
  let manifest_path = root.join(lx_span::LX_MANIFEST);
  let content = fs::read_to_string(&manifest_path).map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
  let manifest: RootManifest = toml::from_str(&content).map_err(|e| format!("invalid {}: {e}", manifest_path.display()))?;
  manifest.validate(&manifest_path)?;
  Ok(manifest)
}

pub fn deps_dir(root: &Path) -> PathBuf {
  root.join(".lx").join("deps")
}

pub fn load_nearest_manifest(start: &Path) -> Result<Option<(PathBuf, RootManifest)>, String> {
  let Some(root) = find_manifest_root(start) else {
    return Ok(None);
  };
  let manifest = load_manifest(&root)?;
  Ok(Some((root, manifest)))
}

pub fn load_dep_dirs_detailed(include_dev: bool, start: &Path) -> Result<HashMap<String, PathBuf>, String> {
  let Some((root, _manifest)) = load_nearest_manifest(start)? else {
    return Ok(HashMap::new());
  };
  let deps = deps_dir(&root);
  if !deps.exists() {
    return Ok(HashMap::new());
  }
  let dev_names: Vec<String> = if !include_dev {
    let marker = deps.join(".dev-deps");
    if marker.exists() {
      fs::read_to_string(&marker)
        .map_err(|e| format!("cannot read {}: {e}", marker.display()))?
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
    } else {
      Vec::new()
    }
  } else {
    Vec::new()
  };
  let entries = fs::read_dir(&deps).map_err(|e| format!("cannot read {}: {e}", deps.display()))?;
  let mut map = HashMap::new();
  for entry in entries {
    let entry = entry.map_err(|e| format!("cannot read entry in {}: {e}", deps.display()))?;
    let path = entry.path();
    if path.is_dir() {
      let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return Err(format!("non-UTF-8 dependency entry in {}", deps.display()));
      };
      if !include_dev && dev_names.iter().any(|d| d == name) {
        continue;
      }
      map.insert(name.to_string(), path);
    }
  }
  Ok(map)
}

pub fn load_dep_dirs_no_dev_detailed(start: &Path) -> Result<HashMap<String, PathBuf>, String> {
  load_dep_dirs_detailed(false, start)
}

pub struct Workspace {
  pub members: Vec<Member>,
}

impl Workspace {
  pub fn member_map(&self) -> HashMap<String, PathBuf> {
    self.members.iter().map(|m| (m.pkg.name.clone(), m.dir.clone())).collect()
  }
}

pub struct Member {
  pub pkg: PackageSection,
  pub dir: PathBuf,
  pub test_dir: String,
  pub test_pattern: String,
}

pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
  let mut dir = start.to_path_buf();
  loop {
    let candidate = dir.join(lx_span::LX_MANIFEST);
    if candidate.exists() {
      let Ok(content) = fs::read_to_string(&candidate) else {
        if !dir.pop() {
          return None;
        }
        continue;
      };
      let manifest: RootManifest = match toml::from_str(&content) {
        Ok(manifest) => manifest,
        Err(_) => {
          if !dir.pop() {
            return None;
          }
          continue;
        },
      };
      if manifest.workspace.is_some() {
        return Some(dir);
      }
    }
    if !dir.pop() {
      return None;
    }
  }
}

pub fn find_workspace_root_detailed(start: &Path) -> Result<PathBuf, WorkspaceRootLookupError> {
  let mut dir = start.to_path_buf();
  let mut first_non_workspace = None;
  loop {
    let candidate = dir.join(lx_span::LX_MANIFEST);
    if candidate.exists() {
      let content = fs::read_to_string(&candidate).map_err(|error| WorkspaceRootLookupError::Read { path: candidate.clone(), error })?;
      let manifest: RootManifest = toml::from_str(&content).map_err(|error| WorkspaceRootLookupError::Parse { path: candidate.clone(), error })?;
      if manifest.workspace.is_some() {
        return Ok(dir);
      }
      if first_non_workspace.is_none() {
        first_non_workspace = Some(candidate);
      }
    }
    if !dir.pop() {
      break;
    }
  }
  Err(match first_non_workspace {
    Some(path) => WorkspaceRootLookupError::NotWorkspace { path },
    None => WorkspaceRootLookupError::NotFound { start: start.to_path_buf() },
  })
}

pub fn load_workspace(root: &Path) -> Result<Workspace, String> {
  let manifest_path = root.join(lx_span::LX_MANIFEST);
  let content = fs::read_to_string(&manifest_path).map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
  let manifest: RootManifest = toml::from_str(&content).map_err(|e| format!("invalid {}: {e}", manifest_path.display()))?;
  let ws = manifest.workspace.ok_or_else(|| format!("{} has no [workspace] section", manifest_path.display()))?;

  let mut members = Vec::new();
  for member_path in &ws.members {
    let member_dir = root.join(member_path);
    let member_manifest_path = member_dir.join(lx_span::LX_MANIFEST);
    if !member_manifest_path.exists() {
      return Err(format!("member '{}' has no lx.toml at {}", member_path, member_manifest_path.display()));
    }
    let member_content = fs::read_to_string(&member_manifest_path).map_err(|e| format!("cannot read {}: {e}", member_manifest_path.display()))?;
    let member_manifest: RootManifest = toml::from_str(&member_content).map_err(|e| format!("invalid {}: {e}", member_manifest_path.display()))?;
    member_manifest.validate(&member_manifest_path)?;
    let pkg = member_manifest.package.ok_or_else(|| format!("{} has no [package] section", member_manifest_path.display()))?;
    let test = member_manifest.test.unwrap_or(TestSection { dir: None, pattern: None, threshold: None, runs: None });
    members.push(Member {
      pkg,
      dir: member_dir,
      test_dir: test.dir.unwrap_or_else(|| "tests/".into()),
      test_pattern: test.pattern.unwrap_or_else(|| "*.lx".into()),
    });
  }

  Ok(Workspace { members })
}

pub fn load_workspace_members_detailed(start: &Path) -> Result<HashMap<String, PathBuf>, String> {
  match find_workspace_root_detailed(start) {
    Ok(root) => load_workspace(&root).map(|ws| ws.member_map()),
    Err(WorkspaceRootLookupError::NotFound { .. }) | Err(WorkspaceRootLookupError::NotWorkspace { .. }) => Ok(HashMap::new()),
    Err(e) => Err(e.to_string()),
  }
}
