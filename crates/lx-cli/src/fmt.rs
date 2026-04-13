use std::env::current_dir;
use std::fs::{read_to_string, write};
use std::process::ExitCode;

use lx_fmt::format;
use lx_parser::lexer::lex;
use lx_parser::parser::parse;
use lx_span::source::FileId;
use miette::{NamedSource, Report};

use crate::check::collect_lx_files;
use crate::manifest::{find_manifest_root, find_workspace_root, load_workspace};

struct FmtFailed;

fn fmt_source(path_str: &str, source: &str) -> Result<String, FmtFailed> {
  let (tokens, comments) = match lex(source) {
    Ok(t) => t,
    Err(err) => {
      let named = NamedSource::new(path_str, source.to_string());
      eprintln!("{:?}", Report::new(err).with_source_code(named));
      return Err(FmtFailed);
    },
  };
  let result = parse(tokens, FileId::new(0), comments, source);
  let Some(program) = result.program else {
    for e in &result.errors {
      let named = NamedSource::new(path_str, source.to_string());
      eprintln!("{:?}", Report::new(e.clone()).with_source_code(named));
    }
    return Err(FmtFailed);
  };
  if !result.errors.is_empty() {
    for e in &result.errors {
      let named = NamedSource::new(path_str, source.to_string());
      eprintln!("parse warning: {:?}", Report::new(e.clone()).with_source_code(named));
    }
  }
  Ok(format(&program))
}

pub fn fmt_file(path: &str, check: bool) -> ExitCode {
  let source = match read_to_string(path) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {path}: {e}");
      return ExitCode::from(1);
    },
  };
  let Ok(formatted) = fmt_source(path, &source) else {
    return ExitCode::from(1);
  };
  if check {
    if formatted != source {
      eprintln!("would reformat {path}");
      return ExitCode::from(1);
    }
  } else if formatted != source {
    if let Err(e) = write(path, &formatted) {
      eprintln!("error: cannot write {path}: {e}");
      return ExitCode::from(1);
    }
    eprintln!("formatted {path}");
  }
  ExitCode::SUCCESS
}

pub fn fmt_workspace(member_filter: Option<&str>, check: bool) -> ExitCode {
  let Ok(cwd) = current_dir() else {
    eprintln!("error: cannot determine cwd");
    return ExitCode::from(1);
  };
  let root = if let Some(r) = find_workspace_root(&cwd) {
    r
  } else if let Some(r) = find_manifest_root(&cwd) {
    r
  } else {
    eprintln!("no lx.toml found");
    return ExitCode::from(1);
  };
  let ws = match load_workspace(&root) {
    Ok(w) => w,
    Err(e) => {
      eprintln!("error: failed to load workspace: {e}");
      return ExitCode::from(1);
    },
  };

  let mut grand_total = 0u32;
  let mut grand_formatted = 0u32;
  let mut grand_failed = 0u32;

  for member in ws.members.iter().filter(|m| member_filter.is_none() || member_filter == Some(m.pkg.name.as_str())) {
    let files = collect_lx_files(&member.dir);
    let mut total = 0u32;
    let mut formatted_count = 0u32;
    let mut failed = 0u32;

    for file in &files {
      total += 1;
      let path_str = file.display().to_string();
      let source = match read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
          eprintln!("error: cannot read {path_str}: {e}");
          failed += 1;
          continue;
        },
      };
      let Ok(output) = fmt_source(&path_str, &source) else {
        failed += 1;
        continue;
      };
      if output != source {
        if check {
          eprintln!("would reformat {path_str}");
          formatted_count += 1;
        } else {
          if let Err(e) = write(file, &output) {
            eprintln!("error: cannot write {path_str}: {e}");
            failed += 1;
            continue;
          }
          eprintln!("formatted {path_str}");
          formatted_count += 1;
        }
      }
    }

    eprintln!("{:<16} {} checked, {} formatted, {} failed", member.pkg.name, total, formatted_count, failed);
    grand_total += total;
    grand_formatted += formatted_count;
    grand_failed += failed;
  }

  eprintln!("\nTOTAL: {grand_total} checked, {grand_formatted} formatted, {grand_failed} failed");

  if grand_failed > 0 || (check && grand_formatted > 0) { ExitCode::from(1) } else { ExitCode::SUCCESS }
}
