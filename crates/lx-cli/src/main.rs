use clap::{Parser, Subcommand};
use std::process::ExitCode;

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
}

fn main() -> ExitCode {
  let cli = Cli::parse();
  match cli.command {
    Command::Run { file, json } => run_file(&file, json),
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

fn run(source: &str, _filename: &str) -> Result<(), Vec<lx::error::LxError>> {
  let tokens = lx::lexer::lex(source).map_err(|e| vec![e])?;
  let program = lx::parser::parse(tokens, source).map_err(|e| vec![e])?;
  let mut interp = lx::interpreter::Interpreter::new(source);
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
