use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use lx::checker::diagnostics::Applicability;
use lx::checker::{CheckResult, DiagLevel, Diagnostic, check};
use lx::error::LxError;
use lx::folder::desugar;
use lx::lexer::lex;
use lx::parser::parse;
use miette::{NamedSource, Report};

use crate::{manifest, run};

enum FixOutcome {
  Applied(Box<CheckResult>, String),
  NoFixes,
  WriteFailed(io::Error),
  RecheckFailed,
}

pub fn check_file(path: &str, strict: bool, fix: bool) -> ExitCode {
  let (source, program) = match run::read_and_parse(path) {
    Ok(sp) => sp,
    Err(code) => return code,
  };
  let source_arc: Arc<str> = Arc::from(source.as_str());
  let result = check(&program, source_arc);

  if fix && let Some(fixed_source) = apply_fixes(&source, &result.diagnostics) {
    if let Err(e) = fs::write(path, &fixed_source) {
      eprintln!("error: cannot write {path}: {e}");
      return ExitCode::from(1);
    }
    eprintln!("applied fixes to {path}");
    match recheck_source(&fixed_source) {
      Ok(recheck_result) => return print_and_exit(&recheck_result, path, &fixed_source, strict),
      Err(detail) => {
        eprintln!("warning: fix produced invalid syntax in {path}, reverting ({detail})");
        if let Err(e) = fs::write(path, &source) {
          eprintln!("error: cannot revert {path}: {e}");
        }
        return ExitCode::from(1);
      },
    }
  }

  if result.diagnostics.is_empty() {
    println!("ok: {path}");
    ExitCode::SUCCESS
  } else {
    print_and_exit(&result, path, &source, strict)
  }
}

fn recheck_source(fixed_source: &str) -> Result<CheckResult, String> {
  let (tokens, comments) = lex(fixed_source).map_err(|e| format!("lex error: {e}"))?;
  let parse_result = parse(tokens, lx::source::FileId::new(0), comments, fixed_source);
  let surface = parse_result.program.ok_or_else(|| {
    let msgs: Vec<String> = parse_result.errors.iter().map(|e| format!("{e}")).collect();
    format!("parse errors: {}", msgs.join("; "))
  })?;
  let program = desugar(surface);
  let fixed_arc: Arc<str> = Arc::from(fixed_source);
  Ok(check(&program, fixed_arc))
}

