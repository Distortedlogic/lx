use crate::ast::{Expr, ListElem, MapEntry, RecordField, SExpr};
use crate::error::LxError;
use crate::value::{LxVal, ValueKey};
use indexmap::IndexMap;
use std::sync::Arc;

impl super::Interpreter {
  pub(super) async fn eval_list(&mut self, elems: &[ListElem]) -> Result<LxVal, LxError> {
    let mut out = Vec::new();
    for elem in elems {
      match elem {
        ListElem::Single(e) => out.push(self.eval(e).await?),
        ListElem::Spread(e) => {
          let v = self.eval(e).await?;
          match v {
            LxVal::List(items) => out.extend(items.as_ref().iter().cloned()),
            other => {
              return Err(LxError::type_err(format!("spread requires List, got {}", other.type_name()), e.span));
            },
          }
        },
      }
    }
    Ok(LxVal::list(out))
  }

  pub(super) async fn eval_record(&mut self, fields: &[RecordField]) -> Result<LxVal, LxError> {
    let mut map = IndexMap::new();
    for f in fields {
      if f.is_spread {
        let v = self.eval(&f.value).await?;
        match v {
          LxVal::Record(r) => map.extend(r.iter().map(|(k, v)| (*k, v.clone()))),
          other => {
            return Err(LxError::type_err(format!("spread requires Record, got {}", other.type_name()), f.value.span));
          },
        }
      } else {
        let val = self.eval(&f.value).await?;
        let name = f.name.unwrap_or_else(|| if let Expr::Ident(n) = &f.value.node { *n } else { "_".into() });
        map.insert(name, val);
      }
    }
    Ok(LxVal::record(map))
  }

  pub(super) async fn eval_tuple(&mut self, elems: &[SExpr]) -> Result<LxVal, LxError> {
    let mut vals = Vec::with_capacity(elems.len());
    for e in elems {
      vals.push(self.eval(e).await?);
    }
    Ok(LxVal::tuple(vals))
  }

  pub(super) async fn eval_map(&mut self, entries: &[MapEntry]) -> Result<LxVal, LxError> {
    let mut map = IndexMap::new();
    for entry in entries {
      if entry.is_spread {
        let v = self.eval(&entry.value).await?;
        match v {
          LxVal::Map(m) => map.extend(m.iter().map(|(k, v)| (k.clone(), v.clone()))),
          other => {
            return Err(LxError::type_err(format!("spread requires Map, got {}", other.type_name()), entry.value.span));
          },
        }
      } else {
        let key_expr = entry.key.as_ref().expect("non-spread map entry must have a key");
        let key = self.eval(key_expr).await?;
        let val = self.eval(&entry.value).await?;
        map.insert(ValueKey(key), val);
      }
    }
    Ok(LxVal::Map(Arc::new(map)))
  }
}
