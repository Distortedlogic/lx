mod agent_cmd;
mod listing;
mod manifest;
mod run;
mod testing;

use std::io::IsTerminal;
use std::process::ExitCode;
use std::sync::Arc;

use lx::backends::{RuntimeCtx, StdinStdoutUserBackend};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lx", version, about = "The lx scripting language")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run {
        file: String,
        #[arg(long)]
        json: bool,
    },
    Test {
        #[arg()]
        dir: Option<String>,
        #[arg(short, long)]
        member: Option<String>,
    },
    Check {
        #[arg()]
        file: Option<String>,
        #[arg(short, long)]
        member: Option<String>,
    },
    Agent {
        script: String,
    },
    Diagram {
        file: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    List,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Run { file, json } => {
            let resolved = resolve_run_target(&file);
            run_file(&resolved, json)
        }
        Command::Check { file, member } => {
            if let Some(file) = file {
                check_file(&file)
            } else {
                check_workspace(member.as_deref())
            }
        }
        Command::Test { dir, member } => {
            if let Some(dir) = dir {
                testing::run_tests_dir(&dir)
            } else {
                testing::run_workspace_tests(member.as_deref())
            }
        }
        Command::Agent { script } => agent_cmd::run_agent(&script),
        Command::Diagram { file, output } => run_diagram(&file, output.as_deref()),
        Command::List => listing::list_workspace(),
    }
}

fn resolve_run_target(target: &str) -> String {
    let path = std::path::Path::new(target);
    if path.exists() && path.is_file() {
        return target.to_string();
    }
    let Ok(cwd) = std::env::current_dir() else {
        return target.to_string();
    };
    let Some(root) = manifest::find_workspace_root(&cwd) else {
        return target.to_string();
    };
    let Ok(ws) = manifest::load_workspace(&root) else {
        return target.to_string();
    };
    for member in &ws.members {
        if member.name == target {
            let entry = member.entry.as_deref().unwrap_or("main.lx");
            return member.dir.join(entry).to_string_lossy().to_string();
        }
    }
    target.to_string()
}

fn run_file(path: &str, _json: bool) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            return ExitCode::from(1);
        }
    };
    let ws_members = manifest::try_load_workspace_members();
    let mut ctx_val = if std::io::stdin().is_terminal() {
        RuntimeCtx {
            user: Arc::new(StdinStdoutUserBackend),
            ..RuntimeCtx::default()
        }
    } else {
        RuntimeCtx::default()
    };
    ctx_val.workspace_members = ws_members;
    let ctx = Arc::new(ctx_val);
    match run::run(&source, path, ctx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(errors) => {
            let named = miette::NamedSource::new(path, source.clone());
            for err in errors {
                if let lx::error::LxError::Sourced {
                    source_name,
                    source_text,
                    inner,
                } = err
                {
                    let src = miette::NamedSource::new(source_name, source_text.to_string());
                    let report = miette::Report::new(*inner).with_source_code(src);
                    eprintln!("{report:?}");
                } else {
                    let report = miette::Report::new(err).with_source_code(named.clone());
                    eprintln!("{report:?}");
                }
            }
            ExitCode::from(1)
        }
    }
}

fn check_file(path: &str) -> ExitCode {
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

fn check_workspace(member_filter: Option<&str>) -> ExitCode {
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

fn collect_lx_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_lx_files_rec(dir, &mut files);
    files.sort();
    files
}

fn collect_lx_files_rec(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
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

fn run_diagram(path: &str, output: Option<&str>) -> ExitCode {
    let (_source, program) = match run::read_and_parse(path) {
        Ok(sp) => sp,
        Err(code) => return code,
    };
    let mermaid = lx::stdlib::diag::extract_mermaid(&program);
    match output {
        Some(out_path) => {
            if let Err(e) = std::fs::write(out_path, &mermaid) {
                eprintln!("error: cannot write {out_path}: {e}");
                return ExitCode::from(1);
            }
            println!("wrote diagram to {out_path}");
        }
        None => print!("{mermaid}"),
    }
    ExitCode::SUCCESS
}
