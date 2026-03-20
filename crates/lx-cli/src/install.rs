use std::path::Path;
use std::process::ExitCode;

use crate::install_ops;
use crate::lockfile::LockFile;
use crate::manifest::{self, DepSpec};

pub fn run_install(package: Option<&str>) -> ExitCode {
    let Ok(cwd) = std::env::current_dir() else {
        eprintln!("error: cannot determine cwd");
        return ExitCode::from(1);
    };
    let Some(root) = manifest::find_manifest_root(&cwd) else {
        eprintln!("error: no lx.toml found");
        return ExitCode::from(1);
    };
    let mut manifest = match manifest::load_manifest(&root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    if let Some(pkg_arg) = package
        && let Err(e) = install_ops::add_dep_to_manifest(&root, &mut manifest, pkg_arg)
    {
        eprintln!("error: {e}");
        return ExitCode::from(1);
    }
    let deps = match manifest.dependencies {
        Some(ref d) => d.clone(),
        None => {
            println!("no dependencies to install");
            return ExitCode::SUCCESS;
        }
    };
    let deps_dir = manifest::deps_dir(&root);
    if let Err(e) = std::fs::create_dir_all(&deps_dir) {
        eprintln!("error: cannot create .lx/deps: {e}");
        return ExitCode::from(1);
    }
    let mut lock = match LockFile::load(&root) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    let mut failed = false;
    for (name, spec) in &deps {
        match install_dep(&root, &deps_dir, name, spec, &lock) {
            Ok(source) => {
                lock.upsert(name, &source);
                println!("  installed {name}");
            }
            Err(e) => {
                eprintln!("error: failed to install {name}: {e}");
                failed = true;
            }
        }
    }
    if let Err(e) = lock.save(&root) {
        eprintln!("error: {e}");
        return ExitCode::from(1);
    }
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

pub fn run_update(package: Option<&str>) -> ExitCode {
    let Ok(cwd) = std::env::current_dir() else {
        eprintln!("error: cannot determine cwd");
        return ExitCode::from(1);
    };
    let Some(root) = manifest::find_manifest_root(&cwd) else {
        eprintln!("error: no lx.toml found");
        return ExitCode::from(1);
    };
    let manifest = match manifest::load_manifest(&root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    let deps = match manifest.dependencies {
        Some(ref d) => d.clone(),
        None => {
            println!("no dependencies to update");
            return ExitCode::SUCCESS;
        }
    };
    let deps_dir = manifest::deps_dir(&root);
    let mut lock = match LockFile::load(&root) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    let mut failed = false;
    let targets: Vec<(&String, &DepSpec)> = if let Some(name) = package {
        match deps.get(name) {
            Some(spec) => {
                let key = deps
                    .keys()
                    .find(|k| k.as_str() == name)
                    .expect("just matched");
                vec![(key, spec)]
            }
            None => {
                eprintln!("error: '{name}' not found in [dependencies]");
                return ExitCode::from(1);
            }
        }
    } else {
        deps.iter().collect()
    };
    for (name, spec) in targets {
        match update_dep(&deps_dir, name, spec) {
            Ok(source) => {
                lock.upsert(name, &source);
                println!("  updated {name}");
            }
            Err(e) => {
                eprintln!("error: failed to update {name}: {e}");
                failed = true;
            }
        }
    }
    if let Err(e) = lock.save(&root) {
        eprintln!("error: {e}");
        return ExitCode::from(1);
    }
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn install_dep(
    root: &Path,
    deps_dir: &Path,
    name: &str,
    spec: &DepSpec,
    lock: &LockFile,
) -> Result<String, String> {
    match spec {
        DepSpec::Git {
            git,
            branch,
            tag,
            rev,
        } => {
            let dest = deps_dir.join(name);
            if dest.exists()
                && let Some(locked) = lock.get(name)
            {
                return Ok(locked.source.clone());
            }
            install_ops::install_git(
                &dest,
                git,
                branch.as_deref(),
                tag.as_deref(),
                rev.as_deref(),
            )
        }
        DepSpec::Path { path } => {
            let source_path = root.join(path);
            let dest = deps_dir.join(name);
            install_ops::install_path(&dest, &source_path)
        }
    }
}

fn update_dep(deps_dir: &Path, name: &str, spec: &DepSpec) -> Result<String, String> {
    match spec {
        DepSpec::Git {
            git,
            branch,
            tag,
            rev,
        } => {
            let dest = deps_dir.join(name);
            if dest.exists() {
                std::fs::remove_dir_all(&dest)
                    .map_err(|e| format!("cannot remove {}: {e}", dest.display()))?;
            }
            install_ops::install_git(
                &dest,
                git,
                branch.as_deref(),
                tag.as_deref(),
                rev.as_deref(),
            )
        }
        DepSpec::Path { .. } => Ok(format!("path+{name}")),
    }
}
