use std::path::Path;
use std::process::ExitCode;

use crate::manifest;

pub fn list_workspace() -> ExitCode {
  let cwd = match std::env::current_dir() {
    Ok(d) => d,
    Err(e) => {
      eprintln!("error: cannot determine cwd: {e}");
      return ExitCode::from(1);
    },
  };
  let Some(root) = manifest::find_workspace_root(&cwd) else {
    eprintln!("error: no workspace lx.toml found");
    return ExitCode::from(1);
  };
  let Ok(ws) = manifest::load_workspace(&root) else {
    eprintln!("error: failed to load workspace");
    return ExitCode::from(1);
  };

  for member in &ws.members {
    let file_count = count_lx_files(&member.dir);
    let entry_display = member.pkg.entry.as_deref().unwrap_or("(no entry)");
    let version = member.pkg.version.as_deref().unwrap_or("0.0.0");
    let test_base = member.dir.join(&member.test_dir);
    let test_count = if test_base.exists() { count_matching_files(test_base.to_str().unwrap_or("."), &member.test_pattern) } else { 0 };
    let desc = member.pkg.description.as_deref().unwrap_or("");
    let mut extra = String::new();
    if let Some(ref lic) = member.pkg.license {
      extra.push_str(&format!(" [{lic}]"));
    }
    if let Some(ref lx_ver) = member.pkg.lx {
      extra.push_str(&format!(" lx{lx_ver}"));
    }
    if let Some(ref authors) = member.pkg.authors
      && !authors.is_empty()
    {
      extra.push_str(&format!(" by {}", authors.join(", ")));
    }
    println!("  {:<12} {:<7} {:>3} files  {:<14} {:>3} tests  {desc}{extra}", member.pkg.name, version, file_count, entry_display, test_count,);
  }
  ExitCode::SUCCESS
}

fn count_lx_files(dir: &Path) -> usize {
  walkdir(dir)
}

fn count_matching_files(dir: &str, pattern: &str) -> usize {
  let (prefix, suffix) = split_glob(pattern);
  let Ok(read_dir) = std::fs::read_dir(dir) else {
    return 0;
  };
  read_dir
    .filter_map(|e| e.ok())
    .filter(|e| {
      let name = e.file_name().to_string_lossy().to_string();
      name.ends_with(".lx") && name.starts_with(&prefix) && name.ends_with(&suffix)
    })
    .count()
}

fn walkdir(dir: &Path) -> usize {
  let Ok(read_dir) = std::fs::read_dir(dir) else {
    return 0;
  };
  let mut count = 0;
  for entry in read_dir.filter_map(|e| e.ok()) {
    let path = entry.path();
    if path.is_dir() {
      count += walkdir(&path);
    } else if path.extension().and_then(|e| e.to_str()) == Some("lx") {
      count += 1;
    }
  }
  count
}

fn split_glob(pattern: &str) -> (String, String) {
  if let Some(idx) = pattern.find('*') { (pattern[..idx].to_string(), pattern[idx + 1..].to_string()) } else { (pattern.to_string(), String::new()) }
}
