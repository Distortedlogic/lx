use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("open".into(), mk("pane.open", 2, bi_open));
    m.insert("update".into(), mk("pane.update", 2, bi_update));
    m.insert("close".into(), mk("pane.close", 1, bi_close));
    m.insert("list".into(), mk("pane.list", 1, bi_list));
    m
}

fn bi_open(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let kind = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("pane.open: first arg must be Str", span))?;
    match ctx.pane.open(kind, &args[1], span) {
        Ok(handle) => Ok(Value::Ok(Box::new(handle))),
        Err(e) => Err(e),
    }
}

fn bi_update(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pane_id = args[0]
        .str_field("__pane_id")
        .ok_or_else(|| LxError::type_err("pane.update: first arg must be a pane handle", span))?;
    ctx.pane.update(pane_id, &args[1], span)?;
    Ok(Value::Unit)
}

fn bi_close(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pane_id = args[0]
        .str_field("__pane_id")
        .ok_or_else(|| LxError::type_err("pane.close: first arg must be a pane handle", span))?;
    ctx.pane.close(pane_id, span)?;
    Ok(Value::Unit)
}

fn bi_list(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    ctx.pane.list(span)
}
