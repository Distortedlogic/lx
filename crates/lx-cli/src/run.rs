use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use lx::prelude::*;

pub fn run(source: &str, filename: &str, ctx: &Arc<RuntimeCtx>, control_spec: Option<&str>) -> Result<(), Vec<LxError>> {
  let (tokens, comments) = lex(source).map_err(|e| vec![LxError::from(e)])?;
  let result = parse(tokens, FileId::new(0), comments, source);
  let lx_errors: Vec<LxError> = result.errors.iter().cloned().map(LxError::from).collect();
  let surface = result.program.ok_or(lx_errors.clone())?;
  if !lx_errors.is_empty() {
    for e in &lx_errors {
      eprintln!("parse warning: {e}");
    }
  }
  let program = desugar(surface);
  let source_dir = Path::new(filename).parent().map(|p| p.to_path_buf());
  let mut interp = Interpreter::new(source, source_dir, Arc::clone(ctx));
  let local = tokio::task::LocalSet::new();
  ctx.tokio_runtime.block_on(local.run_until(async {
    if let Some(spec) = control_spec {
      let state = std::sync::Arc::new(lx_eval::runtime::ControlChannelState {
        global_pause: std::sync::Arc::clone(&ctx.global_pause),
        cancel_flag: std::sync::Arc::clone(&ctx.cancel_flag),
        inject_tx: ctx.inject_tx.clone(),
        event_stream: std::sync::Arc::clone(&ctx.event_stream),
      });
      let state_clone = std::sync::Arc::clone(&state);
      let spec_owned = spec.to_string();
      tokio::spawn(async move {
        if spec_owned == "stdin" {
          lx_eval::runtime::control_stdin::run_stdin_control(state_clone).await;
        } else if let Some(addr) = spec_owned.strip_prefix("ws://") {
          lx_eval::runtime::control_ws::run_ws_control(addr.to_string(), state_clone).await;
        } else if let Some(addr) = spec_owned.strip_prefix("tcp://") {
          lx_eval::runtime::control_tcp::run_tcp_control(addr.to_string(), state_clone).await;
        } else {
          eprintln!("[control] unknown transport: {spec_owned}");
        }
      });
    }

    interp.load_default_tools().await.map_err(|e| vec![e])?;
    interp.load_declared_tools().await.map_err(|e| vec![e])?;
    match interp.exec(&program).await {
      Ok(val) => {
        if !matches!(val, lx_value::LxVal::Unit) {
          println!("{val}");
        }
        Ok(())
      },
      Err(e) => Err(vec![e]),
    }
  }))
}

pub fn read_and_parse(path: &str) -> Result<(String, lx_ast::ast::Program<lx_ast::ast::Core>), ExitCode> {
  let source = fs::read_to_string(path).map_err(|e| {
    eprintln!("error: cannot read {path}: {e}");
    ExitCode::from(1)
  })?;
  let (tokens, comments) = lex(&source).map_err(|e| {
    let named = miette::NamedSource::new(path, source.clone());
    eprintln!("{:?}", miette::Report::new(e).with_source_code(named));
    ExitCode::from(1)
  })?;
  let result = parse(tokens, FileId::new(0), comments, &source);
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
  let program = desugar(surface);
  Ok((source, program))
}
