use std::sync::Arc;

use indexmap::IndexMap;

use crate::BuiltinCtx;
use crate::error::LxError;
use crate::record;
use crate::std_module;
use crate::sym::{Sym, intern};
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<Sym, LxVal> {
  std_module! {
      "define"       => "schema.define",       2, bi_define;
      "validate"     => "schema.validate",     2, bi_validate;
      "validate_all" => "schema.validate_all", 2, bi_validate_all;
      "check"        => "schema.check",        2, bi_check
  }
}

const SCHEMA_TAG: &str = "__schema";

fn parse_constraint(_field_name: Sym, val: &LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
  match val {
    LxVal::Bool(true) => Ok(record! {
        "required" => LxVal::Bool(true)
    }),
    LxVal::Record(r) => {
      let required = r.get(&intern("required")).and_then(|v| v.as_bool()).unwrap_or(false);
      let mut out = IndexMap::new();
      out.insert(intern("required"), LxVal::Bool(required));
      if let Some(default_val) = r.get(&intern("default")) {
        out.insert(intern("default"), default_val.clone());
      }
      if let Some(check_fn) = r.get(&intern("check")) {
        out.insert(intern("check"), check_fn.clone());
      }
      if let Some(one_of) = r.get(&intern("one_of")) {
        one_of.require_list("schema.define one_of", span)?;
        out.insert(intern("one_of"), one_of.clone());
      }
      Ok(LxVal::record(out))
    },
    other => Ok(record! {
        "required" => LxVal::Bool(false),
        "default" => other.clone()
    }),
  }
}

fn bi_define(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let name = args[0].require_str("schema.define", span)?;
  let spec = args[1].require_record("schema.define", span)?;
  let mut constraints = IndexMap::new();
  for (field_name, val) in spec.iter() {
    let constraint = parse_constraint(*field_name, val, span)?;
    constraints.insert(*field_name, constraint);
  }
  Ok(LxVal::Tagged { tag: intern(SCHEMA_TAG), values: Arc::new(vec![LxVal::str(name), LxVal::record(constraints)]) })
}

fn extract_schema<'a>(args: &'a [LxVal], fn_name: &str, span: SourceSpan) -> Result<&'a IndexMap<Sym, LxVal>, LxError> {
  match &args[0] {
    LxVal::Tagged { tag, values } if *tag == intern(SCHEMA_TAG) => values[1].require_record(fn_name, span),
    other => Err(LxError::type_err(format!("{fn_name}: expected Schema, got {}", other.type_name()), span, None)),
  }
}

struct ValidationError {
  field: Sym,
  reason: String,
}

fn validate_field(
  field_name: Sym,
  constraint: &IndexMap<Sym, LxVal>,
  data: &IndexMap<Sym, LxVal>,
  span: SourceSpan,
  ctx: &Arc<dyn BuiltinCtx>,
) -> Result<Option<LxVal>, ValidationError> {
  let required = constraint.get(&intern("required")).and_then(|v| v.as_bool()).unwrap_or(false);
  let val = match data.get(&field_name) {
    Some(v) if !matches!(v, LxVal::None) => v,
    _ => {
      if required {
        return Err(ValidationError { field: field_name, reason: "required field missing".into() });
      }
      if let Some(default_val) = constraint.get(&intern("default")) {
        return Ok(Some(default_val.clone()));
      }
      return Ok(None);
    },
  };

  if let Some(one_of_val) = constraint.get(&intern("one_of"))
    && let Some(list) = one_of_val.as_list()
    && !list.iter().any(|item| item == val)
  {
    let items: Vec<String> = list.iter().map(|v| v.to_string()).collect();
    return Err(ValidationError { field: field_name, reason: format!("must be one of: {}", items.join(", ")) });
  }

  if let Some(check_fn) = constraint.get(&intern("check")) {
    match crate::builtins::call_value_sync(check_fn, val.clone(), span, ctx) {
      Ok(result) => {
        if result.as_bool() == Some(false) {
          return Err(ValidationError { field: field_name, reason: "validation check failed".into() });
        }
      },
      Err(e) => {
        return Err(ValidationError { field: field_name, reason: format!("check function error: {e}") });
      },
    }
  }

  Ok(None)
}

fn error_to_record(err: &ValidationError) -> LxVal {
  record! {
      "field" => LxVal::str(err.field.as_str()),
      "reason" => LxVal::str(&err.reason)
  }
}

fn do_validate(
  constraints: &IndexMap<Sym, LxVal>,
  data: &IndexMap<Sym, LxVal>,
  collect_all: bool,
  span: SourceSpan,
  ctx: &Arc<dyn BuiltinCtx>,
) -> Result<LxVal, LxError> {
  let mut result = IndexMap::new();
  let mut errors: Vec<ValidationError> = Vec::new();

  for (field_name, constraint_val) in constraints.iter() {
    let constraint_rec = constraint_val.require_record("schema.validate", span)?;
    match validate_field(*field_name, constraint_rec, data, span, ctx) {
      Ok(Some(default_val)) => {
        result.insert(*field_name, default_val);
      },
      Ok(None) => {},
      Err(e) => {
        if collect_all {
          errors.push(e);
        } else {
          return Ok(LxVal::err(error_to_record(&e)));
        }
      },
    }
  }

  if !errors.is_empty() {
    let err_list: Vec<LxVal> = errors.iter().map(error_to_record).collect();
    return Ok(LxVal::err(LxVal::list(err_list)));
  }

  for (key, val) in data.iter() {
    if !result.contains_key(key) {
      result.insert(*key, val.clone());
    }
  }

  Ok(LxVal::ok(LxVal::record(result)))
}

fn bi_validate(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let constraints = extract_schema(args, "schema.validate", span)?;
  let data = args[1].require_record("schema.validate", span)?;
  do_validate(constraints, data, false, span, ctx)
}

fn bi_validate_all(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let constraints = extract_schema(args, "schema.validate_all", span)?;
  let data = args[1].require_record("schema.validate_all", span)?;
  do_validate(constraints, data, true, span, ctx)
}

fn bi_check(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let constraints = extract_schema(args, "schema.check", span)?;
  let data = args[1].require_record("schema.check", span)?;
  Ok(LxVal::Bool(matches!(do_validate(constraints, data, false, span, ctx)?, LxVal::Ok(_))))
}
