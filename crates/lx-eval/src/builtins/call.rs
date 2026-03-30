use std::sync::Arc;

use lx_ast::ast::AstArena;
use lx_value::{BuiltinCtx, BuiltinKind, Env, EvalSignal, LxError, LxVal};
use miette::SourceSpan;

use crate::interpreter::Interpreter;
use crate::runtime::RuntimeCtx;

pub(crate) fn extract_runtime_ctx(ctx: &dyn BuiltinCtx) -> &RuntimeCtx {
  ctx.as_any().downcast_ref::<RuntimeCtx>().expect("BuiltinCtx must be RuntimeCtx")
}

pub(crate) async fn call_value(f: &LxVal, arg: LxVal, span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  match f {
    LxVal::Func(lf) => {
      let mut lf = lf.clone();
      lf.applied.push(arg);
      if lf.applied.len() == 1
        && lf.arity > 1
        && let LxVal::Tuple(ref elems) = lf.applied[0]
        && elems.len() == lf.arity
      {
        let elems = elems.as_ref().clone();
        lf.applied = elems;
      }
      if lf.applied.len() < lf.arity {
        return Ok(LxVal::Func(lf));
      }
      let rtx = call_value_get_rtx(ctx);
      let mut interp = Interpreter::with_env(&lf.closure, Arc::clone(&lf.arena), rtx);
      let call_env = lf.closure.child();
      call_env.bind_params(&lf.params, &lf.applied, &lf.defaults);
      interp.set_env(call_env);
      let result = interp.eval_expr(lf.body).await;
      match result {
        Err(LxError::Propagate { value, .. }) => Ok(*value),
        other => other,
      }
    },
    LxVal::BuiltinFunc(bf) => {
      let mut bf = bf.clone();
      bf.applied.push(arg);
      if bf.applied.len() < bf.arity {
        return Ok(LxVal::BuiltinFunc(bf));
      }
      match bf.kind {
        BuiltinKind::Sync(f) => f(&bf.applied, span, ctx),
        BuiltinKind::Async(f) => f(bf.applied, span, Arc::clone(ctx)).await,
        BuiltinKind::DynAsync(ref f) => f(bf.applied.clone(), span, Arc::clone(ctx)).await,
      }
    },
    LxVal::MultiFunc(clauses) => {
      let arena = clauses.first().map(|c| Arc::clone(&c.arena)).unwrap_or_else(|| Arc::new(AstArena::new()));
      let rtx = call_value_get_rtx(ctx);
      let mut interp = Interpreter::with_env(&Env::default(), arena, rtx);
      interp.apply_func(LxVal::MultiFunc(clauses.clone()), arg, span).await.map_err(|e| match e {
        EvalSignal::Error(e) => e,
        EvalSignal::Break(_) => LxError::runtime("break outside loop", span),
        EvalSignal::AgentStop => LxError::runtime("agent stopped", span),
      })
    },
    LxVal::TaggedCtor { tag, arity, applied } => {
      let mut applied = applied.clone();
      applied.push(arg);
      if applied.len() < *arity {
        Ok(LxVal::TaggedCtor { tag: *tag, arity: *arity, applied })
      } else {
        Ok(LxVal::Tagged { tag: *tag, values: Arc::new(applied) })
      }
    },
    other => Err(LxError::type_err(format!("cannot call {}, not a function", other.type_name()), span, None)),
  }
}

fn call_value_get_rtx(ctx: &Arc<dyn BuiltinCtx>) -> Arc<RuntimeCtx> {
  if let Some(wrapper) = ctx.as_any().downcast_ref::<RuntimeCtxWrapper>() {
    Arc::clone(&wrapper.0)
  } else {
    panic!("call_value requires RuntimeCtx-backed BuiltinCtx")
  }
}

pub(crate) struct RuntimeCtxWrapper(pub Arc<RuntimeCtx>);

impl BuiltinCtx for RuntimeCtxWrapper {
  fn event_stream(&self) -> &Arc<lx_value::EventStream> {
    self.0.event_stream()
  }
  fn source_dir(&self) -> Option<std::path::PathBuf> {
    self.0.source_dir()
  }
  fn network_denied(&self) -> bool {
    self.0.network_denied()
  }
  fn test_threshold(&self) -> Option<f64> {
    self.0.test_threshold()
  }
  fn test_runs(&self) -> Option<u32> {
    self.0.test_runs()
  }
  fn as_any(&self) -> &dyn std::any::Any {
    self
  }
}

pub(crate) fn wrap_runtime_ctx(ctx: &Arc<RuntimeCtx>) -> Arc<dyn BuiltinCtx> {
  Arc::new(RuntimeCtxWrapper(Arc::clone(ctx)))
}
