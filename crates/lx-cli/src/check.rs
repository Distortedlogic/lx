use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::manifest;
use crate::run;

pub fn check_file(path: &str) -> ExitCode {
    let (source, program) = match run::read_and_parse(path) {
        Ok(sp) => sp,
        Err(code) => return code,
    };
    let result = lx::checker::check(&program);
    if result.diagnostics.is_empty() {
        println!("ok: {path}");
        ExitCode::SUCCESS
    } else {
        for d in &result.diagnostics {
            let err = lx::error::LxError::type_err(&d.msg, d.span);
            let named = miette::NamedSource::new(path, source.clone());
            let report = miette::Report::new(err).with_source_code(named);
            eprintln!("{report:?}");
        }
        ExitCode::from(1)
    }
}

pub fn check_workspace(member_filter: Option<&str>) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: cannot determine cwd: {e}");
            return ExitCode::from(1);
        }
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
            eprintln!(
                "available: {}",
                ws.members
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return ExitCode::from(1);
        }
        found
    } else {
        ws.members.iter().collect()
    };

    let mut total_ok = 0u32;
    let mut total_err = 0u32;
    let mut any_failure = false;

    for member in &members {
        let files = collect_lx_files(&member.dir);
        let mut member_ok = 0u32;
        let mut member_err = 0u32;
        for file in &files {
            let path_str = file.display().to_string();
            match run::read_and_parse(&path_str) {
                Ok((source, program)) => {
                    let result = lx::checker::check(&program);
                    if result.diagnostics.is_empty() {
                        member_ok += 1;
                    } else {
                        member_err += 1;
                        for d in &result.diagnostics {
                            let err = lx::error::LxError::type_err(&d.msg, d.span);
                            let named = miette::NamedSource::new(path_str.clone(), source.clone());
                            let report = miette::Report::new(err).with_source_code(named);
                            eprintln!("{report:?}");
                        }
                    }
                }
                Err(_) => {
                    member_err += 1;
                }
            }
        }
        let status = if member_err > 0 { "FAIL" } else { "ok" };
        println!(
            "{:<16} {} checked, {} errors — {status}",
            member.name,
            member_ok + member_err,
            member_err
        );
        total_ok += member_ok;
        total_err += member_err;
        if member_err > 0 {
            any_failure = true;
        }
    }

    println!(
        "\nTOTAL: {} files, {} errors, {} members",
        total_ok + total_err,
        total_err,
        members.len()
    );
    if any_failure {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
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
    for entry in read_dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_lx_files_rec(&path, files);
        } else if path.extension().and_then(|e| e.to_str()) == Some("lx") {
            files.push(path);
        }
    }
}
