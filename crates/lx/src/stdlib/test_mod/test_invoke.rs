use std::sync::Arc;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

pub(super) fn invoke_flow(flow_path: &str, input: &LxVal, ctx: &Arc<RuntimeCtx>, span: SourceSpan) -> Result<LxVal, LxError> {
  let path = if flow_path.starts_with("./") || flow_path.starts_with("../") {
    if let Some(ref dir) = *ctx.source_dir.lock() { dir.join(flow_path) } else { std::path::PathBuf::from(flow_path) }
  } else {
    std::path::PathBuf::from(flow_path)
  };
  let source = std::fs::read_to_string(&path).map_err(|e| LxError::runtime(format!("test.run: cannot read flow '{flow_path}': {e}"), span))?;
  let tokens = crate::lexer::lex(&source).map_err(|e| LxError::runtime(format!("test.run: lex error in '{flow_path}': {e}"), span))?;
  let surface = crate::parser::parse(tokens).map_err(|e| LxError::runtime(format!("test.run: parse error in '{flow_path}': {e}"), span))?;
  let program = crate::folder::desugar(surface);
  let module_dir = path.parent().map(|p| p.to_path_buf());
  let mut interp = crate::interpreter::Interpreter::new(&source, module_dir, Arc::clone(ctx));
  tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
      interp.exec(&program).await.map_err(|e| LxError::runtime(format!("test.run: exec error in '{flow_path}': {e}"), span))?;

      let entry_name =
        find_flow_entry_name(&program).ok_or_else(|| LxError::runtime(format!("test.run: flow '{flow_path}' must export +run or +main"), span))?;
      let entry = interp
        .env
        .get(crate::sym::intern(&entry_name))
        .ok_or_else(|| LxError::runtime(format!("test.run: flow '{flow_path}' exported +{entry_name} not found in env"), span))?;
      interp.apply_func(entry, input.clone(), span).await
    })
  })
}

fn find_flow_entry_name<P>(program: &crate::ast::Program<P>) -> Option<String> {
  use crate::ast::{BindTarget, Stmt};
  let mut has_run = false;
  let mut has_main = false;
  for &sid in &program.stmts {
    if let Stmt::Binding(b) = program.arena.stmt(sid)
      && b.exported
      && let BindTarget::Name(ref name) = b.target
    {
      if name == "run" {
        has_run = true;
      } else if name == "main" {
        has_main = true;
      }
    }
  }
  if has_run {
    Some("run".into())
  } else if has_main {
    Some("main".into())
  } else {
    None
  }
}
