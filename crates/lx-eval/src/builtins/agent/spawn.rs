use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::interpreter::Interpreter;
use crate::runtime::RuntimeCtx;
use crate::runtime::agent_registry::{AgentHandle, AgentMessage, register_agent};
use lx_span::sym::intern;
use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::{LxClass, LxVal};
use miette::SourceSpan;

pub fn bi_agent_spawn(args: Vec<LxVal>, span: SourceSpan, ctx: Arc<dyn BuiltinCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> {
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

    let parent_rtx = crate::builtins::call::call_value_get_rtx(&ctx);
    let rtx: Arc<RuntimeCtx> = Arc::new(RuntimeCtx {
      yield_: Arc::new(crate::runtime::StdinStdoutYieldBackend),
      source_dir: parking_lot::Mutex::new(None),
      workspace_members: HashMap::new(),
      dep_dirs: HashMap::new(),
      tools: HashMap::new(),
      tokio_runtime: Arc::clone(&parent_rtx.tokio_runtime),
      test_threshold: ctx.test_threshold(),
      test_runs: ctx.test_runs(),
      event_stream: Arc::clone(ctx.event_stream()),
      network_denied: ctx.network_denied(),
      global_pause: Arc::new(std::sync::atomic::AtomicBool::new(false)),
      cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
      inject_tx: None,
    });
    let task_name = name.clone();
    let task_rx = Arc::clone(&rx);

    let join_handle = std::thread::spawn(move || {
      let rt = Arc::clone(&rtx.tokio_runtime);
      let _guard = rt.enter();
      rt.handle().block_on(async move {
        let mut interp = Interpreter::new("", None, Arc::clone(&rtx));
        interp.agent_name = Some(task_name.clone());

        let has_handle = handle_method.is_some();
        let has_run = run_method.is_some();

        if has_handle && !has_run {
          let Some(handle_fn) = handle_method else {
            return;
          };
          let mut rx_guard = task_rx.lock().await;
          let bctx = interp.builtin_ctx();
          while let Some(msg) = rx_guard.recv().await {
            let result = crate::builtins::call_value(&handle_fn, msg.payload.clone(), miette::SourceSpan::new(0.into(), 0), &bctx)
              .await
              .unwrap_or_else(|e| LxVal::err_str(e.to_string()));
            if let Some(reply) = msg.reply {
              let _ = reply.send(result);
            }
          }
        } else if has_run && !has_handle {
          drop(task_rx);
          if let Some(run_fn) = run_method {
            let bctx = interp.builtin_ctx();
            let _ = crate::builtins::call_value(&run_fn, LxVal::Unit, miette::SourceSpan::new(0.into(), 0), &bctx).await;
          }
        } else {
          interp.agent_mailbox_rx = Some(task_rx.clone());
          interp.agent_handle_fn = handle_method.clone();
          if let Some(run_fn) = run_method {
            let bctx = interp.builtin_ctx();
            let _ = crate::builtins::call_value(&run_fn, LxVal::Unit, miette::SourceSpan::new(0.into(), 0), &bctx).await;
          }
          if let Some(handle_fn) = handle_method {
            let mut rx_guard = task_rx.lock().await;
            let bctx = interp.builtin_ctx();
            while let Some(msg) = rx_guard.recv().await {
              let result = crate::builtins::call_value(&handle_fn, msg.payload.clone(), miette::SourceSpan::new(0.into(), 0), &bctx)
                .await
                .unwrap_or_else(|e| LxVal::err_str(e.to_string()));
              if let Some(reply) = msg.reply {
                let _ = reply.send(result);
              }
            }
          }
        }
    })});

    let handle = AgentHandle { name: name.clone(), mailbox: tx, task: join_handle, pause_flag };

    register_agent(name.clone(), handle).map_err(|msg| LxError::runtime(msg, span))?;

    let subscribes_val = class.defaults.get(&intern("subscribes")).or_else(|| class.methods.get(&intern("subscribes")));
    if let Some(LxVal::List(channels)) = subscribes_val {
      for ch in channels.iter() {
        if let LxVal::Channel { name: ch_name } = ch {
          crate::runtime::channel_registry::channel_subscribe(ch_name.as_str(), &name).unwrap_or_else(|e| eprintln!("auto-subscribe failed: {e}"));
        } else if let Some(ch_name) = ch.as_str() {
          crate::runtime::channel_registry::channel_subscribe(ch_name, &name).unwrap_or_else(|e| eprintln!("auto-subscribe failed: {e}"));
        }
      }
    }
    let mut fields = indexmap::IndexMap::new();
    fields.insert(intern("agent"), LxVal::str(&name));
    if let Some(adapter_val) = class.defaults.get(&intern("adapter"))
      && let Some(adapter_str) = adapter_val.as_str()
    {
      fields.insert(intern("adapter"), LxVal::str(adapter_str));
    }
    ctx.event_stream().xadd("agent/spawn", &name, None, fields);

    Ok(LxVal::ok(LxVal::str(name)))
  })
}
