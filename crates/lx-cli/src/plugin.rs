use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Deserialize;

#[derive(Deserialize)]
struct PluginManifest {
  plugin: PluginMeta,
}

#[derive(Deserialize)]
struct PluginMeta {
  name: String,
  version: String,
  description: Option<String>,
  wasm: String,
}

fn global_plugins_dir() -> Result<PathBuf, String> {
  let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
  Ok(PathBuf::from(home).join(".lx").join("plugins"))
}

fn validate_plugin_name(name: &str) -> Result<(), String> {
  if name.is_empty() {
    return Err("plugin name cannot be empty".to_string());
  }
  if name.contains('/') || name.contains('\\') || name.contains("..") || name.chars().any(|c| c.is_whitespace()) {
    return Err(format!("invalid plugin name '{name}': must not contain '/', '\\', '..', or whitespace"));
  }
  Ok(())
}

fn read_manifest(dir: &Path) -> Result<PluginManifest, String> {
  let manifest_path = dir.join("plugin.toml");
  let content = std::fs::read_to_string(&manifest_path).map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
  let manifest: PluginManifest = toml::from_str(&content).map_err(|e| format!("invalid plugin.toml in {}: {e}", dir.display()))?;
  if manifest.plugin.name.is_empty() {
    return Err(format!("plugin.toml in {} missing [plugin].name", dir.display()));
  }
  if manifest.plugin.version.is_empty() {
    return Err(format!("plugin.toml in {} missing [plugin].version", dir.display()));
  }
  if manifest.plugin.wasm.is_empty() {
    return Err(format!("plugin.toml in {} missing [plugin].wasm", dir.display()));
  }
  Ok(manifest)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
  std::fs::create_dir_all(dst).map_err(|e| format!("cannot create {}: {e}", dst.display()))?;
  let entries = std::fs::read_dir(src).map_err(|e| format!("cannot read directory {}: {e}", src.display()))?;
  for entry in entries {
    let entry = entry.map_err(|e| format!("error reading entry in {}: {e}", src.display()))?;
    let src_path = entry.path();
    let real_path = std::fs::canonicalize(&src_path).map_err(|e| format!("cannot resolve {}: {e}", src_path.display()))?;
    let file_name = entry.file_name();
    let dst_path = dst.join(&file_name);
    let meta = std::fs::metadata(&real_path).map_err(|e| format!("cannot stat {}: {e}", real_path.display()))?;
    if meta.is_dir() {
      copy_dir_recursive(&real_path, &dst_path)?;
    } else {
      std::fs::copy(&real_path, &dst_path).map_err(|e| format!("cannot copy {} -> {}: {e}", real_path.display(), dst_path.display()))?;
    }
  }
  Ok(())
}

