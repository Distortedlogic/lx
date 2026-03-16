use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub(crate) fn call_value(
    f: &Value,
    arg: Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    match f {
        Value::Func(lf) => {
            let mut lf = lf.clone();
            lf.applied.push(arg);
            if lf.applied.len() == 1
                && lf.arity > 1
                && let Value::Tuple(ref elems) = lf.applied[0]
                && elems.len() == lf.arity
            {
                let elems = elems.as_ref().clone();
                lf.applied = elems;
            }
            if lf.applied.len() < lf.arity {
                return Ok(Value::Func(lf));
            }
            let mut interp =
                crate::interpreter::Interpreter::with_env(&lf.closure, Arc::clone(ctx));
            let mut call_env = lf.closure.child();
            for (i, name) in lf.params.iter().enumerate() {
                if i < lf.applied.len() {
                    call_env.bind(name.clone(), lf.applied[i].clone());
                } else if let Some(Some(def)) = lf.defaults.get(i) {
                    call_env.bind(name.clone(), def.clone());
                }
            }
            interp.set_env(call_env);
            let result = interp.eval_expr(&lf.body);
            match result {
                Err(LxError::Propagate { value, .. }) => Ok(*value),
                other => other,
            }
        }
        Value::BuiltinFunc(bf) => {
            let mut bf = bf.clone();
            bf.applied.push(arg);
            if bf.applied.len() < bf.arity {
                return Ok(Value::BuiltinFunc(bf));
            }
            (bf.func)(&bf.applied, span, ctx)
        }
        Value::TaggedCtor {
            tag,
            arity,
            applied,
        } => {
            let mut applied = applied.clone();
            applied.push(arg);
            if applied.len() < *arity {
                Ok(Value::TaggedCtor {
                    tag: tag.clone(),
                    arity: *arity,
                    applied,
                })
            } else {
                Ok(Value::Tagged {
                    tag: tag.clone(),
                    values: Arc::new(applied),
                })
            }
        }
        other => Err(LxError::type_err(
            format!("cannot call {}, not a function", other.type_name()),
            span,
        )),
    }
}