fn print_and_exit(result: &CheckResult, path: &str, source: &str, strict: bool) -> ExitCode {
  print_diagnostics(result, path, source, None);
  if count_errors(result, strict) > 0 { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

fn try_apply_fixes(path: &str, source: &str, result: &CheckResult) -> FixOutcome {
  let Some(fixed_source) = apply_fixes(source, &result.diagnostics) else {
    return FixOutcome::NoFixes;
  };
  if let Err(e) = fs::write(path, &fixed_source) {
    return FixOutcome::WriteFailed(e);
  }
  eprintln!("applied fixes to {path}");
  match recheck_source(&fixed_source) {
    Ok(recheck_result) => FixOutcome::Applied(Box::new(recheck_result), fixed_source),
    Err(detail) => {
      eprintln!("warning: fix produced invalid syntax in {path}, reverting ({detail})");
      if let Err(e) = fs::write(path, source) {
        eprintln!("error: cannot revert {path}: {e}");
      }
      FixOutcome::RecheckFailed
    },
  }
}

fn count_errors(result: &CheckResult, strict: bool) -> u32 {
  result.diagnostics.iter().filter(|d| d.level == DiagLevel::Error || (strict && d.level == DiagLevel::Warning)).count() as u32
}

pub fn check_workspace(member_filter: Option<&str>, strict: bool, fix: bool) -> ExitCode {
  let cwd = match env::current_dir() {
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

  let mut total_ok = 0u32;
  let mut total_err = 0u32;
  let mut total_parse_err = 0u32;
  let mut total_fixed = 0u32;
  let mut any_failure = false;

  for member in &members {
    let files = collect_lx_files(&member.dir);
    let mut member_ok = 0u32;
    let mut member_err = 0u32;
    let mut member_parse_err = 0u32;
    let mut member_fixed = 0u32;
    for file in &files {
      let path_str = file.display().to_string();
      match run::read_and_parse(&path_str) {
        Ok((source, program)) => {
          let source_arc: Arc<str> = Arc::from(source.as_str());
          let result = check(&program, source_arc);
          let (final_result, final_source) = if fix {
            match try_apply_fixes(&path_str, &source, &result) {
              FixOutcome::Applied(r, s) => {
                member_fixed += 1;
                (*r, s)
              },
              FixOutcome::NoFixes => (result, source),
              FixOutcome::WriteFailed(e) => {
                eprintln!("error: cannot write {path_str}: {e}");
                any_failure = true;
                continue;
              },
              FixOutcome::RecheckFailed => {
                any_failure = true;
                continue;
              },
            }
          } else {
            (result, source)
          };
          let file_errors = count_errors(&final_result, strict);
          if file_errors == 0 && final_result.diagnostics.is_empty() {
            member_ok += 1;
          } else if file_errors == 0 {
            member_ok += 1;
            print_diagnostics(&final_result, &path_str, &final_source, Some("warning"));
          } else {
            member_err += 1;
            print_diagnostics(&final_result, &path_str, &final_source, None);
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
    if member_fixed > 0 {
      println!("{:<16} {total_files} checked, {member_fixed} fixed, {member_err} remaining errors — {status}", member.pkg.name);
    } else if member_parse_err > 0 {
      println!("{:<16} {total_files} checked, {member_err} type errors, {member_parse_err} parse errors — {status}", member.pkg.name);
    } else {
      println!("{:<16} {total_files} checked, {member_err} errors — {status}", member.pkg.name);
    }
    total_ok += member_ok;
    total_err += member_err;
    total_parse_err += member_parse_err;
    total_fixed += member_fixed;
    if member_err > 0 {
      any_failure = true;
    }
  }

  if total_fixed > 0 {
    println!("\nTOTAL: {} files, {} fixed, {} errors, {} members", total_ok + total_err + total_parse_err, total_fixed, total_err, members.len());
  } else if total_parse_err > 0 {
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

fn print_diagnostics(result: &CheckResult, path_str: &str, source: &str, prefix_override: Option<&str>) {
  for d in &result.diagnostics {
    let prefix = prefix_override.unwrap_or(match d.level {
      DiagLevel::Error => "error",
      DiagLevel::Warning => "warning",
    });
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

fn apply_fixes(source: &str, diagnostics: &[Diagnostic]) -> Option<String> {
  let mut edits: Vec<(usize, usize, &str)> = Vec::new();
  for diag in diagnostics {
    if let Some(ref fix) = diag.fix
      && fix.applicability == Applicability::MachineApplicable
    {
      for edit in &fix.edits {
        let start = edit.range.offset();
        let end = start + edit.range.len();
        edits.push((start, end, &edit.replacement));
      }
    }
  }

  if edits.is_empty() {
    return None;
  }

  edits.sort_by(|a, b| b.0.cmp(&a.0));

  let mut result = source.to_string();
  let mut last_start = usize::MAX;
  for (start, end, replacement) in &edits {
    if *end > last_start {
      continue;
    }
    result.replace_range(*start..*end, replacement);
    last_start = *start;
  }

  Some(result)
}

pub fn collect_lx_files(dir: &Path) -> Vec<PathBuf> {
  let mut files = Vec::new();
  collect_lx_files_rec(dir, &mut files);
  files.sort();
  files
}

fn collect_lx_files_rec(dir: &Path, files: &mut Vec<PathBuf>) {
  let Ok(read_dir) = fs::read_dir(dir) else {
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
