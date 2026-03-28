use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use indexmap::IndexMap;
use miette::SourceSpan;

use crate::error::LxError;
use crate::sym::intern;
use crate::value::{LxVal, mk_dyn_async};

impl super::Interpreter {
  pub(super) async fn build_lx_tool_module(&mut self, command: &str, span: SourceSpan) -> Result<LxVal, LxError> {
    let file_path = self.resolve_lx_tool_path(command, span)?;
    let exports = self.load_module(&file_path, span).await?;

    let module_name: Arc<str> = Arc::from(command);
    let call_counter = Arc::new(AtomicU64::new(1));
    let mut fields = IndexMap::new();

    for (name, val) in &exports.bindings {
      if !matches!(val, LxVal::Func(_) | LxVal::MultiFunc(_) | LxVal::BuiltinFunc(_)) {
        fields.insert(*name, val.clone());
        continue;
      }

      let func_val = val.clone();
      let method_name: Arc<str> = Arc::from(name.as_str());
      let module = Arc::clone(&module_name);
      let counter = Arc::clone(&call_counter);
      let event_stream = Arc::clone(&self.ctx.event_stream);
      let ctx_ref = Arc::clone(&self.ctx);

      let dispatch = mk_dyn_async(
        "lx_tool.call",
        1,
        Arc::new(move |args: Vec<LxVal>, call_span: SourceSpan, _ctx: Arc<crate::runtime::RuntimeCtx>| {
          let func_val = func_val.clone();
          let method_name = Arc::clone(&method_name);
          let module = Arc::clone(&module);
          let counter = Arc::clone(&counter);
          let event_stream = Arc::clone(&event_stream);
          let ctx_ref = Arc::clone(&ctx_ref);
          Box::pin(async move {
            let call_id = counter.fetch_add(1, Ordering::Relaxed);
            let arg = args.into_iter().next().unwrap_or(LxVal::Unit);

            let mut call_fields = IndexMap::new();
            call_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
            call_fields.insert(intern("tool"), LxVal::str(module.as_ref()));
            call_fields.insert(intern("method"), LxVal::str(method_name.as_ref()));
            call_fields.insert(intern("args"), arg.clone());
            event_stream.xadd("tool/call", "main", None, call_fields);

            let result = crate::builtins::call_value(&func_val, arg, call_span, &ctx_ref).await;

            match result {
              Ok(val) => {
                let mut result_fields = IndexMap::new();
                result_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
                result_fields.insert(intern("tool"), LxVal::str(module.as_ref()));
                result_fields.insert(intern("method"), LxVal::str(method_name.as_ref()));
                result_fields.insert(intern("result"), val.clone());
                event_stream.xadd("tool/result", "main", None, result_fields);
                Ok(val)
              },
              Err(e) => {
                let err_msg = e.to_string();
                let mut error_fields = IndexMap::new();
                error_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
                error_fields.insert(intern("tool"), LxVal::str(module.as_ref()));
                error_fields.insert(intern("method"), LxVal::str(method_name.as_ref()));
                error_fields.insert(intern("error"), LxVal::str(&err_msg));
                event_stream.xadd("tool/error", "main", None, error_fields);
                Err(LxError::runtime(format!("lx tool '{module}' method '{method_name}': {err_msg}"), call_span))
              },
            }
          })
        }),
      );

      fields.insert(*name, dispatch);
    }

    Ok(LxVal::record(fields))
  }

  fn resolve_lx_tool_path(&self, command: &str, span: SourceSpan) -> Result<PathBuf, LxError> {
    let path = std::path::Path::new(command);
    if path.is_absolute() {
      return Ok(path.to_path_buf());
    }
    let source_dir = self.source_dir.as_ref().ok_or_else(|| LxError::runtime(format!("cannot resolve lx tool path '{command}': no source directory"), span))?;
    let resolved = source_dir.join(command);
    if !resolved.exists() {
      return Err(LxError::runtime(format!("lx tool file not found: {}", resolved.display()), span));
    }
    Ok(resolved)
  }
}
