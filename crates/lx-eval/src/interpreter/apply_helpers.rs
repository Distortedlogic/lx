use std::sync::Arc;

use num_traits::ToPrimitive;

use lx_ast::ast::{ExprId, FieldKind};
use lx_value::LxVal;
use lx_value::{EvalResult, LxError};
use miette::SourceSpan;

use super::Interpreter;

impl Interpreter {
  pub(super) async fn eval_field_access(&mut self, expr: ExprId, field: &FieldKind, span: SourceSpan) -> EvalResult<LxVal> {
    let val = self.eval(expr).await?;
    match field {
      FieldKind::Named(name) => match &val {
        LxVal::Record(r) => Ok(r.get(name).cloned().unwrap_or(LxVal::None)),
        LxVal::Class(c) => {
          if let Some(method) = c.methods.get(name) {
            Ok(Self::inject_self(method, &val))
          } else {
            Ok(LxVal::None)
          }
        },
        LxVal::Object(o) => {
          if let Some(method) = o.methods.get(name) {
            Ok(Self::inject_self(method, &val))
          } else {
            Ok(crate::stdlib::object_get_field(o.id, name.as_str()).unwrap_or(LxVal::None))
          }
        },
        LxVal::Store { .. } => {
          Ok(crate::stdlib::store_method(name.as_str(), &val).ok_or_else(|| LxError::type_err(format!("Store has no method '{name}'"), span, None))?)
        },
        LxVal::Channel { name: channel_name } => Ok(crate::runtime::channel_registry::channel_dispatch(channel_name.as_str(), name.as_str(), span)?),
        other => Err(LxError::type_err(format!("field access on {}, not Record", other.type_name()), span, None).into()),
      },
      FieldKind::Index(idx) => {
        let items = match &val {
          LxVal::Tuple(t) => t.as_ref(),
          LxVal::List(l) => l.as_ref(),
          other => {
            return Err(LxError::type_err(format!("index access on {}, not Tuple/List", other.type_name()), span, None).into());
          },
        };
        let i = if *idx < 0 { items.len() as i64 + idx } else { *idx } as usize;
        Ok(items.get(i).cloned().ok_or_else(|| LxError::runtime(format!("index {idx} out of bounds"), span))?)
      },
      FieldKind::Computed(key_eid) => {
        let key = self.eval(*key_eid).await?;
        match (&val, &key) {
          (LxVal::Record(r), LxVal::Str(s)) => Ok(r.get(&lx_span::sym::intern(s)).cloned().unwrap_or(LxVal::None)),
          (LxVal::Map(m), LxVal::Str(s)) => {
            let vk = lx_value::ValueKey(LxVal::Str(s.clone()));
            Ok(m.get(&vk).cloned().unwrap_or(LxVal::None))
          },
          (LxVal::List(items), LxVal::Int(n)) => {
            let i = n.to_i64().ok_or_else(|| LxError::runtime(format!("index {n} too large for i64"), span))?;
            let i = if i < 0 { items.len() as i64 + i } else { i } as usize;
            Ok(items.get(i).cloned().ok_or_else(|| LxError::runtime(format!("index {i} out of bounds (list length {})", items.len()), span))?)
          },
          (LxVal::Tuple(items), LxVal::Int(n)) => {
            let i = n.to_i64().ok_or_else(|| LxError::runtime(format!("index {n} too large for i64"), span))?;
            let i = if i < 0 { items.len() as i64 + i } else { i } as usize;
            Ok(items.get(i).cloned().ok_or_else(|| LxError::runtime(format!("index {i} out of bounds (tuple length {})", items.len()), span))?)
          },
          _ => Err(LxError::type_err(format!("computed field access: unsupported types {} / {}", val.type_name(), key.type_name()), span, None).into()),
        }
      },
    }
  }

  fn inject_self(method: &LxVal, self_val: &LxVal) -> LxVal {
    match method {
      LxVal::Func(lf) => {
        let method_env = lf.closure.child();
        method_env.bind_str("self", self_val.clone());
        let mut lf = lf.clone();
        lf.closure = Arc::new(method_env);
        LxVal::Func(lf)
      },
      LxVal::MultiFunc(clauses) => {
        let injected = clauses
          .iter()
          .map(|lf| {
            let method_env = lf.closure.child();
            method_env.bind_str("self", self_val.clone());
            let mut lf = lf.clone();
            lf.closure = Arc::new(method_env);
            lf
          })
          .collect();
        LxVal::MultiFunc(injected)
      },
      _ => method.clone(),
    }
  }
}

