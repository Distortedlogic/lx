use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct RootManifest {
    pub workspace: Option<WorkspaceSection>,
    pub package: Option<PackageSection>,
    pub test: Option<TestSection>,
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
}

#[derive(Deserialize)]
pub struct TestSection {
    pub dir: Option<String>,
    pub pattern: Option<String>,
}

pub struct Workspace {
    pub members: Vec<Member>,
}

pub struct Member {
    pub name: String,
    pub version: Option<String>,
    pub dir: PathBuf,
    pub entry: Option<String>,
    pub description: Option<String>,
    pub test_dir: String,
    pub test_pattern: String,
}

pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("lx.toml");
        if candidate.exists() {
            let content = std::fs::read_to_string(&candidate).ok()?;
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
    let manifest_path = root.join("lx.toml");
    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
    let manifest: RootManifest = toml::from_str(&content)
        .map_err(|e| format!("invalid {}: {e}", manifest_path.display()))?;
    let ws = manifest
        .workspace
        .ok_or_else(|| format!("{} has no [workspace] section", manifest_path.display()))?;

    let mut members = Vec::new();
    for member_path in &ws.members {
        let member_dir = root.join(member_path);
        let member_manifest_path = member_dir.join("lx.toml");
        if !member_manifest_path.exists() {
            return Err(format!(
                "member '{}' has no lx.toml at {}",
                member_path,
                member_manifest_path.display()
            ));
        }
        let member_content = std::fs::read_to_string(&member_manifest_path)
            .map_err(|e| format!("cannot read {}: {e}", member_manifest_path.display()))?;
        let member_manifest: RootManifest = toml::from_str(&member_content)
            .map_err(|e| format!("invalid {}: {e}", member_manifest_path.display()))?;
        let pkg = member_manifest.package.ok_or_else(|| {
            format!(
                "{} has no [package] section",
                member_manifest_path.display()
            )
        })?;
        let test = member_manifest.test.unwrap_or(TestSection {
            dir: None,
            pattern: None,
        });
        members.push(Member {
            name: pkg.name,
            version: pkg.version,
            dir: member_dir,
            entry: pkg.entry,
            description: pkg.description,
            test_dir: test.dir.unwrap_or_else(|| "tests/".into()),
            test_pattern: test.pattern.unwrap_or_else(|| "*.lx".into()),
        });
    }

    Ok(Workspace { members })
}

pub fn workspace_member_map(ws: &Workspace) -> HashMap<String, PathBuf> {
    ws.members
        .iter()
        .map(|m| (m.name.clone(), m.dir.clone()))
        .collect()
}

pub fn try_load_workspace_members() -> HashMap<String, PathBuf> {
    let Ok(cwd) = std::env::current_dir() else {
        return HashMap::new();
    };
    let Some(root) = find_workspace_root(&cwd) else {
        return HashMap::new();
    };
    let Ok(ws) = load_workspace(&root) else {
        return HashMap::new();
    };
    workspace_member_map(&ws)
}
