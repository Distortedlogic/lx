use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use lx_eval::runtime::RuntimeCtx;

use crate::manifest::{self, Member};

struct TestEntry {
  name: String,
  path: PathBuf,
}

pub fn run_tests_dir(dir: &str) -> ExitCode {
  let cwd = match env::current_dir() {
    Ok(d) => d,
    Err(e) => {
      eprintln!("error: cannot determine cwd: {e}");
      return ExitCode::from(1);
    },
  };
  let (threshold, runs) = match load_test_config(&cwd) {
    Ok(config) => config,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let entries = discover_tests(dir);
  let ws_members = match manifest::load_workspace_members_detailed(&cwd) {
    Ok(ws_members) => ws_members,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let dep_dirs = match manifest::load_dep_dirs_detailed(true, &cwd) {
    Ok(dep_dirs) => dep_dirs,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let result = execute_tests(&entries, &ws_members, &dep_dirs, threshold, runs);
  print_results(&result);
  if result.failed > 0 { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

pub fn run_workspace_tests(member_filter: Option<&str>) -> ExitCode {
  let cwd = match env::current_dir() {
    Ok(d) => d,
    Err(e) => {
      eprintln!("error: cannot determine cwd: {e}");
      return ExitCode::from(1);
    },
  };
  let (threshold, runs) = match load_test_config(&cwd) {
    Ok(config) => config,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let root = match manifest::find_workspace_root_detailed(&cwd) {
    Ok(root) => root,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let ws = match manifest::load_workspace(&root) {
    Ok(ws) => ws,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };

  let members: Vec<&Member> = if let Some(filter) = member_filter {
    let found: Vec<_> = ws.members.iter().filter(|m| m.pkg.name == filter).collect();
    if found.is_empty() {
      eprintln!("error: no member named '{filter}'");
      eprintln!("available: {}", ws.members.iter().map(|m| m.pkg.name.as_str()).collect::<Vec<_>>().join(", "));
      return ExitCode::from(1);
    }
    found
  } else {
    ws.members.iter().collect()
  };

  let ws_members = ws.member_map();
  let dep_dirs = match manifest::load_dep_dirs_detailed(true, &cwd) {
    Ok(dep_dirs) => dep_dirs,
    Err(e) => {
      eprintln!("error: {e}");
      return ExitCode::from(1);
    },
  };
  let mut total_passed = 0u32;
  let mut total_failed = 0u32;
  let mut member_results = Vec::new();
  let mut any_failure = false;

  for member in &members {
    let test_base = member.dir.join(&member.test_dir);
    if !test_base.exists() {
      member_results.push((member.pkg.name.clone(), 0u32, 0u32, true));
      continue;
    }
    let entries = discover_tests_with_pattern(test_base.to_str().unwrap_or("."), &member.test_pattern);
    let result = execute_tests(&entries, &ws_members, &dep_dirs, threshold, runs);
    total_passed += result.passed;
    total_failed += result.failed;
    if result.failed > 0 {
      any_failure = true;
    }
    member_results.push((member.pkg.name.clone(), result.passed, result.failed, false));
  }

  println!();
  for (name, passed, failed, no_tests) in &member_results {
    if *no_tests {
      println!("{name:<16} (no tests)");
    } else {
      println!("{name:<16} {passed} passed, {failed} failed");
    }
  }
  println!("\nTOTAL: {total_passed} passed, {total_failed} failed, {} members", members.len());

  if any_failure { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

fn discover_tests(dir: &str) -> Vec<TestEntry> {
  discover_tests_with_pattern(dir, "*.lx")
}

fn discover_tests_with_pattern(dir: &str, pattern: &str) -> Vec<TestEntry> {
  let mut entries: Vec<TestEntry> = Vec::new();
  let read_dir = match fs::read_dir(dir) {
    Ok(d) => d,
    Err(e) => {
      eprintln!("error: cannot read directory {dir}: {e}");
      return entries;
    },
  };
  for entry in read_dir {
    let Ok(entry) = entry else { continue };
    let path = entry.path();
    if path.extension().and_then(|e| e.to_str()) == Some("lx") {
      let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
      if glob_matches(pattern, &name) {
        entries.push(TestEntry { name, path });
      }
    } else if path.is_dir() {
      let main_lx = path.join("main.lx");
      if main_lx.exists() {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        entries.push(TestEntry { name, path: main_lx });
      }
    }
  }
  entries.sort_by(|a, b| a.name.cmp(&b.name));
  entries
}

struct TestResults {
  passed: u32,
  failed: u32,
  fail_details: Vec<(String, Vec<lx_value::error::LxError>, miette::NamedSource<String>)>,
}

fn execute_tests(
  entries: &[TestEntry],
  workspace_members: &HashMap<String, PathBuf>,
  dep_dirs: &HashMap<String, PathBuf>,
  threshold: Option<f64>,
  runs: Option<u32>,
) -> TestResults {
  let mut passed = 0;
  let mut failed = 0;
  let mut fail_details = Vec::new();
  for entry in entries {
    let source = match fs::read_to_string(&entry.path) {
      Ok(s) => s,
      Err(e) => {
        println!("SKIP {}: {e}", entry.name);
        continue;
      },
    };
    let ctx = Arc::new(RuntimeCtx {
      workspace_members: workspace_members.clone(),
      dep_dirs: dep_dirs.clone(),
      test_threshold: threshold,
      test_runs: runs,
      ..RuntimeCtx::default()
    });
    let filename = entry.path.to_str().unwrap_or(&entry.name);
    match crate::run::run(&source, filename, &ctx, None) {
      Ok(()) => {
        println!("PASS {}", entry.name);
        passed += 1;
      },
      Err(errors) => {
        let named = miette::NamedSource::new(&entry.name, source.clone());
        let first = &errors[0];
        let line = format!("{first}");
        println!("FAIL {}: {line}", entry.name);
        failed += 1;
        fail_details.push((entry.name.clone(), errors, named));
      },
    }
  }
  TestResults { passed, failed, fail_details }
}

fn print_results(results: &TestResults) {
  println!("\n{} passed, {} failed, {} total", results.passed, results.failed, results.passed + results.failed);
  if !results.fail_details.is_empty() {
    println!("\n--- failures ---");
    for (name, errors, named) in &results.fail_details {
      println!("\n{name}:");
      for err in errors {
        let report = miette::Report::new(err.clone()).with_source_code(named.clone());
        eprintln!("{report:?}");
      }
    }
  }
}

fn load_test_config(cwd: &std::path::Path) -> Result<(Option<f64>, Option<u32>), String> {
  let Some((_root, manifest)) = manifest::load_nearest_manifest(cwd)? else {
    return Ok((None, None));
  };
  Ok(match manifest.test {
    Some(test) => (test.threshold, test.runs),
    None => (None, None),
  })
}

fn glob_matches(pattern: &str, name: &str) -> bool {
  if let Some(idx) = pattern.find('*') {
    let prefix = &pattern[..idx];
    let suffix = &pattern[idx + 1..];
    name.starts_with(prefix) && name.ends_with(suffix)
  } else {
    pattern == name
  }
}
