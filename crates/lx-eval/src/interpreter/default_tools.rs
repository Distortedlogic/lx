use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use indexmap::IndexMap;
use lx_desugar::folder::desugar;
use lx_parser::parser::parse;
use lx_span::source::FileId;
use lx_span::sym::intern;
use lx_value::{EvalSignal, LxError, LxVal, mk_dyn_async};
use miette::SourceSpan;

use crate::runtime::ToolDecl;

use super::Interpreter;

const DEFAULT_TOOL_SOURCES: &[&str] =
  &["tools/bash", "tools/read", "tools/write", "tools/edit", "tools/glob", "tools/grep", "tools/web_search", "tools/web_fetch"];

impl Interpreter {
  pub async fn load_default_tools(&mut self) -> Result<(), LxError> {
    let saved_arena = Arc::clone(&self.arena);
    for &module_name in DEFAULT_TOOL_SOURCES {
      let Some(source) = crate::stdlib::lx_std_module_source(module_name) else { continue };
      let span = SourceSpan::from(0..0);
      let (tokens, comments) = lx_parser::lexer::lex(source).map_err(|e| LxError::runtime(format!("std/{module_name}: {e}"), span))?;
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

  pub async fn load_declared_tools(&mut self) -> Result<(), LxError> {
    let span = SourceSpan::from(0..0);
    let tools: Vec<(String, ToolDecl)> = self.ctx.tools.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    for (name, decl) in tools {
      match decl {
        ToolDecl::Lx { path } => {
          let resolved = if path.is_absolute() { path } else { self.source_dir.as_ref().map(|d| d.join(&path)).unwrap_or(path) };
          let exports = self.load_module(&resolved, span).await?;
          let record = LxVal::record(exports.bindings);
          let env = self.env.child();
          env.bind(intern(&name), record);
          self.env = Arc::new(env);
        },
        ToolDecl::Mcp { command } => {
          let tm = crate::tool_module::ToolModule::new(&command, &name).await.map_err(|e| LxError::runtime(format!("tool '{name}': {e}"), span))?;
          let tm_arc = Arc::new(tm);
          self.tool_modules.push(Arc::clone(&tm_arc));
          let val = self.build_mcp_tool_record(&name, &tm_arc);
          let env = self.env.child();
          env.bind(intern(&name), val);
          self.env = Arc::new(env);
        },
      }
    }
    Ok(())
  }

  fn build_mcp_tool_record(&self, alias: &str, tm: &Arc<crate::tool_module::ToolModule>) -> LxVal {
    let agent_name: Arc<str> = Arc::from(self.agent_name.as_deref().unwrap_or("main"));
    let call_counter = Arc::new(AtomicU64::new(1));
    let event_stream = Arc::clone(&self.ctx.event_stream);
    let module_name: Arc<str> = Arc::from(alias);

    let mut fields = IndexMap::new();
    fields.insert(intern("command"), LxVal::str(&tm.command));
    fields.insert(intern("alias"), LxVal::str(&tm.alias));

    let tm_call = Arc::clone(tm);
    let agent_call = Arc::clone(&agent_name);
    let counter_call = Arc::clone(&call_counter);
    let es_call = Arc::clone(&event_stream);
    let mod_call = Arc::clone(&module_name);

    let call_tool_fn = mk_dyn_async(
      "mcp_tool.call",
      2,
      Arc::new(move |args: Vec<LxVal>, call_span: SourceSpan, _ctx: Arc<dyn lx_value::BuiltinCtx>| {
        let tm = Arc::clone(&tm_call);
        let agent = Arc::clone(&agent_call);
        let counter = Arc::clone(&counter_call);
        let es = Arc::clone(&es_call);
        let module = Arc::clone(&mod_call);
        Box::pin(async move {
          let method = args[0].as_str().ok_or_else(|| LxError::runtime("mcp_tool.call: first arg must be method name (Str)", call_span))?;
          let arg = args.get(1).cloned().unwrap_or(LxVal::Unit);
          let call_id = counter.fetch_add(1, Ordering::Relaxed);

          let mut call_fields = IndexMap::new();
          call_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
          call_fields.insert(intern("tool"), LxVal::str(module.as_ref()));
          call_fields.insert(intern("method"), LxVal::str(method));
          call_fields.insert(intern("args"), arg.clone());
          es.xadd("tool/call", &agent, None, call_fields);

          let result = tm.call_tool(method, arg, &es, &agent).await;

          match result {
            Ok(val) => {
              let mut result_fields = IndexMap::new();
              result_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
              result_fields.insert(intern("tool"), LxVal::str(module.as_ref()));
              result_fields.insert(intern("method"), LxVal::str(method));
              result_fields.insert(intern("result"), val.clone());
              es.xadd("tool/result", &agent, None, result_fields);
              Ok(val)
            },
            Err(e) => {
              let err_msg = e.to_string();
              let mut error_fields = IndexMap::new();
              error_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
              error_fields.insert(intern("tool"), LxVal::str(module.as_ref()));
              error_fields.insert(intern("method"), LxVal::str(method));
              error_fields.insert(intern("error"), LxVal::str(&err_msg));
              es.xadd("tool/error", &agent, None, error_fields);
              Err(LxError::runtime(format!("mcp tool '{module}' method '{method}': {err_msg}"), call_span))
            },
          }
        })
      }),
    );
    fields.insert(intern("call"), call_tool_fn);

    LxVal::record(fields)
  }
}
