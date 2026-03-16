use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("match".into(), mk("re.match", 2, bi_match));
    m.insert("find_all".into(), mk("re.find_all", 2, bi_find_all));
    m.insert("replace".into(), mk("re.replace", 3, bi_replace));
    m.insert(
        "replace_all".into(),
        mk("re.replace_all", 3, bi_replace_all),
    );
    m.insert("split".into(), mk("re.split", 2, bi_split));
    m.insert("is_match".into(), mk("re.is_match", 2, bi_is_match));
    m
}

enum RePattern<'a> {
    Compiled(&'a regex::Regex),
    Raw(&'a str),
}

fn get_pattern(v: &Value, span: Span) -> Result<RePattern<'_>, LxError> {
    match v {
        Value::Regex(r) => Ok(RePattern::Compiled(r)),
        Value::Str(s) => Ok(RePattern::Raw(s.as_ref())),
        other => Err(LxError::type_err(
            format!(
                "re: expected Regex or Str pattern, got {}",
                other.type_name()
            ),
            span,
        )),
    }
}

fn to_regex<'a>(
    pat: &'a RePattern<'a>,
    span: Span,
) -> Result<std::borrow::Cow<'a, regex::Regex>, LxError> {
    match pat {
        RePattern::Compiled(r) => Ok(std::borrow::Cow::Borrowed(r)),
        RePattern::Raw(s) => {
            let re = regex::Regex::new(s)
                .map_err(|e| LxError::runtime(format!("re: invalid pattern: {e}"), span))?;
            Ok(std::borrow::Cow::Owned(re))
        }
    }
}

fn bi_match(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pat = get_pattern(&args[0], span)?;
    let input = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("re.match expects Str input", span))?;
    let re = to_regex(&pat, span)?;
    match re.find(input) {
        Some(m) => {
            let mut fields = IndexMap::new();
            fields.insert("text".into(), Value::Str(Arc::from(m.as_str())));
            fields.insert("start".into(), Value::Int(m.start().into()));
            fields.insert("end".into(), Value::Int(m.end().into()));
            Ok(Value::Some(Box::new(Value::Record(Arc::new(fields)))))
        }
        None => Ok(Value::None),
    }
}

fn bi_find_all(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pat = get_pattern(&args[0], span)?;
    let input = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("re.find_all expects Str input", span))?;
    let re = to_regex(&pat, span)?;
    let matches: Vec<Value> = re
        .find_iter(input)
        .map(|m| Value::Str(Arc::from(m.as_str())))
        .collect();
    Ok(Value::List(Arc::new(matches)))
}

fn bi_replace(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pat = get_pattern(&args[0], span)?;
    let replacement = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("re.replace expects Str replacement", span))?;
    let input = args[2]
        .as_str()
        .ok_or_else(|| LxError::type_err("re.replace expects Str input", span))?;
    let re = to_regex(&pat, span)?;
    let result = re.replace(input, replacement);
    Ok(Value::Str(Arc::from(result.as_ref())))
}

fn bi_replace_all(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pat = get_pattern(&args[0], span)?;
    let replacement = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("re.replace_all expects Str replacement", span))?;
    let input = args[2]
        .as_str()
        .ok_or_else(|| LxError::type_err("re.replace_all expects Str input", span))?;
    let re = to_regex(&pat, span)?;
    let result = re.replace_all(input, replacement);
    Ok(Value::Str(Arc::from(result.as_ref())))
}

fn bi_split(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pat = get_pattern(&args[0], span)?;
    let input = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("re.split expects Str input", span))?;
    let re = to_regex(&pat, span)?;
    let parts: Vec<Value> = re.split(input).map(|s| Value::Str(Arc::from(s))).collect();
    Ok(Value::List(Arc::new(parts)))
}

fn bi_is_match(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pat = get_pattern(&args[0], span)?;
    let input = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("re.is_match expects Str input", span))?;
    let re = to_regex(&pat, span)?;
    Ok(Value::Bool(re.is_match(input)))
}
