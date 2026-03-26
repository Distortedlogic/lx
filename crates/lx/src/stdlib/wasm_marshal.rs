use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::error::LxError;
use crate::sym::intern;
use crate::value::LxVal;

pub fn lxval_to_json(val: &LxVal) -> Result<String, LxError> {
  let json = lxval_to_json_value(val)?;
  serde_json::to_string(&json).map_err(|e| LxError::runtime(format!("json serialization failed: {e}"), (0, 0).into()))
}

fn lxval_to_json_value(val: &LxVal) -> Result<serde_json::Value, LxError> {
  match val {
    LxVal::Int(n) => {
      if let Some(i) = n.to_i64() {
        Ok(serde_json::Value::Number(serde_json::Number::from(i)))
      } else {
        Ok(serde_json::Value::String(n.to_string()))
      }
    },
    LxVal::Float(f) => serde_json::Number::from_f64(*f)
      .map(serde_json::Value::Number)
      .ok_or_else(|| LxError::runtime(format!("cannot serialize float {f} to JSON"), (0, 0).into())),
    LxVal::Bool(b) => Ok(serde_json::Value::Bool(*b)),
    LxVal::Str(s) => Ok(serde_json::Value::String(s.to_string())),
    LxVal::Unit => Ok(serde_json::Value::Null),
    LxVal::None => Ok(serde_json::Value::Null),
    LxVal::List(items) => {
      let arr: Result<Vec<_>, _> = items.iter().map(lxval_to_json_value).collect();
      Ok(serde_json::Value::Array(arr?))
    },
    LxVal::Tuple(items) => {
      let arr: Result<Vec<_>, _> = items.iter().map(lxval_to_json_value).collect();
      Ok(serde_json::Value::Array(arr?))
    },
    LxVal::Record(fields) => {
      let mut map = serde_json::Map::new();
      for (k, v) in fields.as_ref() {
        map.insert(k.as_str().to_string(), lxval_to_json_value(v)?);
      }
      Ok(serde_json::Value::Object(map))
    },
    LxVal::Ok(v) => {
      let mut map = serde_json::Map::new();
      map.insert("Ok".to_string(), lxval_to_json_value(v)?);
      Ok(serde_json::Value::Object(map))
    },
    LxVal::Err(v) => {
      let mut map = serde_json::Map::new();
      map.insert("Err".to_string(), lxval_to_json_value(v)?);
      Ok(serde_json::Value::Object(map))
    },
    LxVal::Some(v) => lxval_to_json_value(v),
    LxVal::Tagged { tag, values } => {
      let mut map = serde_json::Map::new();
      map.insert("_tag".to_string(), serde_json::Value::String(tag.as_str().to_string()));
      let arr: Result<Vec<_>, _> = values.iter().map(lxval_to_json_value).collect();
      map.insert("values".to_string(), serde_json::Value::Array(arr?));
      Ok(serde_json::Value::Object(map))
    },
    LxVal::Type(s) => Ok(serde_json::Value::String(s.as_str().to_string())),
    other => Err(LxError::runtime(format!("cannot serialize {} to JSON for wasm plugin call", other.type_name()), (0, 0).into())),
  }
}

pub fn json_to_lxval(json: &str) -> Result<LxVal, LxError> {
  let value: serde_json::Value = serde_json::from_str(json).map_err(|e| LxError::runtime(format!("json parse failed: {e}"), (0, 0).into()))?;
  Ok(json_value_to_lxval(value))
}

