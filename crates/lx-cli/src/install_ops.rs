use std::path::{Path, PathBuf};
use std::process::Command;

use crate::manifest::{DepSpec, RootManifest};

pub fn install_git(
    dest: &Path,
    url: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("clone").arg("--depth").arg("1");
    if let Some(b) = branch {
        cmd.arg("--branch").arg(b);
    } else if let Some(t) = tag {
        cmd.arg("--branch").arg(t);
    }
    cmd.arg(url).arg(dest);
    let output = cmd.output().map_err(|e| format!("git clone failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git clone failed: {stderr}"));
    }
    if let Some(r) = rev {
        let checkout = Command::new("git")
            .arg("checkout")
            .arg(r)
            .current_dir(dest)
            .output()
            .map_err(|e| format!("git checkout failed: {e}"))?;
        if !checkout.status.success() {
            let stderr = String::from_utf8_lossy(&checkout.stderr);
            return Err(format!("git checkout {r} failed: {stderr}"));
        }
    }
    let commit = resolve_git_commit(dest)?;
    Ok(format!("git+{url}#{commit}"))
}

fn resolve_git_commit(repo: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(repo)
        .output()
        .map_err(|e| format!("git rev-parse failed: {e}"))?;
    if !output.status.success() {
        return Err("git rev-parse HEAD failed".into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn install_path(dest: &Path, source: &Path) -> Result<String, String> {
    if !source.exists() {
        return Err(format!("path {} does not exist", source.display()));
    }
    if dest.exists() {
        return Ok(format!("path+{}", source.display()));
    }
    #[cfg(unix)]
    {
        let canonical = std::fs::canonicalize(source)
            .map_err(|e| format!("cannot resolve {}: {e}", source.display()))?;
        std::os::unix::fs::symlink(&canonical, dest).map_err(|e| format!("symlink failed: {e}"))?;
    }
    #[cfg(not(unix))]
    {
        copy_dir(source, dest)?;
    }
    Ok(format!("path+{}", source.display()))
}

#[cfg(not(unix))]
fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("mkdir failed: {e}"))?;
    let entries = std::fs::read_dir(src).map_err(|e| format!("readdir failed: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("readdir entry failed: {e}"))?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest).map_err(|e| format!("copy failed: {e}"))?;
        }
    }
    Ok(())
}

pub fn add_dep_to_manifest(
    root: &Path,
    manifest: &mut RootManifest,
    pkg_arg: &str,
) -> Result<(), String> {
    let (name, spec) = parse_pkg_arg(pkg_arg)?;
    let deps = manifest.dependencies.get_or_insert_with(Default::default);
    deps.insert(name.clone(), spec.clone());
    write_dep_to_toml(root, &name, &spec)
}

fn parse_pkg_arg(arg: &str) -> Result<(String, DepSpec), String> {
    if arg.starts_with("http://") || arg.starts_with("https://") || arg.starts_with("git@") {
        let name = url_to_name(arg);
        return Ok((
            name,
            DepSpec::Git {
                git: arg.to_string(),
                branch: Some("main".to_string()),
                tag: None,
                rev: None,
            },
        ));
    }
    let path = PathBuf::from(arg);
    if path.exists() && path.is_dir() {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("dep")
            .to_string();
        return Ok((
            name,
            DepSpec::Path {
                path: arg.to_string(),
            },
        ));
    }
    Err(format!(
        "cannot determine dependency type for '{arg}' — provide a git URL or local path"
    ))
}

fn url_to_name(url: &str) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or("dep")
        .trim_end_matches(".git")
        .to_string()
}

fn write_dep_to_toml(root: &Path, name: &str, spec: &DepSpec) -> Result<(), String> {
    let manifest_path = root.join("lx.toml");
    let content =
        std::fs::read_to_string(&manifest_path).map_err(|e| format!("cannot read lx.toml: {e}"))?;
    let dep_line = match spec {
        DepSpec::Git {
            git,
            branch,
            tag,
            rev,
        } => {
            let mut parts = vec![format!("git = \"{git}\"")];
            if let Some(b) = branch {
                parts.push(format!("branch = \"{b}\""));
            }
            if let Some(t) = tag {
                parts.push(format!("tag = \"{t}\""));
            }
            if let Some(r) = rev {
                parts.push(format!("rev = \"{r}\""));
            }
            format!("{name} = {{ {} }}", parts.join(", "))
        }
        DepSpec::Path { path } => format!("{name} = {{ path = \"{path}\" }}"),
    };
    let new_content = if content.contains("[dependencies]") {
        let idx = content.find("[dependencies]").expect("just checked");
        let after = &content[idx..];
        let insert_pos = if let Some(next_section) = after[1..].find('[') {
            idx + 1 + next_section
        } else {
            content.len()
        };
        let mut result = content[..insert_pos].to_string();
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&dep_line);
        result.push('\n');
        result.push_str(&content[insert_pos..]);
        result
    } else {
        let mut result = content.clone();
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str("\n[dependencies]\n");
        result.push_str(&dep_line);
        result.push('\n');
        result
    };
    std::fs::write(&manifest_path, new_content).map_err(|e| format!("cannot write lx.toml: {e}"))
}
