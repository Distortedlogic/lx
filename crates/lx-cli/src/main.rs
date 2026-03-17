use std::io::{BufRead, IsTerminal, Write};
use std::path::Path;
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
        #[arg(long, help = "Output diagnostics as JSON")]
        json: bool,
    },
    Test {
        #[arg(help = "Directory containing .lx test files")]
        dir: String,
    },
    Check {
        file: String,
    },
    Agent {
        #[arg(help = "Agent script file (must evaluate to a handler function)")]
        script: String,
    },
    Diagram {
        file: String,
        #[arg(short, long, help = "Write Mermaid output to file instead of stdout")]
        output: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Run { file, json } => run_file(&file, json),
        Command::Check { file } => check_file(&file),
        Command::Test { dir } => run_tests(&dir),
        Command::Agent { script } => run_agent(&script),
        Command::Diagram { file, output } => run_diagram(&file, output.as_deref()),
    }
}

fn run_file(path: &str, _json: bool) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            return ExitCode::from(1);
        }
    };
    let ctx = if std::io::stdin().is_terminal() {
        Arc::new(RuntimeCtx {
            user: Arc::new(StdinStdoutUserBackend),
            ..RuntimeCtx::default()
        })
    } else {
        Arc::new(RuntimeCtx::default())
    };
    match run(&source, path, ctx) {
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
    let (source, program) = match read_and_parse(path) {
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

fn run_tests(dir: &str) -> ExitCode {
    let mut entries: Vec<TestEntry> = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| {
        eprintln!("error: cannot read directory {dir}: {e}");
        std::process::exit(1);
    }) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("lx") {
            let name = path
                .file_name()
                .expect("test file must have name")
                .to_string_lossy()
                .to_string();
            entries.push(TestEntry { name, path });
        } else if path.is_dir() {
            let main_lx = path.join("main.lx");
            if main_lx.exists() {
                let name = path
                    .file_name()
                    .expect("dir must have name")
                    .to_string_lossy()
                    .to_string();
                entries.push(TestEntry {
                    name,
                    path: main_lx,
                });
            }
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    let mut passed = 0;
    let mut failed = 0;
    let mut fail_details = Vec::new();
    for entry in &entries {
        let source = match std::fs::read_to_string(&entry.path) {
            Ok(s) => s,
            Err(e) => {
                println!("SKIP {}: {e}", entry.name);
                continue;
            }
        };
        let ctx = Arc::new(RuntimeCtx::default());
        match run(&source, entry.path.to_str().unwrap_or(&entry.name), ctx) {
            Ok(()) => {
                println!("PASS {}", entry.name);
                passed += 1;
            }
            Err(errors) => {
                let named = miette::NamedSource::new(&entry.name, source.clone());
                let first = &errors[0];
                let line = format!("{first}");
                println!("FAIL {}: {line}", entry.name);
                failed += 1;
                fail_details.push((entry.name.clone(), errors, named));
            }
        }
    }
    println!(
        "\n{passed} passed, {failed} failed, {} total",
        passed + failed
    );
    if !fail_details.is_empty() {
        println!("\n--- failures ---");
        for (name, errors, named) in &fail_details {
            println!("\n{name}:");
            for err in errors {
                let report = miette::Report::new(err.clone()).with_source_code(named.clone());
                eprintln!("{report:?}");
            }
        }
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

struct TestEntry {
    name: String,
    path: std::path::PathBuf,
}

fn run(source: &str, filename: &str, ctx: Arc<RuntimeCtx>) -> Result<(), Vec<lx::error::LxError>> {
    let tokens = lx::lexer::lex(source).map_err(|e| vec![e])?;
    let program = lx::parser::parse(tokens).map_err(|e| vec![e])?;
    let source_dir = Path::new(filename).parent().map(|p| p.to_path_buf());
    let mut interp = lx::interpreter::Interpreter::new(source, source_dir, ctx);
    match interp.exec(&program) {
        Ok(val) => {
            if !matches!(val, lx::value::Value::Unit) {
                println!("{val}");
            }
            Ok(())
        }
        Err(e) => Err(vec![e]),
    }
}

fn run_diagram(path: &str, output: Option<&str>) -> ExitCode {
    let (_source, program) = match read_and_parse(path) {
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

fn read_and_parse(path: &str) -> Result<(String, lx::ast::Program), ExitCode> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        eprintln!("error: cannot read {path}: {e}");
        ExitCode::from(1)
    })?;
    let tokens = lx::lexer::lex(&source).map_err(|e| {
        let named = miette::NamedSource::new(path, source.clone());
        eprintln!("{:?}", miette::Report::new(e).with_source_code(named));
        ExitCode::from(1)
    })?;
    let program = lx::parser::parse(tokens).map_err(|e| {
        let named = miette::NamedSource::new(path, source.clone());
        eprintln!("{:?}", miette::Report::new(e).with_source_code(named));
        ExitCode::from(1)
    })?;
    Ok((source, program))
}

fn run_agent(script_path: &str) -> ExitCode {
    let source = match std::fs::read_to_string(script_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("agent error: cannot read {script_path}: {e}");
            return ExitCode::from(1);
        }
    };
    let tokens = match lx::lexer::lex(&source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("agent error: {e}");
            return ExitCode::from(1);
        }
    };
    let program = match lx::parser::parse(tokens) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("agent error: {e}");
            return ExitCode::from(1);
        }
    };
    let source_dir = Path::new(script_path).parent().map(|p| p.to_path_buf());
    let ctx = Arc::new(lx::backends::RuntimeCtx::default());
    let mut interp = lx::interpreter::Interpreter::new(&source, source_dir, ctx);
    let handler = match interp.exec(&program) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("agent error: {e}");
            return ExitCode::from(1);
        }
    };
    let stdin = std::io::stdin();
    let reader = std::io::BufReader::new(stdin.lock());
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let json_val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "{}",
                    serde_json::json!({"__err": format!("JSON decode: {e}")})
                );
                continue;
            }
        };
        let msg = lx::stdlib::json_conv::json_to_lx(json_val);
        match interp.call(handler.clone(), msg) {
            Ok(result) => {
                let result_json =
                    lx::stdlib::json_conv::lx_to_json(&result, lx::span::Span::default());
                match result_json {
                    Ok(j) => println!("{}", serde_json::to_string(&j).unwrap_or_default()),
                    Err(e) => println!("{}", serde_json::json!({"__err": format!("{e}")})),
                }
            }
            Err(e) => println!("{}", serde_json::json!({"__err": format!("{e}")})),
        }
        if let Err(e) = std::io::stdout().flush() {
            eprintln!("agent: stdout flush failed: {e}");
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}
