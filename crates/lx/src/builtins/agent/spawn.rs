use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::error::LxError;
use crate::interpreter::Interpreter;
use crate::runtime::RuntimeCtx;
use crate::runtime::agent_registry::{AgentHandle, AgentMessage, register_agent};
use crate::sym::intern;
use crate::value::{LxClass, LxVal};
use miette::SourceSpan;

pub fn bi_agent_spawn(args: Vec<LxVal>, span: SourceSpan, ctx: Arc<RuntimeCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> {
  Box::pin(async move {
    let class: Box<LxClass> = match args.into_iter().next() {
      Some(LxVal::Class(c)) => c,
      _ => return Err(LxError::type_err("spawn: expected a Class value", span, None)),
    };

    let name = class.name.as_str().to_string();
    let handle_method = class.methods.get(&intern("handle")).cloned();
    let run_method = class.methods.get(&intern("run")).cloned();

    if handle_method.is_none() && run_method.is_none() {
      return Err(LxError::runtime("spawn: agent class has no handle or run method", span));
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<AgentMessage>(256);
    let pause_flag = Arc::new(AtomicBool::new(false));
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    let task_ctx = Arc::clone(&ctx);
    let task_name = name.clone();
    let task_rx = Arc::clone(&rx);

    let join_handle = tokio::task::spawn_local(async move {
      let mut interp = Interpreter::new("", None, task_ctx);
      interp.agent_name = Some(task_name.clone());

      let has_handle = handle_method.is_some();
      let has_run = run_method.is_some();

      if has_handle && !has_run {
        let Some(handle_fn) = handle_method else {
          return;
        };
        let mut rx_guard = task_rx.lock().await;
        while let Some(msg) = rx_guard.recv().await {
          let result = crate::builtins::call_value(&handle_fn, msg.payload.clone(), miette::SourceSpan::new(0.into(), 0), &interp.ctx)
            .await
            .unwrap_or_else(|e| LxVal::err_str(e.to_string()));
          if let Some(reply) = msg.reply {
            let _ = reply.send(result);
          }
        }
      } else if has_run && !has_handle {
        drop(task_rx);
        if let Some(run_fn) = run_method {
          let _ = crate::builtins::call_value(&run_fn, LxVal::Unit, miette::SourceSpan::new(0.into(), 0), &interp.ctx).await;
        }
      } else {
        interp.agent_mailbox_rx = Some(task_rx);
        interp.agent_handle_fn = handle_method;
        if let Some(run_fn) = run_method {
          let _ = crate::builtins::call_value(&run_fn, LxVal::Unit, miette::SourceSpan::new(0.into(), 0), &interp.ctx).await;
        }
      }
    });

    let handle = AgentHandle { name: name.clone(), mailbox: tx, task: join_handle, pause_flag };

    register_agent(name.clone(), handle).map_err(|msg| LxError::runtime(msg, span))?;

    let mut fields = indexmap::IndexMap::new();
    fields.insert(intern("agent"), LxVal::str(&name));
    ctx.event_stream.xadd("agent/spawn", &name, None, fields);

    Ok(LxVal::ok(LxVal::str(name)))
  })
}
