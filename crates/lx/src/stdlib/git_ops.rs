use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::git::{get_bool, get_str, git_err, git_err_from, git_ok, run_git, str_val};

pub fn bi_add(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let paths = args[0]
        .as_list()
        .ok_or_else(|| LxError::type_err("git.add expects List of Str paths", span))?;
    let mut cmd_args = vec!["add"];
    let path_strs: Vec<String> = paths
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| LxError::type_err("git.add: each path must be Str", span))
                .map(String::from)
        })
        .collect::<Result<_, _>>()?;
    let path_refs: Vec<&str> = path_strs.iter().map(|s| s.as_str()).collect();
    cmd_args.extend(&path_refs);
    match run_git(&cmd_args) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_commit(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let msg = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.commit expects Str message", span))?;
    match run_git(&["commit", "-m", msg]) {
        Ok(out) if out.status.success() => {
            let hash = get_head_hash();
            let mut f = IndexMap::new();
            f.insert("hash".into(), str_val(&hash));
            Ok(git_ok(Value::Record(Arc::new(f))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_commit_with(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let opts = match &args[0] {
        Value::Record(r) => r.as_ref(),
        _ => return Err(LxError::type_err("git.commit_with expects Record", span)),
    };
    let msg = get_str(opts, "msg")
        .ok_or_else(|| LxError::runtime("git.commit_with: missing 'msg'", span))?;
    let mut cmd_args = vec!["commit".to_string(), "-m".to_string(), msg.to_string()];
    if let Some(author) = get_str(opts, "author") {
        cmd_args.push(format!("--author={author}"));
    }
    if get_bool(opts, "amend").unwrap_or(false) {
        cmd_args.push("--amend".to_string());
    }
    if get_bool(opts, "allow_empty").unwrap_or(false) {
        cmd_args.push("--allow-empty".to_string());
    }
    let refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    match run_git(&refs) {
        Ok(out) if out.status.success() => {
            let hash = get_head_hash();
            let mut f = IndexMap::new();
            f.insert("hash".into(), str_val(&hash));
            Ok(git_ok(Value::Record(Arc::new(f))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn get_head_hash() -> String {
    match run_git(&["rev-parse", "HEAD"]) {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => String::new(),
    }
}

pub fn bi_tag(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.tag expects Str name", span))?;
    match run_git(&["tag", name]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_tag_with(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.tag_with: expects Str name", span))?;
    let opts = match &args[1] {
        Value::Record(r) => r.as_ref(),
        _ => return Err(LxError::type_err("git.tag_with: opts must be Record", span)),
    };
    let mut cmd_args = vec!["tag".to_string(), name.to_string()];
    if let Some(msg) = get_str(opts, "msg") {
        cmd_args.push("-m".to_string());
        cmd_args.push(msg.to_string());
    }
    if let Some(r) = get_str(opts, "ref") {
        cmd_args.push(r.to_string());
    }
    let refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    match run_git(&refs) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}
