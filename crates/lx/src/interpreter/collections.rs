use crate::ast::{ListElem, RecordField, Expr, SExpr, MapEntry};
use crate::error::LxError;
use crate::value::{Value, ValueKey};
use indexmap::IndexMap;
use std::sync::Arc;

impl super::Interpreter {
  pub(super) fn eval_list(&mut self, elems: &[ListElem]) -> Result<Value, LxError> {
    let mut out = Vec::new();
    for elem in elems {
      match elem {
        ListElem::Single(e) => out.push(self.eval(e)?),
        ListElem::Spread(e) => {
          let v = self.eval(e)?;
          match v {
            Value::List(items) => out.extend(items.as_ref().iter().cloned()),
            other => return Err(LxError::type_err(format!("spread requires List, got {}", other.type_name()), e.span)),
          }
        },
      }
    }
    Ok(Value::List(Arc::new(out)))
  }

  pub(super) fn eval_record(&mut self, fields: &[RecordField]) -> Result<Value, LxError> {
    let mut map = IndexMap::new();
    for f in fields {
      if f.is_spread {
        let v = self.eval(&f.value)?;
        match v {
          Value::Record(r) => {
            for (k, v) in r.as_ref() {
              map.insert(k.clone(), v.clone());
            }
          },
          other => return Err(LxError::type_err(format!("spread requires Record, got {}", other.type_name()), f.value.span)),
        }
      } else {
        let val = self.eval(&f.value)?;
        let name = f.name.clone().unwrap_or_else(|| if let Expr::Ident(n) = &f.value.node { n.clone() } else { "_".into() });
        map.insert(name, val);
      }
    }
    Ok(Value::Record(Arc::new(map)))
  }

  pub(super) fn eval_tuple(&mut self, elems: &[SExpr]) -> Result<Value, LxError> {
    let vals: Result<Vec<_>, _> = elems.iter().map(|e| self.eval(e)).collect();
    Ok(Value::Tuple(Arc::new(vals?)))
  }

  pub(super) fn eval_map(&mut self, entries: &[MapEntry]) -> Result<Value, LxError> {
    let mut map = IndexMap::new();
    for entry in entries {
      if entry.is_spread {
        let v = self.eval(&entry.value)?;
        match v {
          Value::Map(m) => {
            for (k, v) in m.as_ref() {
              map.insert(k.clone(), v.clone());
            }
          },
          other => return Err(LxError::type_err(format!("spread requires Map, got {}", other.type_name()), entry.value.span)),
        }
      } else {
        let key_expr = entry.key.as_ref().expect("non-spread map entry must have a key");
        let key = self.eval(key_expr)?;
        let val = self.eval(&entry.value)?;
        map.insert(ValueKey(key), val);
      }
    }
    Ok(Value::Map(Arc::new(map)))
  }

}
