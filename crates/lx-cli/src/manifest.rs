use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RootManifest {
  pub workspace: Option<WorkspaceSection>,
  pub package: Option<PackageSection>,
  pub test: Option<TestSection>,
  pub backends: Option<BackendsSection>,
  pub dependencies: Option<HashMap<String, DepSpec>>,
  #[serde(rename = "deps")]
  pub deps_table: Option<DepsTable>,
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
pub enum EmitBackend {
  Noop,
  Stdout,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogBackend {
  Noop,
  Stderr,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LlmBackend {
  ClaudeCode,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HttpBackend {
  Reqwest,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum YieldBackend {
  StdinStdout,
}

#[derive(Deserialize)]
pub struct BackendsSection {
  pub llm: Option<LlmBackend>,
  pub http: Option<HttpBackend>,
  pub emit: Option<EmitBackend>,
  #[serde(rename = "yield")]
  pub yield_backend: Option<YieldBackend>,
  pub log: Option<LogBackend>,
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

pub fn find_manifest_root(start: &Path) -> Option<PathBuf> {
  let mut dir = start.to_path_buf();
  loop {
    let candidate = dir.join(lx::LX_MANIFEST);
    if candidate.exists() {
      return Some(dir);
    }
    if !dir.pop() {
      return None;
    }
  }
}

pub fn load_manifest(root: &Path) -> Result<RootManifest, String> {
  let manifest_path = root.join(lx::LX_MANIFEST);
  let content = fs::read_to_string(&manifest_path).map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
  let manifest: RootManifest = toml::from_str(&content).map_err(|e| format!("invalid {}: {e}", manifest_path.display()))?;
  validate_manifest(&manifest, &manifest_path)?;
  Ok(manifest)
}

fn validate_manifest(manifest: &RootManifest, path: &Path) -> Result<(), String> {
  if let Some(ref pkg) = manifest.package {
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

pub fn deps_dir(root: &Path) -> PathBuf {
  root.join(".lx").join("deps")
}

pub fn try_load_dep_dirs() -> HashMap<String, PathBuf> {
  load_dep_dirs_filtered(true)
}

pub fn try_load_dep_dirs_no_dev() -> HashMap<String, PathBuf> {
  load_dep_dirs_filtered(false)
}

fn load_dep_dirs_filtered(include_dev: bool) -> HashMap<String, PathBuf> {
  let Ok(cwd) = env::current_dir() else {
    return HashMap::new();
  };
  let Some(root) = find_manifest_root(&cwd) else {
    return HashMap::new();
  };
  let deps = deps_dir(&root);
  if !deps.exists() {
    return HashMap::new();
  }
  let dev_names: Vec<String> = if !include_dev {
    let marker = deps.join(".dev-deps");
    fs::read_to_string(marker).unwrap_or_default().lines().filter(|l| !l.is_empty()).map(|l| l.to_string()).collect()
  } else {
    Vec::new()
  };
  let Ok(entries) = fs::read_dir(&deps) else {
    return HashMap::new();
  };
  let mut map = HashMap::new();
  for entry in entries.filter_map(|e| e.ok()) {
    let path = entry.path();
    if path.is_dir()
      && let Some(name) = path.file_name().and_then(|n| n.to_str())
    {
      if !include_dev && dev_names.iter().any(|d| d == name) {
        continue;
      }
      map.insert(name.to_string(), path);
    }
  }
  map
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
    let candidate = dir.join(lx::LX_MANIFEST);
    if candidate.exists() {
      let content = fs::read_to_string(&candidate).ok()?;
      let manifest: RootManifest = toml::from_str(&content).ok()?;
      if manifest.workspace.is_some() {
        return Some(dir);
      }
    }
    if !dir.pop() {
      return None;
    }
  }
}

pub fn load_workspace(root: &Path) -> Result<Workspace, String> {
  let manifest_path = root.join(lx::LX_MANIFEST);
  let content = fs::read_to_string(&manifest_path).map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
  let manifest: RootManifest = toml::from_str(&content).map_err(|e| format!("invalid {}: {e}", manifest_path.display()))?;
  let ws = manifest.workspace.ok_or_else(|| format!("{} has no [workspace] section", manifest_path.display()))?;

  let mut members = Vec::new();
  for member_path in &ws.members {
    let member_dir = root.join(member_path);
    let member_manifest_path = member_dir.join(lx::LX_MANIFEST);
    if !member_manifest_path.exists() {
      return Err(format!("member '{}' has no lx.toml at {}", member_path, member_manifest_path.display()));
    }
    let member_content = fs::read_to_string(&member_manifest_path).map_err(|e| format!("cannot read {}: {e}", member_manifest_path.display()))?;
    let member_manifest: RootManifest = toml::from_str(&member_content).map_err(|e| format!("invalid {}: {e}", member_manifest_path.display()))?;
    validate_manifest(&member_manifest, &member_manifest_path)?;
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

pub fn try_load_workspace_members() -> HashMap<String, PathBuf> {
  let Ok(cwd) = env::current_dir() else {
    return HashMap::new();
  };
  let Some(root) = find_workspace_root(&cwd) else {
    return HashMap::new();
  };
  let Ok(ws) = load_workspace(&root) else {
    return HashMap::new();
  };
  ws.member_map()
}
