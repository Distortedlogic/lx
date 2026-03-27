use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
  #[serde(default)]
  pub package: Vec<LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
  pub name: String,
  pub source: String,
  pub version: Option<String>,
}

impl LockFile {
  pub fn load(root: &Path) -> Result<Self, String> {
    let path = root.join("lx.lock");
    if !path.exists() {
      return Ok(Self { package: Vec::new() });
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("cannot read lx.lock: {e}"))?;
    toml::from_str(&content).map_err(|e| format!("invalid lx.lock: {e}"))
  }

  pub fn save(&self, root: &Path) -> Result<(), String> {
    let path = root.join("lx.lock");
    let content = toml::to_string_pretty(self).map_err(|e| format!("cannot serialize lx.lock: {e}"))?;
    fs::write(&path, content).map_err(|e| format!("cannot write lx.lock: {e}"))
  }

  pub fn upsert(&mut self, name: &str, source: &str, version: Option<&str>) {
    if let Some(pkg) = self.package.iter_mut().find(|p| p.name == name) {
      pkg.source = source.to_string();
      pkg.version = version.map(|v| v.to_string());
    } else {
      self.package.push(LockedPackage { name: name.to_string(), source: source.to_string(), version: version.map(|v| v.to_string()) });
    }
    self.package.sort_by(|a, b| a.name.cmp(&b.name));
  }

  pub fn get(&self, name: &str) -> Option<&LockedPackage> {
    self.package.iter().find(|p| p.name == name)
  }
}
