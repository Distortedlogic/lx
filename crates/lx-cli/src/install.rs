use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use crate::install_ops;
use crate::lockfile::LockFile;
use crate::manifest::{self, DepSpec};

pub fn run_install(package: Option<&str>) -> ExitCode {
  let Ok(cwd) = env::current_dir() else {
    eprintln!("error: cannot determine cwd");
    return ExitCode::from(1);
  };
  let (root, mut manifest) = match manifest::load_nearest_manifest(&cwd) {
    Ok(Some((root, manifest))) => (root, manifest),
    Ok(None) => {
      eprintln!("error: no lx.toml found");
      return ExitCode::from(1);
    },
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  if let Some(pkg_arg) = package
    && let Err(e) = install_ops::add_dep_to_manifest(&root, &mut manifest, pkg_arg)
  {
    eprintln!("error: {e}");
    return ExitCode::from(1);
  }
  let deps = manifest.dependencies.clone().unwrap_or_default();
  let dev_deps = manifest.deps_table.as_ref().and_then(|dt| dt.dev.clone()).unwrap_or_default();
  if deps.is_empty() && dev_deps.is_empty() {
    println!("no dependencies to install");
    return ExitCode::SUCCESS;
  }
  let deps_dir = manifest::deps_dir(&root);
  if let Err(e) = fs::create_dir_all(&deps_dir) {
    eprintln!("error: cannot create .lx/deps: {e}");
    return ExitCode::from(1);
  }
  let mut lock = match LockFile::load(&root) {
    Ok(l) => l,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let mut failed = false;
  for (name, spec) in &deps {
    match install_dep(&root, &deps_dir, name, spec, &lock) {
      Ok((source, version)) => {
        lock.upsert(name, &source, version.as_deref());
        println!("  installed {name}");
      },
      Err(e) => {
        eprintln!("error: failed to install {name}: {e}");
        failed = true;
      },
    }
  }
  for (name, spec) in &dev_deps {
    match install_dep(&root, &deps_dir, name, spec, &lock) {
      Ok((source, version)) => {
        lock.upsert(name, &source, version.as_deref());
        println!("  installed {name} (dev)");
      },
      Err(e) => {
        eprintln!("error: failed to install dev dep {name}: {e}");
        failed = true;
      },
    }
  }
  if !dev_deps.is_empty() {
    let dev_marker = deps_dir.join(".dev-deps");
    let dev_names: Vec<&str> = dev_deps.keys().map(|k| k.as_str()).collect();
    if let Err(e) = fs::write(&dev_marker, dev_names.join("\n")) {
      eprintln!("error: cannot write .dev-deps marker: {e}");
      return ExitCode::from(1);
    }
  }
  if let Err(e) = lock.save(&root) {
    eprintln!("error: {e}");
    return ExitCode::from(1);
  }
  if failed { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

pub fn run_update(package: Option<&str>) -> ExitCode {
  let Ok(cwd) = env::current_dir() else {
    eprintln!("error: cannot determine cwd");
    return ExitCode::from(1);
  };
  let (root, manifest) = match manifest::load_nearest_manifest(&cwd) {
    Ok(Some((root, manifest))) => (root, manifest),
    Ok(None) => {
      eprintln!("error: no lx.toml found");
      return ExitCode::from(1);
    },
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let deps = match manifest.dependencies {
    Some(ref d) => d.clone(),
    None => {
      println!("no dependencies to update");
      return ExitCode::SUCCESS;
    },
  };
  let deps_dir = manifest::deps_dir(&root);
  let mut lock = match LockFile::load(&root) {
    Ok(l) => l,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let mut failed = false;
  let targets: Vec<(&String, &DepSpec)> = if let Some(name) = package {
    match deps.get(name) {
      Some(spec) => {
        let key = deps.keys().find(|k| k.as_str() == name).expect("just matched");
        vec![(key, spec)]
      },
      None => {
        eprintln!("error: '{name}' not found in [dependencies]");
        return ExitCode::from(1);
      },
    }
  } else {
    deps.iter().collect()
  };
  for (name, spec) in targets {
    match update_dep(&deps_dir, name, spec) {
      Ok((source, version)) => {
        lock.upsert(name, &source, version.as_deref());
        println!("  updated {name}");
      },
      Err(e) => {
        eprintln!("error: failed to update {name}: {e}");
        failed = true;
      },
    }
  }
  if let Err(e) = lock.save(&root) {
    eprintln!("error: {e}");
    return ExitCode::from(1);
  }
  if failed { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

fn install_dep(root: &Path, deps_dir: &Path, name: &str, spec: &DepSpec, lock: &LockFile) -> Result<(String, Option<String>), String> {
  match spec {
    DepSpec::Git { git, branch, tag, rev } => {
      let dest = deps_dir.join(name);
      if dest.exists()
        && let Some(locked) = lock.get(name)
      {
        return Ok((locked.source.clone(), locked.version.clone()));
      }
      let source = install_ops::install_git(&dest, git, branch.as_deref(), tag.as_deref(), rev.as_deref())?;
      let version = tag.as_deref().or(branch.as_deref()).or(rev.as_deref()).map(|v| v.to_string());
      Ok((source, version))
    },
    DepSpec::Path { path } => {
      let source_path = root.join(path);
      let dest = deps_dir.join(name);
      let source = install_ops::install_path(&dest, &source_path)?;
      Ok((source, Some("path".to_string())))
    },
  }
}

fn update_dep(deps_dir: &Path, name: &str, spec: &DepSpec) -> Result<(String, Option<String>), String> {
  match spec {
    DepSpec::Git { git, branch, tag, rev } => {
      let dest = deps_dir.join(name);
      if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| format!("cannot remove {}: {e}", dest.display()))?;
      }
      let source = install_ops::install_git(&dest, git, branch.as_deref(), tag.as_deref(), rev.as_deref())?;
      let version = tag.as_deref().or(branch.as_deref()).or(rev.as_deref()).map(|v| v.to_string());
      Ok((source, version))
    },
    DepSpec::Path { .. } => Ok((format!("path+{name}"), Some("path".to_string()))),
  }
}
