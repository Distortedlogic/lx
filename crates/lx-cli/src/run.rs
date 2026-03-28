use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use lx::error::LxError;
use lx::runtime::RuntimeCtx;

pub fn run(source: &str, filename: &str, ctx: &Arc<RuntimeCtx>, control_spec: Option<&str>) -> Result<(), Vec<LxError>> {
  let (tokens, comments) = lx::lexer::lex(source).map_err(|e| vec![e])?;
  let result = lx::parser::parse(tokens, lx::source::FileId::new(0), comments, source);
  let surface = result.program.ok_or(result.errors.clone())?;
  if !result.errors.is_empty() {
    for e in &result.errors {
      eprintln!("parse warning: {e}");
    }
  }
  let program = lx::folder::desugar(surface);
  let source_dir = Path::new(filename).parent().map(|p| p.to_path_buf());
  let mut interp = lx::interpreter::Interpreter::new(source, source_dir, Arc::clone(ctx));
  ctx.tokio_runtime.block_on(async {
    if let Some(spec) = control_spec {
      let state = std::sync::Arc::new(lx::runtime::ControlChannelState {
        global_pause: std::sync::Arc::clone(&ctx.global_pause),
        cancel_flag: std::sync::Arc::clone(&ctx.cancel_flag),
        inject_tx: ctx.inject_tx.clone(),
      });
      let state_clone = std::sync::Arc::clone(&state);
      let spec_owned = spec.to_string();
      tokio::spawn(async move {
        if spec_owned == "stdin" {
          lx::runtime::control_stdin::run_stdin_control(state_clone).await;
        } else if let Some(addr) = spec_owned.strip_prefix("ws://") {
          lx::runtime::control_ws::run_ws_control(addr.to_string(), state_clone).await;
        } else if let Some(addr) = spec_owned.strip_prefix("tcp://") {
          lx::runtime::control_tcp::run_tcp_control(addr.to_string(), state_clone).await;
        } else {
          eprintln!("[control] unknown transport: {spec_owned}");
        }
      });
    }

    interp.load_default_tools().await.map_err(|e| vec![e])?;
    match interp.exec(&program).await {
      Ok(val) => {
        if !matches!(val, lx::value::LxVal::Unit) {
          println!("{val}");
        }
        Ok(())
      },
      Err(e) => Err(vec![e]),
    }
  })
}

pub fn read_and_parse(path: &str) -> Result<(String, lx::ast::Program<lx::ast::Core>), ExitCode> {
  let source = fs::read_to_string(path).map_err(|e| {
    eprintln!("error: cannot read {path}: {e}");
    ExitCode::from(1)
  })?;
  let (tokens, comments) = lx::lexer::lex(&source).map_err(|e| {
    let named = miette::NamedSource::new(path, source.clone());
    eprintln!("{:?}", miette::Report::new(e).with_source_code(named));
    ExitCode::from(1)
  })?;
  let result = lx::parser::parse(tokens, lx::source::FileId::new(0), comments, &source);
  let surface = result.program.ok_or_else(|| {
    for e in &result.errors {
      let named = miette::NamedSource::new(path, source.clone());
      eprintln!("{:?}", miette::Report::new(e.clone()).with_source_code(named));
    }
    ExitCode::from(1)
  })?;
  if !result.errors.is_empty() {
    for e in &result.errors {
      let named = miette::NamedSource::new(path, source.clone());
      eprintln!("parse warning: {:?}", miette::Report::new(e.clone()).with_source_code(named));
    }
  }
  let program = lx::folder::desugar(surface);
  Ok((source, program))
}
