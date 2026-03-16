use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("get".into(), mk("env.get", 1, bi_get));
    m.insert("vars".into(), mk("env.vars", 1, bi_vars));
    m.insert("args".into(), mk("env.args", 1, bi_args));
    m.insert("cwd".into(), mk("env.cwd", 1, bi_cwd));
    m.insert("home".into(), mk("env.home", 1, bi_home));
    m
}

fn bi_get(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let key = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("env.get expects Str", span))?;
    match std::env::var(key) {
        Ok(val) => Ok(Value::Some(Box::new(Value::Str(Arc::from(val.as_str()))))),
        Err(_) => Ok(Value::None),
    }
}

fn bi_vars(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    let mut fields = IndexMap::new();
    for (k, v) in std::env::vars() {
        fields.insert(k, Value::Str(Arc::from(v.as_str())));
    }
    Ok(Value::Record(Arc::new(fields)))
}

fn bi_args(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    let items: Vec<Value> = std::env::args()
        .map(|a| Value::Str(Arc::from(a.as_str())))
        .collect();
    Ok(Value::List(Arc::new(items)))
}

fn bi_cwd(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match std::env::current_dir() {
        Ok(p) => Ok(Value::Str(Arc::from(p.to_string_lossy().as_ref()))),
        Err(e) => Err(LxError::runtime(format!("env.cwd: {e}"), span)),
    }
}

fn bi_home(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match std::env::var("HOME") {
        Ok(h) => Ok(Value::Some(Box::new(Value::Str(Arc::from(h.as_str()))))),
        Err(_) => Ok(Value::None),
    }
}
