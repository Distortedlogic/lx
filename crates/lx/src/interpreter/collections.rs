use crate::ast::{ExprId, ListElem, MapEntry, RecordField};
use crate::error::LxError;
use crate::value::{LxVal, ValueKey};
use indexmap::IndexMap;
use std::sync::Arc;

impl super::Interpreter {
  pub(super) async fn eval_list(&mut self, elems: &[ListElem]) -> Result<LxVal, LxError> {
    let mut out = Vec::new();
    for elem in elems {
      let eid = match elem {
        ListElem::Single(eid) => *eid,
        ListElem::Spread(eid) => *eid,
      };
      let v = self.eval(eid).await?;
      match elem {
        ListElem::Single(_) => out.push(v),
        ListElem::Spread(_) => match v {
          LxVal::List(items) => out.extend(items.as_ref().iter().cloned()),
          other => {
            let espan = self.arena.expr_span(eid);
            return Err(LxError::type_err(format!("spread requires List, got {}", other.type_name()), espan, None));
          },
        },
      }
    }
    Ok(LxVal::list(out))
  }

  pub(super) async fn eval_record(&mut self, fields: &[RecordField]) -> Result<LxVal, LxError> {
    let mut map = IndexMap::new();
    for f in fields {
      match f {
        RecordField::Spread(value) => {
          let v = self.eval(*value).await?;
          let fspan = self.arena.expr_span(*value);
          match v {
            LxVal::Record(r) => map.extend(r.iter().map(|(k, v)| (*k, v.clone()))),
            other => {
              return Err(LxError::type_err(format!("spread requires Record, got {}", other.type_name()), fspan, None));
            },
          }
        },
        RecordField::Named { name, value } => {
          let val = self.eval(*value).await?;
          map.insert(*name, val);
        },
      }
    }
    Ok(LxVal::record(map))
  }

  pub(super) async fn eval_tuple(&mut self, elems: &[ExprId]) -> Result<LxVal, LxError> {
    let mut vals = Vec::with_capacity(elems.len());
    for &e in elems {
      vals.push(self.eval(e).await?);
    }
    Ok(LxVal::tuple(vals))
  }

  pub(super) async fn eval_map(&mut self, entries: &[MapEntry]) -> Result<LxVal, LxError> {
    let mut map = IndexMap::new();
    for entry in entries {
      match entry {
        MapEntry::Spread(value) => {
          let v = self.eval(*value).await?;
          let vspan = self.arena.expr_span(*value);
          match v {
            LxVal::Map(m) => map.extend(m.iter().map(|(k, v)| (k.clone(), v.clone()))),
            other => {
              return Err(LxError::type_err(format!("spread requires Map, got {}", other.type_name()), vspan, None));
            },
          }
        },
        MapEntry::Keyed { key, value } => {
          let k = self.eval(*key).await?;
          let val = self.eval(*value).await?;
          map.insert(ValueKey(k), val);
        },
      }
    }
    Ok(LxVal::Map(Arc::new(map)))
  }
}
