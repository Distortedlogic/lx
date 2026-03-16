use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::git::{get_bool, get_int, get_str, git_err, git_err_from, git_ok, run_git, str_val};

pub(super) fn parse_unified_diff(raw: &str) -> Vec<Value> {
    super::git_diff_parse::parse_unified_diff(raw)
}

pub fn bi_diff(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let opts = match &args[0] {
        Value::Record(r) => r.as_ref().clone(),
        Value::Unit => IndexMap::new(),
        _ => return Err(LxError::type_err("git.diff expects Record opts or ()", span)),
    };
    let context = get_int(&opts, "context").unwrap_or(3);
    let mut cmd_args = vec!["diff".to_string(), "--no-color".to_string()];
    cmd_args.push(format!("-U{context}"));
    if get_bool(&opts, "staged").unwrap_or(false) {
        cmd_args.push("--cached".to_string());
    }
    if let Some(range) = get_str(&opts, "range") {
        cmd_args.push(range.to_string());
    } else if let Some(r) = get_str(&opts, "ref") {
        cmd_args.push(r.to_string());
    }
    if let Some(path) = get_str(&opts, "path") {
        cmd_args.push("--".to_string());
        cmd_args.push(path.to_string());
    }
    let refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    match run_git(&refs) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            Ok(git_ok(Value::List(Arc::new(
                super::git_diff_parse::parse_unified_diff(&raw),
            ))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_diff_stat(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let opts = match &args[0] {
        Value::Record(r) => r.as_ref().clone(),
        Value::Unit => IndexMap::new(),
        _ => {
            return Err(LxError::type_err(
                "git.diff_stat expects Record opts or ()",
                span,
            ))
        }
    };
    let mut cmd_args = vec!["diff".to_string(), "--numstat".to_string()];
    if get_bool(&opts, "staged").unwrap_or(false) {
        cmd_args.push("--cached".to_string());
    }
    if let Some(range) = get_str(&opts, "range") {
        cmd_args.push(range.to_string());
    } else if let Some(r) = get_str(&opts, "ref") {
        cmd_args.push(r.to_string());
    }
    if let Some(path) = get_str(&opts, "path") {
        cmd_args.push("--".to_string());
        cmd_args.push(path.to_string());
    }
    let refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    match run_git(&refs) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let stats: Vec<Value> = raw
                .lines()
                .filter(|l| !l.is_empty())
                .filter_map(parse_numstat_line)
                .collect();
            Ok(git_ok(Value::List(Arc::new(stats))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn parse_numstat_line(line: &str) -> Option<Value> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 3 {
        return None;
    }
    let additions: i64 = parts[0].parse().unwrap_or(0);
    let deletions: i64 = parts[1].parse().unwrap_or(0);
    let mut f = IndexMap::new();
    f.insert("path".into(), str_val(parts[2]));
    f.insert("additions".into(), Value::Int(BigInt::from(additions)));
    f.insert("deletions".into(), Value::Int(BigInt::from(deletions)));
    Some(Value::Record(Arc::new(f)))
}

pub fn bi_grep(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pattern = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.grep expects Str pattern", span))?;
    let opts = match &args[1] {
        Value::Record(r) => r.as_ref().clone(),
        Value::Unit => IndexMap::new(),
        _ => return Err(LxError::type_err("git.grep: opts must be Record or ()", span)),
    };
    let mut cmd_args = vec!["grep".to_string(), "-n".to_string()];
    if get_bool(&opts, "ignore_case").unwrap_or(false) {
        cmd_args.push("-i".to_string());
    }
    cmd_args.push(pattern.to_string());
    if let Some(r) = get_str(&opts, "ref") {
        cmd_args.push(r.to_string());
    }
    if let Some(path) = get_str(&opts, "path") {
        cmd_args.push("--".to_string());
        cmd_args.push(path.to_string());
    }
    let refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    match run_git(&refs) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let hits: Vec<Value> = raw
                .lines()
                .filter(|l| !l.is_empty())
                .filter_map(parse_grep_line)
                .collect();
            Ok(git_ok(Value::List(Arc::new(hits))))
        }
        Ok(out) if out.status.code() == Some(1) => {
            Ok(git_ok(Value::List(Arc::new(Vec::new()))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn parse_grep_line(line: &str) -> Option<Value> {
    let (path, rest) = line.split_once(':')?;
    let (line_num, content) = rest.split_once(':')?;
    let mut f = IndexMap::new();
    f.insert("path".into(), str_val(path));
    f.insert(
        "line".into(),
        Value::Int(BigInt::from(line_num.parse::<i64>().unwrap_or(0))),
    );
    f.insert("content".into(), str_val(content));
    Some(Value::Record(Arc::new(f)))
}
