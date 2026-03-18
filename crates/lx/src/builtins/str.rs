use std::sync::Arc;

use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::mk;

#[path = "str_extra.rs"]
mod str_extra;

fn str_transform(
    args: &[Value],
    span: Span,
    name: &str,
    f: fn(&str) -> String,
) -> Result<Value, LxError> {
    match &args[0] {
        Value::Str(s) => Ok(Value::Str(Arc::from(f(s).as_str()))),
        other => Err(LxError::type_err(
            format!("{name} expects Str, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_trim(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    str_transform(args, span, "trim", |s| s.trim().to_string())
}

fn bi_trim_start(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    str_transform(args, span, "trim_start", |s| s.trim_start().to_string())
}

fn bi_trim_end(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    str_transform(args, span, "trim_end", |s| s.trim_end().to_string())
}

fn bi_upper(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    str_transform(args, span, "upper", |s| s.to_uppercase())
}

fn bi_lower(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    str_transform(args, span, "lower", |s| s.to_lowercase())
}

fn bi_lines(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Str(s) => {
            let items: Vec<Value> = s.lines().map(|l| Value::Str(Arc::from(l))).collect();
            Ok(Value::List(Arc::new(items)))
        }
        other => Err(LxError::type_err(
            format!("lines expects Str, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_chars(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Str(s) => {
            let items: Vec<Value> = s
                .chars()
                .map(|c| Value::Str(Arc::from(c.to_string().as_str())))
                .collect();
            Ok(Value::List(Arc::new(items)))
        }
        other => Err(LxError::type_err(
            format!("chars expects Str, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_byte_len(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    match &args[0] {
        Value::Str(s) => Ok(Value::Int(BigInt::from(s.len()))),
        other => Err(LxError::type_err(
            format!("byte_len expects Str, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_split(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sep = args[0].as_str().ok_or_else(|| {
        LxError::type_err(
            format!("split: first arg must be Str, got {}", args[0].type_name()),
            span,
        )
    })?;
    let s = args[1].as_str().ok_or_else(|| {
        LxError::type_err(
            format!("split: second arg must be Str, got {}", args[1].type_name()),
            span,
        )
    })?;
    let items: Vec<Value> = s.split(sep).map(|p| Value::Str(Arc::from(p))).collect();
    Ok(Value::List(Arc::new(items)))
}

fn bi_join(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sep = args[0].as_str().ok_or_else(|| {
        LxError::type_err(
            format!("join: first arg must be Str, got {}", args[0].type_name()),
            span,
        )
    })?;
    let list = args[1].as_list().ok_or_else(|| {
        LxError::type_err(
            format!("join: second arg must be List, got {}", args[1].type_name()),
            span,
        )
    })?;
    let parts: Result<Vec<&str>, LxError> = list
        .iter()
        .map(|v| {
            v.as_str().ok_or_else(|| {
                LxError::type_err(
                    format!("join: list elements must be Str, got {}", v.type_name()),
                    span,
                )
            })
        })
        .collect();
    Ok(Value::Str(Arc::from(parts?.join(sep).as_str())))
}

pub(super) fn register(env: &mut Env) {
    env.bind("trim".into(), mk("trim", 1, bi_trim));
    env.bind("trim_start".into(), mk("trim_start", 1, bi_trim_start));
    env.bind("trim_end".into(), mk("trim_end", 1, bi_trim_end));
    env.bind("upper".into(), mk("upper", 1, bi_upper));
    env.bind("lower".into(), mk("lower", 1, bi_lower));
    env.bind("lines".into(), mk("lines", 1, bi_lines));
    env.bind("chars".into(), mk("chars", 1, bi_chars));
    env.bind("byte_len".into(), mk("byte_len", 1, bi_byte_len));
    env.bind("split".into(), mk("split", 2, bi_split));
    env.bind("join".into(), mk("join", 2, bi_join));
    env.bind("replace".into(), mk("replace", 3, str_extra::bi_replace));
    env.bind(
        "replace_all".into(),
        mk("replace_all", 3, str_extra::bi_replace_all),
    );
    env.bind("repeat".into(), mk("repeat", 2, str_extra::bi_repeat));
    env.bind("starts?".into(), mk("starts?", 2, str_extra::bi_starts));
    env.bind("ends?".into(), mk("ends?", 2, str_extra::bi_ends));
    env.bind("pad_left".into(), mk("pad_left", 2, str_extra::bi_pad_left));
    env.bind(
        "pad_right".into(),
        mk("pad_right", 2, str_extra::bi_pad_right),
    );
}