pub fn install(path: &Path) -> ExitCode {
  let source = match std::fs::canonicalize(path) {
    Ok(p) => p,
    Err(e) => {
      eprintln!("error: cannot resolve path {}: {e}", path.display());
      return ExitCode::from(1);
    },
  };
  let manifest = match read_manifest(&source) {
    Ok(m) => m,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let wasm_path = source.join(&manifest.plugin.wasm);
  if !wasm_path.exists() {
    eprintln!("error: wasm file '{}' referenced in plugin.toml does not exist", manifest.plugin.wasm);
    return ExitCode::from(1);
  }
  let global_dir = match global_plugins_dir() {
    Ok(d) => d,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  if let Err(e) = std::fs::create_dir_all(&global_dir) {
    eprintln!("error: cannot create {}: {e}", global_dir.display());
    return ExitCode::from(1);
  }
  let target = global_dir.join(&manifest.plugin.name);
  if target.exists() {
    if let Ok(old_manifest) = read_manifest(&target) {
      eprintln!("updating {} {} → {}", manifest.plugin.name, old_manifest.plugin.version, manifest.plugin.version);
    }
    if let Err(e) = std::fs::remove_dir_all(&target) {
      eprintln!("error: cannot remove old plugin at {}: {e}", target.display());
      return ExitCode::from(1);
    }
  }
  if let Err(e) = copy_dir_recursive(&source, &target) {
    eprintln!("error: {e}");
    return ExitCode::from(1);
  }
  println!("installed {} {} to ~/.lx/plugins/{}/", manifest.plugin.name, manifest.plugin.version, manifest.plugin.name);
  ExitCode::SUCCESS
}

pub fn list() -> ExitCode {
  let mut entries: Vec<(String, String, &str, String)> = Vec::new();
  if let Ok(global_dir) = global_plugins_dir() {
    scan_plugins_dir(&global_dir, "global", &mut entries);
  }
  if let Ok(cwd) = std::env::current_dir() {
    let local_dir = cwd.join(".lx").join("plugins");
    scan_plugins_dir(&local_dir, "local", &mut entries);
  }
  if entries.is_empty() {
    println!("no plugins installed");
    return ExitCode::SUCCESS;
  }
  let w_name = entries.iter().map(|e| e.0.len()).max().unwrap_or(4).max(4);
  let w_ver = entries.iter().map(|e| e.1.len()).max().unwrap_or(7).max(7);
  let w_loc = entries.iter().map(|e| e.2.len()).max().unwrap_or(8).max(8);
  println!("{:<w_name$}  {:<w_ver$}  {:<w_loc$}  Description", "Name", "Version", "Location");
  for (name, version, location, description) in &entries {
    println!("{name:<w_name$}  {version:<w_ver$}  {location:<w_loc$}  {description}");
  }
  ExitCode::SUCCESS
}

fn scan_plugins_dir<'a>(dir: &Path, location: &'a str, out: &mut Vec<(String, String, &'a str, String)>) {
  let Ok(rd) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in rd {
    let Ok(entry) = entry else {
      continue;
    };
    let path = entry.path();
    if !path.is_dir() {
      continue;
    }
    match read_manifest(&path) {
      Ok(m) => {
        let desc = m.plugin.description.unwrap_or_default();
        out.push((m.plugin.name, m.plugin.version, location, desc));
      },
      Err(e) => {
        eprintln!("warning: {e}");
      },
    }
  }
}

pub fn remove(name: &str) -> ExitCode {
  let global_dir = match global_plugins_dir() {
    Ok(d) => d,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let target = global_dir.join(name);
  if !target.exists() {
    eprintln!("error: plugin '{name}' not found in ~/.lx/plugins/");
    return ExitCode::from(1);
  }
  if let Err(e) = std::fs::remove_dir_all(&target) {
    eprintln!("error: cannot remove {}: {e}", target.display());
    return ExitCode::from(1);
  }
  println!("removed {name}");
  ExitCode::SUCCESS
}

pub fn new_plugin(name: &str) -> ExitCode {
  if let Err(e) = validate_plugin_name(name) {
    eprintln!("error: {e}");
    return ExitCode::from(1);
  }
  let dir = PathBuf::from(name);
  if dir.exists() {
    eprintln!("error: directory '{name}' already exists");
    return ExitCode::from(1);
  }
  let src_dir = dir.join("src");
  let cargo_dir = dir.join(".cargo");
  if let Err(e) = std::fs::create_dir_all(&src_dir) {
    eprintln!("error: cannot create {}: {e}", src_dir.display());
    return ExitCode::from(1);
  }
  if let Err(e) = std::fs::create_dir_all(&cargo_dir) {
    eprintln!("error: cannot create {}: {e}", cargo_dir.display());
    return ExitCode::from(1);
  }
  let underscore_name = name.replace('-', "_");
  let cargo_toml = format!(
    "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n\
     [lib]\ncrate-type = [\"cdylib\"]\n\n\
     [dependencies]\nextism-pdk = \"1.4.1\"\nserde = {{ version = \"1\", features = [\"derive\"] }}\nserde_json = \"1\"\n"
  );
  let lib_rs = format!(
    "use extism_pdk::*;\n\n\
     #[plugin_fn]\n\
     pub fn hello(input: String) -> FnResult<String> {{\n    \
     Ok(format!(\"Hello from {name}: {{input}}\"))\n\
     }}\n"
  );
  let plugin_toml = format!(
    "[plugin]\nname = \"{name}\"\nversion = \"0.1.0\"\ndescription = \"\"\n\
     wasm = \"target/wasm32-unknown-unknown/release/{underscore_name}.wasm\"\n\n\
     [exports]\nhello = {{ arity = 1 }}\n"
  );
  let cargo_config = "[build]\ntarget = \"wasm32-unknown-unknown\"\n";
  let writes: Vec<(PathBuf, &str)> = vec![
    (dir.join("Cargo.toml"), &cargo_toml),
    (dir.join("src").join("lib.rs"), &lib_rs),
    (dir.join("plugin.toml"), &plugin_toml),
    (dir.join(".cargo").join("config.toml"), cargo_config),
  ];
  for (path, content) in writes {
    if let Err(e) = std::fs::write(&path, content) {
      eprintln!("error: cannot write {}: {e}", path.display());
      return ExitCode::from(1);
    }
  }
  println!("Created plugin project '{name}'\n");
  println!("Build:   cargo build --release");
  println!("Install: lx plugin install ./{name}");
  ExitCode::SUCCESS
}
