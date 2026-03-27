use std::fs;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

pub fn run_agent(script_path: &str) -> ExitCode {
  let source = match fs::read_to_string(script_path) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("agent error: cannot read {script_path}: {e}");
      return ExitCode::from(1);
    },
  };
  let (tokens, comments) = match lx::lexer::lex(&source) {
    Ok(t) => t,
    Err(e) => {
      eprintln!("agent error: {e}");
      return ExitCode::from(1);
    },
  };
  let result = lx::parser::parse(tokens, lx::source::FileId::new(0), comments, &source);
  let surface = match result.program {
    Some(p) => {
      for e in &result.errors {
        eprintln!("agent warning: {e}");
      }
      p
    },
    None => {
      for e in &result.errors {
        eprintln!("agent error: {e}");
      }
      return ExitCode::from(1);
    },
  };
  let program = lx::folder::desugar(surface);
  let source_dir = Path::new(script_path).parent().map(|p| p.to_path_buf());
  let ctx = Arc::new(lx::runtime::RuntimeCtx::default());
  let mut interp = lx::interpreter::Interpreter::new(&source, source_dir, Arc::clone(&ctx));
  if let Err(e) = ctx.tokio_runtime.block_on(interp.load_default_tools()) {
    eprintln!("agent error: {e}");
    return ExitCode::from(1);
  }
  let handler = match ctx.tokio_runtime.block_on(interp.exec(&program)) {
    Ok(val) => val,
    Err(e) => {
      eprintln!("agent error: {e}");
      return ExitCode::from(1);
    },
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
        println!("{}", serde_json::json!({"__err": format!("JSON decode: {e}")}));
        continue;
      },
    };
    let msg = lx::value::LxVal::from(json_val);
    match ctx.tokio_runtime.block_on(interp.call(handler.clone(), msg)) {
      Ok(result) => {
        let j = serde_json::Value::from(&result);
        println!("{}", serde_json::to_string(&j).expect("serde_json::Value serialization is infallible"));
      },
      Err(e) => println!("{}", serde_json::json!({"__err": format!("{e}")})),
    }
    if let Err(e) = std::io::stdout().flush() {
      eprintln!("agent: stdout flush failed: {e}");
      return ExitCode::from(1);
    }
  }
  ExitCode::SUCCESS
}
