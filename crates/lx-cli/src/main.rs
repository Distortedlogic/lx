use std::path::Path;
use std::process::ExitCode;

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
}

fn main() -> ExitCode {
  let cli = Cli::parse();
  match cli.command {
    Command::Run { file, json } => run_file(&file, json),
    Command::Test { dir } => run_tests(&dir),
  }
}

fn run_file(path: &str, _json: bool) -> ExitCode {
  let source = match std::fs::read_to_string(path) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {path}: {e}");
      return ExitCode::from(1);
    },
  };
  match run(&source, path) {
    Ok(()) => ExitCode::SUCCESS,
    Err(errors) => {
      let named = miette::NamedSource::new(path, source.clone());
      for err in errors {
        let report = miette::Report::new(err).with_source_code(named.clone());
        eprintln!("{report:?}");
      }
      ExitCode::from(1)
    },
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
      let name = path.file_name().expect("test file must have name").to_string_lossy().to_string();
      entries.push(TestEntry { name, path });
    } else if path.is_dir() {
      let main_lx = path.join("main.lx");
      if main_lx.exists() {
        let name = path.file_name().expect("dir must have name").to_string_lossy().to_string();
        entries.push(TestEntry { name, path: main_lx });
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
      },
    };
    match run(&source, entry.path.to_str().unwrap_or(&entry.name)) {
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
  println!("\n{passed} passed, {failed} failed, {} total", passed + failed);
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

fn run(source: &str, filename: &str) -> Result<(), Vec<lx::error::LxError>> {
  let tokens = lx::lexer::lex(source).map_err(|e| vec![e])?;
  let program = lx::parser::parse(tokens).map_err(|e| vec![e])?;
  let source_dir = Path::new(filename).parent().map(|p| p.to_path_buf());
  let mut interp = lx::interpreter::Interpreter::new(source, source_dir);
  match interp.exec(&program) {
    Ok(val) => {
      if !matches!(val, lx::value::Value::Unit) {
        println!("{val}");
      }
      Ok(())
    },
    Err(e) => Err(vec![e]),
  }
}
