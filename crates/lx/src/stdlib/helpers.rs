use std::fmt::Display;
use std::sync::Arc;

use chrono::{DateTime, Local, Utc};
use indexmap::IndexMap;

use crate::error::LxError;
use crate::record;
use crate::sym::{Sym, intern};
use crate::value::LxVal;
use miette::SourceSpan;

pub fn extract_handle_id(val: &LxVal, field: &str, fn_name: &str, span: SourceSpan) -> Result<u64, LxError> {
  match val {
    LxVal::Record(r) => r
      .get(&intern(field))
      .and_then(|v| v.as_int())
      .and_then(|n| n.try_into().ok())
      .ok_or_else(|| LxError::type_err(format!("{fn_name}: expected handle with '{field}'"), span, None)),
    _ => Err(LxError::type_err(format!("{fn_name}: expected Record handle"), span, None)),
  }
}

pub fn wrap_io<T: Into<LxVal>>(r: Result<T, impl Display>) -> LxVal {
  match r {
    Ok(v) => LxVal::ok(v.into()),
    Err(e) => LxVal::err_str(e.to_string()),
  }
}

pub fn datetime_to_record(dt: DateTime<Utc>) -> LxVal {
  let local: DateTime<Local> = dt.with_timezone(&Local);
  record! {
      "epoch" => LxVal::int(dt.timestamp()),
      "ms" => LxVal::int(dt.timestamp_millis()),
      "iso" => LxVal::str(dt.to_rfc3339()),
      "local" => LxVal::str(local.to_rfc3339()),
  }
}

pub fn require_str_field(rec: &IndexMap<Sym, LxVal>, key: &str, fn_name: &str, span: SourceSpan) -> Result<Arc<str>, LxError> {
  rec.get(&intern(key)).and_then(|v| v.as_str()).map(Arc::from).ok_or_else(|| LxError::type_err(format!("{fn_name}: '{key}' must be Str"), span, None))
}

#[macro_export]
macro_rules! std_module {
  ($($name:expr => $builtin_name:expr, $arity:expr, $func:expr);+ $(;)?) => {{
    let mut m = indexmap::IndexMap::new();
    $(m.insert($crate::sym::intern($name), $crate::builtins::mk($builtin_name, $arity, $func));)+
    m
  }};
}
