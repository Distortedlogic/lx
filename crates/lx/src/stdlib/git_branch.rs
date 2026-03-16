use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::git::{get_bool, get_str, git_err, git_err_from, git_ok, run_git, str_val};

pub fn bi_create_branch(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.create_branch expects Str", span))?;
    match run_git(&["branch", name]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_create_branch_at(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.create_branch_at: expects Str name", span))?;
    let ref_name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.create_branch_at: expects Str ref", span))?;
    match run_git(&["branch", name, ref_name]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_delete_branch(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.delete_branch expects Str", span))?;
    match run_git(&["branch", "-d", name]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_checkout(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let ref_name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.checkout expects Str ref", span))?;
    match run_git(&["checkout", ref_name]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_checkout_create(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.checkout_create expects Str", span))?;
    match run_git(&["checkout", "-b", name]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_merge(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let ref_name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.merge expects Str ref", span))?;
    match run_git(&["merge", "--no-edit", ref_name]) {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{stdout}\n{stderr}");
            let fast_forward =
                combined.contains("Fast-forward") || combined.contains("fast-forward");
            let conflict_files = extract_conflict_files(&combined);
            let merged = out.status.success();
            let mut f = IndexMap::new();
            f.insert("fast_forward".into(), Value::Bool(fast_forward));
            f.insert(
                "conflicts".into(),
                Value::List(Arc::new(
                    conflict_files.iter().map(|s| str_val(s)).collect(),
                )),
            );
            f.insert("merged".into(), Value::Bool(merged));
            Ok(git_ok(Value::Record(Arc::new(f))))
        }
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn extract_conflict_files(output: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in output.lines() {
        if let Some(path) = line
            .strip_prefix("CONFLICT ")
            .and_then(|rest| rest.rsplit_once("Merge conflict in "))
        {
            files.push(path.1.trim().to_string());
        }
    }
    files
}

pub fn bi_stash(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["stash"]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_stash_with(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let msg = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.stash_with expects Str message", span))?;
    match run_git(&["stash", "push", "-m", msg]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_stash_pop(
    args: &[Value],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["stash", "pop"]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_stash_list(
    args: &[Value],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["stash", "list", "--format=%gd\x1f%gs\x1f%s"]) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let entries: Vec<Value> = raw
                .lines()
                .filter(|l| !l.is_empty())
                .enumerate()
                .map(|(i, line)| {
                    let parts: Vec<&str> = line.split('\x1f').collect();
                    let msg = parts.get(2).unwrap_or(&"");
                    let branch = parts.get(1).unwrap_or(&"");
                    let mut f = IndexMap::new();
                    f.insert("index".into(), Value::Int(BigInt::from(i)));
                    f.insert("msg".into(), str_val(msg));
                    f.insert("branch".into(), str_val(branch));
                    Value::Record(Arc::new(f))
                })
                .collect();
            Ok(git_ok(Value::List(Arc::new(entries))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_stash_drop(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let idx = args[0]
        .as_int()
        .ok_or_else(|| LxError::type_err("git.stash_drop expects Int index", span))?;
    let idx_i64 = idx
        .to_i64()
        .ok_or_else(|| LxError::runtime("git.stash_drop: index too large", span))?;
    let ref_str = format!("stash@{{{idx_i64}}}");
    match run_git(&["stash", "drop", &ref_str]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_fetch(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let remote = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.fetch expects Str remote", span))?;
    match run_git(&["fetch", remote]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_pull(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["pull"]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_push(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["push"]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_push_with(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let opts = match &args[0] {
        Value::Record(r) => r.as_ref(),
        _ => return Err(LxError::type_err("git.push_with expects Record", span)),
    };
    let mut cmd_args = vec!["push".to_string()];
    if get_bool(opts, "force").unwrap_or(false) {
        cmd_args.push("--force".to_string());
    }
    if get_bool(opts, "set_upstream").unwrap_or(false) {
        cmd_args.push("-u".to_string());
    }
    let remote = get_str(opts, "remote").unwrap_or("origin");
    cmd_args.push(remote.to_string());
    if let Some(branch) = get_str(opts, "branch") {
        cmd_args.push(branch.to_string());
    }
    let refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    match run_git(&refs) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}
