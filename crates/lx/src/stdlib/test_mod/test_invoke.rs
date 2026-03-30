use std::sync::Arc;

use crate::BuiltinCtx;
use crate::ast::{BindTarget, Program, Stmt};
use crate::error::{EvalSignal, LxError};
use crate::folder::desugar;
use crate::interpreter::Interpreter;
use crate::lexer::lex;
use crate::parser::parse;
use crate::runtime::RuntimeCtx;
use crate::source::FileId;
use crate::sym::intern;
use crate::value::LxVal;
use miette::SourceSpan;

const ENTRY_RUN: &str = "run";
const ENTRY_MAIN: &str = "main";

pub(super) fn invoke_flow(flow_path: &str, input: &LxVal, ctx: &Arc<dyn BuiltinCtx>, span: SourceSpan) -> Result<LxVal, LxError> {
  let path = if flow_path.starts_with("./") || flow_path.starts_with("../") {
    if let Some(ref dir) = ctx.source_dir() { dir.join(flow_path) } else { std::path::PathBuf::from(flow_path) }
  } else {
    std::path::PathBuf::from(flow_path)
  };
  let source = std::fs::read_to_string(&path).map_err(|e| LxError::runtime(format!("test.run: cannot read flow '{flow_path}': {e}"), span))?;
  let (tokens, comments) = lex(&source).map_err(|e| LxError::runtime(format!("test.run: lex error in '{flow_path}': {e}"), span))?;
  let result = parse(tokens, FileId::new(0), comments, &source);
  let surface = result.program.ok_or_else(|| {
    let msgs: Vec<String> = result.errors.iter().map(|e| format!("{e}")).collect();
    LxError::runtime(format!("test.run: parse errors in '{flow_path}': {}", msgs.join("; ")), span)
  })?;
  if !result.errors.is_empty() {
    let msgs: Vec<String> = result.errors.iter().map(|e| format!("{e}")).collect();
    eprintln!("test.run: parse warnings in '{flow_path}': {}", msgs.join("; "));
  }
  let program = desugar(surface);
  let module_dir = path.parent().map(|p| p.to_path_buf());
  let rtx = crate::builtins::extract_runtime_ctx(ctx.as_ref());
  let rtx_arc = Arc::new(RuntimeCtx {
    event_stream: Arc::clone(ctx.event_stream()),
    source_dir: parking_lot::Mutex::new(ctx.source_dir()),
    network_denied: ctx.network_denied(),
    test_threshold: ctx.test_threshold(),
    test_runs: ctx.test_runs(),
    workspace_members: rtx.workspace_members.clone(),
    dep_dirs: rtx.dep_dirs.clone(),
    tokio_runtime: Arc::clone(&rtx.tokio_runtime),
    ..RuntimeCtx::default()
  });
  let mut interp = Interpreter::new(&source, module_dir, rtx_arc);
  tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
      interp.load_default_tools().await.map_err(|e| LxError::runtime(format!("test.run: tool init error in '{flow_path}': {e}"), span))?;
      interp.exec(&program).await.map_err(|e| LxError::runtime(format!("test.run: exec error in '{flow_path}': {e}"), span))?;

      let entry_name =
        find_flow_entry_name(&program).ok_or_else(|| LxError::runtime(format!("test.run: flow '{flow_path}' must export +run or +main"), span))?;
      let entry = interp
        .env
        .get(intern(&entry_name))
        .ok_or_else(|| LxError::runtime(format!("test.run: flow '{flow_path}' exported +{entry_name} not found in env"), span))?;
      interp.apply_func(entry, input.clone(), span).await.map_err(|e| match e {
        EvalSignal::Error(e) => e,
        EvalSignal::Break(_) => LxError::runtime("break outside loop", span),
        EvalSignal::AgentStop => LxError::runtime("agent stopped", span),
      })
    })
  })
}

fn find_flow_entry_name<P>(program: &Program<P>) -> Option<String> {
  let mut has_run = false;
  let mut has_main = false;
  for &sid in &program.stmts {
    if let Stmt::Binding(b) = program.arena.stmt(sid)
      && b.exported
      && let BindTarget::Name(ref name) = b.target
    {
      if name == ENTRY_RUN {
        has_run = true;
      } else if name == ENTRY_MAIN {
        has_main = true;
      }
    }
  }
  if has_run {
    Some(ENTRY_RUN.into())
  } else if has_main {
    Some(ENTRY_MAIN.into())
  } else {
    None
  }
}
