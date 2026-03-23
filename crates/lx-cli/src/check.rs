use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use lx::checker::{CheckResult, DiagLevel, Diagnostic, check};
use lx::error::LxError;
use miette::{NamedSource, Report};

use crate::manifest;
use crate::run;

pub fn check_file(path: &str, strict: bool) -> ExitCode {
  let (source, program) = match run::read_and_parse(path) {
    Ok(sp) => sp,
    Err(code) => return code,
  };
  let source_arc: Arc<str> = Arc::from(source.as_str());
  let result = check(&program, source_arc);
  if result.diagnostics.is_empty() {
    println!("ok: {path}");
    ExitCode::SUCCESS
  } else {
    let mut errors = 0u32;
    let mut warnings = 0u32;
    for d in &result.diagnostics {
      let prefix = match d.level {
        DiagLevel::Error => {
          errors += 1;
          "error"
        },
        DiagLevel::Warning => {
          warnings += 1;
          "warning"
        },
      };
      let msg = d.kind.display(&result.semantic.type_arena);
      let err = LxError::type_err(format!("{prefix}: {msg}"), d.span, d.kind.help(&result.semantic.type_arena));
      let named = NamedSource::new(path, source.clone());
      let report = Report::new(err).with_source_code(named);
      eprintln!("{report:?}");
      print_fix(d);
    }
    let fail_count = if strict { errors + warnings } else { errors };
    if fail_count > 0 { ExitCode::from(1) } else { ExitCode::SUCCESS }
  }
}

pub fn check_workspace(member_filter: Option<&str>, strict: bool) -> ExitCode {
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
  let members: Vec<&manifest::Member> = if let Some(filter) = member_filter {
    let found: Vec<_> = ws.members.iter().filter(|m| m.name == filter).collect();
    if found.is_empty() {
      eprintln!("error: no member named '{filter}'");
      eprintln!("available: {}", ws.members.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", "));
      return ExitCode::from(1);
    }
    found
  } else {
    ws.members.iter().collect()
  };

  let mut total_ok = 0u32;
  let mut total_err = 0u32;
  let mut total_parse_err = 0u32;
  let mut any_failure = false;

  for member in &members {
    let files = collect_lx_files(&member.dir);
    let mut member_ok = 0u32;
    let mut member_err = 0u32;
    let mut member_parse_err = 0u32;
    for file in &files {
      let path_str = file.display().to_string();
      match run::read_and_parse(&path_str) {
        Ok((source, program)) => {
          let source_arc: Arc<str> = Arc::from(source.as_str());
          let result = check(&program, source_arc);
          let file_errors: u32 = result.diagnostics.iter().filter(|d| d.level == DiagLevel::Error || (strict && d.level == DiagLevel::Warning)).count() as u32;
          if file_errors == 0 && result.diagnostics.is_empty() {
            member_ok += 1;
          } else if file_errors == 0 {
            member_ok += 1;
            print_diagnostics(&result, &path_str, &source, "warning");
          } else {
            member_err += 1;
            print_all_diagnostics(&result, &path_str, &source);
          }
        },
        Err(_) => {
          member_parse_err += 1;
          eprintln!("  parse error: {path_str}");
        },
      }
    }
    let status = if member_err > 0 { "FAIL" } else { "ok" };
    let total_files = member_ok + member_err + member_parse_err;
    if member_parse_err > 0 {
      println!("{:<16} {} checked, {} type errors, {} parse errors — {status}", member.name, total_files, member_err, member_parse_err);
    } else {
      println!("{:<16} {} checked, {} errors — {status}", member.name, total_files, member_err);
    }
    total_ok += member_ok;
    total_err += member_err;
    total_parse_err += member_parse_err;
    if member_err > 0 {
      any_failure = true;
    }
  }

  if total_parse_err > 0 {
    println!(
      "\nTOTAL: {} files, {} type errors, {} parse errors, {} members",
      total_ok + total_err + total_parse_err,
      total_err,
      total_parse_err,
      members.len()
    );
  } else {
    println!("\nTOTAL: {} files, {} errors, {} members", total_ok + total_err, total_err, members.len());
  }
  if any_failure { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

fn print_diagnostics(result: &CheckResult, path_str: &str, source: &str, prefix: &str) {
  for d in &result.diagnostics {
    let msg = d.kind.display(&result.semantic.type_arena);
    let err = LxError::type_err(format!("{prefix}: {msg}"), d.span, d.kind.help(&result.semantic.type_arena));
    let named = NamedSource::new(path_str, source.to_string());
    let report = Report::new(err).with_source_code(named);
    eprintln!("{report:?}");
    print_fix(d);
  }
}

fn print_all_diagnostics(result: &CheckResult, path_str: &str, source: &str) {
  for d in &result.diagnostics {
    let prefix = match d.level {
      DiagLevel::Error => "error",
      DiagLevel::Warning => "warning",
    };
    let msg = d.kind.display(&result.semantic.type_arena);
    let err = LxError::type_err(format!("{prefix}: {msg}"), d.span, d.kind.help(&result.semantic.type_arena));
    let named = NamedSource::new(path_str, source.to_string());
    let report = Report::new(err).with_source_code(named);
    eprintln!("{report:?}");
    print_fix(d);
  }
}

fn print_fix(d: &Diagnostic) {
  if let Some(fix) = &d.fix {
    eprintln!("  fix: {} ({})", fix.description, fix.applicability);
  }
}

fn collect_lx_files(dir: &Path) -> Vec<PathBuf> {
  let mut files = Vec::new();
  collect_lx_files_rec(dir, &mut files);
  files.sort();
  files
}

fn collect_lx_files_rec(dir: &Path, files: &mut Vec<PathBuf>) {
  let Ok(read_dir) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in read_dir.filter_map(|e| match e {
    Ok(entry) => Some(entry),
    Err(err) => {
      eprintln!("warning: failed to read directory entry in {}: {err}", dir.display());
      None
    },
  }) {
    let path = entry.path();
    if path.is_dir() {
      collect_lx_files_rec(&path, files);
    } else if path.extension().and_then(|e| e.to_str()) == Some("lx") {
      files.push(path);
    }
  }
}