fn json_value_to_lxval(value: serde_json::Value) -> LxVal {
  match value {
    serde_json::Value::Null => LxVal::None,
    serde_json::Value::Bool(b) => LxVal::Bool(b),
    serde_json::Value::Number(n) => {
      if let Some(i) = n.as_i64() {
        LxVal::Int(BigInt::from(i))
      } else {
        LxVal::Float(n.as_f64().unwrap_or(0.0))
      }
    },
    serde_json::Value::String(s) => LxVal::Str(Arc::from(s.as_str())),
    serde_json::Value::Array(arr) => LxVal::List(Arc::new(arr.into_iter().map(json_value_to_lxval).collect())),
    serde_json::Value::Object(obj) => {
      if let Some(ok_val) = obj.get("Ok")
        && obj.len() == 1
      {
        return LxVal::Ok(Box::new(json_value_to_lxval(ok_val.clone())));
      }
      if let Some(err_val) = obj.get("Err")
        && obj.len() == 1
      {
        return LxVal::Err(Box::new(json_value_to_lxval(err_val.clone())));
      }
      if let Some(serde_json::Value::String(tag)) = obj.get("_tag")
        && let Some(serde_json::Value::Array(vals)) = obj.get("values")
        && obj.len() == 2
      {
        let tag = intern(tag);
        let values: Vec<LxVal> = vals.iter().cloned().map(json_value_to_lxval).collect();
        return LxVal::Tagged { tag, values: Arc::new(values) };
      }
      let mut rec = IndexMap::new();
      for (k, v) in obj {
        rec.insert(intern(&k), json_value_to_lxval(v));
      }
      LxVal::Record(Arc::new(rec))
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn roundtrip(val: &LxVal) -> LxVal {
    let json = lxval_to_json(val).expect("serialize");
    json_to_lxval(&json).expect("deserialize")
  }

  #[test]
  fn test_int() {
    let val = LxVal::int(42);
    let rt = roundtrip(&val);
    assert_eq!(rt.as_int().and_then(|n| n.to_i64()), Some(42));
  }

  #[test]
  fn test_bigint() {
    let big = BigInt::parse_bytes(b"99999999999999999999999999", 10).expect("parse");
    let val = LxVal::Int(big.clone());
    let json = lxval_to_json(&val).expect("serialize");
    let rt = json_to_lxval(&json).expect("deserialize");
    assert_eq!(rt.as_str(), Some("99999999999999999999999999"));
  }

  #[test]
  fn test_float() {
    let val = LxVal::Float(2.72);
    let rt = roundtrip(&val);
    assert!((rt.as_float().expect("float") - 2.72).abs() < 1e-10);
  }

  #[test]
  fn test_bool() {
    assert_eq!(roundtrip(&LxVal::Bool(true)).as_bool(), Some(true));
    assert_eq!(roundtrip(&LxVal::Bool(false)).as_bool(), Some(false));
  }

  #[test]
  fn test_str() {
    let val = LxVal::str("hello world");
    let rt = roundtrip(&val);
    assert_eq!(rt.as_str(), Some("hello world"));
  }

  #[test]
  fn test_unit() {
    let rt = roundtrip(&LxVal::Unit);
    assert!(matches!(rt, LxVal::None));
  }

  #[test]
  fn test_none() {
    let rt = roundtrip(&LxVal::None);
    assert!(matches!(rt, LxVal::None));
  }

  #[test]
  fn test_list() {
    let val = LxVal::list(vec![LxVal::int(1), LxVal::str("two"), LxVal::Bool(true)]);
    let json = lxval_to_json(&val).expect("serialize");
    let rt = json_to_lxval(&json).expect("deserialize");
    let items = rt.as_list().expect("list");
    assert_eq!(items.len(), 3);
  }

  #[test]
  fn test_record() {
    let mut fields = IndexMap::new();
    fields.insert(intern("name"), LxVal::str("alice"));
    fields.insert(intern("age"), LxVal::int(30));
    let val = LxVal::Record(Arc::new(fields));
    let rt = roundtrip(&val);
    assert_eq!(rt.str_field("name"), Some("alice"));
  }

  #[test]
  fn test_ok() {
    let val = LxVal::ok(LxVal::int(42));
    let rt = roundtrip(&val);
    assert!(matches!(rt, LxVal::Ok(_)));
  }

  #[test]
  fn test_err() {
    let val = LxVal::err(LxVal::str("bad"));
    let rt = roundtrip(&val);
    assert!(matches!(rt, LxVal::Err(_)));
  }

  #[test]
  fn test_some() {
    let val = LxVal::some(LxVal::int(7));
    let json = lxval_to_json(&val).expect("serialize");
    let rt = json_to_lxval(&json).expect("deserialize");
    assert_eq!(rt.as_int().and_then(|n| n.to_i64()), Some(7));
  }

  #[test]
  fn test_tagged() {
    let val = LxVal::Tagged { tag: intern("Color"), values: Arc::new(vec![LxVal::int(255), LxVal::int(0), LxVal::int(0)]) };
    let rt = roundtrip(&val);
    if let LxVal::Tagged { tag, values } = rt {
      assert_eq!(tag.as_str(), "Color");
      assert_eq!(values.len(), 3);
    } else {
      panic!("expected Tagged");
    }
  }
}
