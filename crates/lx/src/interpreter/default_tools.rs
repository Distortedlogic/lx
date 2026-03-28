use std::sync::Arc;

use crate::error::{EvalSignal, LxError};
use crate::folder::desugar;
use crate::parser::parse;
use crate::source::FileId;

use super::Interpreter;

const DEFAULT_TOOL_SOURCES: &[&str] =
  &["tools/bash", "tools/read", "tools/write", "tools/edit", "tools/glob", "tools/grep", "tools/web_search", "tools/web_fetch"];

impl Interpreter {
  pub async fn load_default_tools(&mut self) -> Result<(), LxError> {
    let saved_arena = Arc::clone(&self.arena);
    for &module_name in DEFAULT_TOOL_SOURCES {
      let Some(source) = crate::stdlib::lx_std_module_source(module_name) else { continue };
      let span = miette::SourceSpan::from(0..0);
      let (tokens, comments) = crate::lexer::lex(source).map_err(|e| LxError::runtime(format!("std/{module_name}: {e}"), span))?;
      let result = parse(tokens, FileId::new(0), comments, source);
      let surface = result.program.ok_or_else(|| LxError::runtime(format!("std/{module_name}: parse error"), span))?;
      let program = desugar(surface);
      self.arena = Arc::new(program.arena.clone());
      let stmts = program.stmts.clone();
      for sid in &stmts {
        self.eval_stmt(*sid).await.map_err(|e| match e {
          EvalSignal::Error(e) => e,
          EvalSignal::Break(_) => LxError::runtime("break outside loop", span),
          EvalSignal::AgentStop => LxError::runtime("agent stopped", span),
        })?;
      }
    }
    self.arena = saved_arena;
    Ok(())
  }
}
