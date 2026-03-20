use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::git::{git_err, git_err_from, git_ok, run_git, str_val};

pub fn bi_worktree_add(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.worktree_add: path must be Str", span))?;
    let branch = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.worktree_add: branch must be Str", span))?;
    match run_git(&["worktree", "add", path, "-b", branch]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_worktree_remove(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.worktree_remove: path must be Str", span))?;
    match run_git(&["worktree", "remove", path]) {
        Ok(out) if out.status.success() => Ok(git_ok(Value::Unit)),
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_worktree_list(
    args: &[Value],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["worktree", "list", "--porcelain"]) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let mut entries: Vec<Value> = Vec::new();
            let mut path = "";
            let mut head = "";
            let mut branch = "detached";
            for line in raw.lines() {
                if line.is_empty() {
                    if !path.is_empty() {
                        entries.push(record! {
                            "path" => str_val(path),
                            "head" => str_val(head),
                            "branch" => str_val(branch),
                        });
                    }
                    path = "";
                    head = "";
                    branch = "detached";
                } else if let Some(p) = line.strip_prefix("worktree ") {
                    path = p;
                } else if let Some(h) = line.strip_prefix("HEAD ") {
                    head = h;
                } else if let Some(b) = line.strip_prefix("branch ") {
                    branch = b.strip_prefix("refs/heads/").unwrap_or(b);
                }
            }
            if !path.is_empty() {
                entries.push(record! {
                    "path" => str_val(path),
                    "head" => str_val(head),
                    "branch" => str_val(branch),
                });
            }
            Ok(git_ok(Value::List(Arc::new(entries))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}
