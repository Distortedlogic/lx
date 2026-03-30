use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::LxVal;

macro_rules! marker_map {
  ($serializer:expr, $(($key:expr, $val:expr)),+ $(,)?) => {{
    let count = [$($key),+].len();
    let mut map = $serializer.serialize_map(Some(count))?;
    $(map.serialize_entry($key, $val)?;)+
    map.end()
  }};
}

impl Serialize for LxVal {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      LxVal::Int(n) => {
        if let Ok(i) = i64::try_from(n) {
          serializer.serialize_i64(i)
        } else {
          serializer.serialize_str(&n.to_string())
        }
      },
      LxVal::Float(f) => serializer.serialize_f64(*f),
      LxVal::Bool(b) => serializer.serialize_bool(*b),
      LxVal::Str(s) => serializer.serialize_str(s),
      LxVal::Unit | LxVal::None => serializer.serialize_none(),
      LxVal::List(items) => items.serialize(serializer),
      LxVal::Tuple(items) => items.serialize(serializer),
      LxVal::Record(fields) => {
        let mut map = serializer.serialize_map(Some(fields.len()))?;
        for (k, v) in fields.as_ref() {
          map.serialize_entry(k.as_str(), v)?;
        }
        map.end()
      },
      LxVal::Map(entries) => {
        let mut map = serializer.serialize_map(Some(entries.len()))?;
        for (k, v) in entries.as_ref() {
          map.serialize_entry(&k.0.to_string(), v)?;
        }
        map.end()
      },
      LxVal::Ok(v) => v.serialize(serializer),
      LxVal::Some(v) => v.serialize(serializer),
      LxVal::Err(v) => marker_map!(serializer, ("__err", v.as_ref())),
      LxVal::Tagged { tag, values } => {
        marker_map!(serializer, ("__tag", &tag.as_str()), ("__values", values.as_ref()))
      },
      LxVal::Range { start, end, inclusive } => {
        marker_map!(serializer, ("start", start), ("end", end), ("inclusive", inclusive))
      },
      LxVal::Store { id } => marker_map!(serializer, ("__store", id)),
      LxVal::Stream { id } => marker_map!(serializer, ("__stream", id)),
      LxVal::Type(s) => serializer.serialize_str(s.as_str()),
      LxVal::Object(o) => marker_map!(serializer, ("__object", &o.id), ("__class", &o.class_name.as_str())),
      _ => serializer.serialize_str(&format!("<{}>", self.type_name())),
    }
  }
}

impl<'de> Deserialize<'de> for LxVal {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let json = serde_json::Value::deserialize(deserializer)?;
    Ok(LxVal::from(json))
  }
}

impl From<serde_json::Value> for LxVal {
  fn from(v: serde_json::Value) -> Self {
    match v {
      serde_json::Value::Null => LxVal::None,
      serde_json::Value::Bool(b) => LxVal::Bool(b),
      serde_json::Value::Number(n) => {
        if let Some(i) = n.as_i64() {
          LxVal::Int(BigInt::from(i))
        } else {
          LxVal::Float(n.as_f64().or_else(|| n.as_u64().map(|u| u as f64)).unwrap_or(0.0))
        }
      },
      serde_json::Value::String(s) => LxVal::Str(Arc::from(s.as_str())),
      serde_json::Value::Array(arr) => LxVal::List(Arc::new(arr.into_iter().map(LxVal::from).collect())),
      serde_json::Value::Object(obj) => {
        let mut rec = IndexMap::new();
        for (k, v) in obj {
          rec.insert(lx_span::sym::intern(&k), LxVal::from(v));
        }
        LxVal::Record(Arc::new(rec))
      },
    }
  }
}

impl From<&LxVal> for serde_json::Value {
  fn from(v: &LxVal) -> Self {
    serde_json::to_value(v).expect("LxVal is always serializable")
  }
}
