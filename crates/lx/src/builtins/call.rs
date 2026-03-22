use std::sync::Arc;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::{BuiltinKind, LxVal};
use miette::SourceSpan;

pub(crate) async fn call_value(f: &LxVal, arg: LxVal, span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
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
      let mut interp = crate::interpreter::Interpreter::with_env(&lf.closure, Arc::clone(ctx));
      let mut call_env = lf.closure.child();
      for (i, &sym) in lf.params.iter().enumerate() {
        if i < lf.applied.len() {
          call_env.bind(sym, lf.applied[i].clone());
        } else if let Some(Some(def)) = lf.defaults.get(i) {
          call_env.bind(sym, def.clone());
        }
      }
      interp.set_env(call_env);
      let result = interp.eval_expr(&lf.body).await;
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
      }
    },
    LxVal::TaggedCtor { tag, arity, applied } => {
      let mut applied = applied.clone();
      applied.push(arg);
      if applied.len() < *arity {
        Ok(LxVal::TaggedCtor { tag: tag.clone(), arity: *arity, applied })
      } else {
        Ok(LxVal::Tagged { tag: tag.clone(), values: Arc::new(applied) })
      }
    },
    other => Err(LxError::type_err(format!("cannot call {}, not a function", other.type_name()), span)),
  }
}
