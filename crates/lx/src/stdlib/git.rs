use std::process::{Command, Output};
use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("status".into(), mk("git.status", 1, bi_status));
    m.insert("branch".into(), mk("git.branch", 1, bi_branch));
    m.insert("root".into(), mk("git.root", 1, bi_root));
    m.insert("is_repo".into(), mk("git.is_repo", 1, bi_is_repo));
    m.insert(
        "branches".into(),
        mk("git.branches", 1, super::git_status::bi_branches),
    );
    m.insert(
        "remotes".into(),
        mk("git.remotes", 1, super::git_status::bi_remotes),
    );
    m.insert("log".into(), mk("git.log", 1, super::git_log::bi_log));
    m.insert("show".into(), mk("git.show", 1, super::git_log::bi_show));
    m.insert("blame".into(), mk("git.blame", 1, super::git_log::bi_blame));
    m.insert(
        "blame_range".into(),
        mk("git.blame_range", 3, super::git_log::bi_blame_range),
    );
    m.insert("diff".into(), mk("git.diff", 1, super::git_diff::bi_diff));
    m.insert(
        "diff_stat".into(),
        mk("git.diff_stat", 1, super::git_diff::bi_diff_stat),
    );
    m.insert("grep".into(), mk("git.grep", 2, super::git_diff::bi_grep));
    m.insert("add".into(), mk("git.add", 1, super::git_ops::bi_add));
    m.insert(
        "commit".into(),
        mk("git.commit", 1, super::git_ops::bi_commit),
    );
    m.insert(
        "commit_with".into(),
        mk("git.commit_with", 1, super::git_ops::bi_commit_with),
    );
    m.insert("tag".into(), mk("git.tag", 1, super::git_ops::bi_tag));
    m.insert(
        "tag_with".into(),
        mk("git.tag_with", 2, super::git_ops::bi_tag_with),
    );
    m.insert(
        "create_branch".into(),
        mk("git.create_branch", 1, super::git_branch::bi_create_branch),
    );
    m.insert(
        "create_branch_at".into(),
        mk(
            "git.create_branch_at",
            2,
            super::git_branch::bi_create_branch_at,
        ),
    );
    m.insert(
        "delete_branch".into(),
        mk("git.delete_branch", 1, super::git_branch::bi_delete_branch),
    );
    m.insert(
        "checkout".into(),
        mk("git.checkout", 1, super::git_branch::bi_checkout),
    );
    m.insert(
        "checkout_create".into(),
        mk(
            "git.checkout_create",
            1,
            super::git_branch::bi_checkout_create,
        ),
    );
    m.insert(
        "merge".into(),
        mk("git.merge", 1, super::git_branch::bi_merge),
    );
    m.insert(
        "stash".into(),
        mk("git.stash", 1, super::git_branch::bi_stash),
    );
    m.insert(
        "stash_with".into(),
        mk("git.stash_with", 1, super::git_branch::bi_stash_with),
    );
    m.insert(
        "stash_pop".into(),
        mk("git.stash_pop", 1, super::git_branch::bi_stash_pop),
    );
    m.insert(
        "stash_list".into(),
        mk("git.stash_list", 1, super::git_branch::bi_stash_list),
    );
    m.insert(
        "stash_drop".into(),
        mk("git.stash_drop", 1, super::git_branch::bi_stash_drop),
    );
    m.insert(
        "fetch".into(),
        mk("git.fetch", 1, super::git_branch::bi_fetch),
    );
    m.insert("pull".into(), mk("git.pull", 1, super::git_branch::bi_pull));
    m.insert("push".into(), mk("git.push", 1, super::git_branch::bi_push));
    m.insert(
        "push_with".into(),
        mk("git.push_with", 1, super::git_branch::bi_push_with),
    );
    m.insert(
        "worktree_add".into(),
        mk("git.worktree_add", 2, super::git_worktree::bi_worktree_add),
    );
    m.insert(
        "worktree_remove".into(),
        mk(
            "git.worktree_remove",
            1,
            super::git_worktree::bi_worktree_remove,
        ),
    );
    m.insert(
        "worktree_list".into(),
        mk(
            "git.worktree_list",
            1,
            super::git_worktree::bi_worktree_list,
        ),
    );
    m
}

pub(super) fn run_git(args: &[&str]) -> std::io::Result<Output> {
    Command::new("git").args(args).output()
}

pub(super) fn git_ok(v: Value) -> Value {
    Value::Ok(Box::new(v))
}

pub(super) fn git_err(msg: &str) -> Value {
    Value::Err(Box::new(Value::Str(Arc::from(msg))))
}

pub(super) fn git_err_from(out: &Output) -> Value {
    let stderr = String::from_utf8_lossy(&out.stderr);
    let msg = stderr.trim();
    if msg.is_empty() {
        git_err(&format!(
            "git exited with code {}",
            out.status.code().unwrap_or(-1)
        ))
    } else {
        git_err(msg)
    }
}

pub(super) fn str_val(s: &str) -> Value {
    Value::Str(Arc::from(s))
}

pub(super) fn int_val(n: i64) -> Value {
    Value::Int(BigInt::from(n))
}

pub(super) fn get_str<'a>(r: &'a IndexMap<String, Value>, key: &str) -> Option<&'a str> {
    r.get(key).and_then(|v| v.as_str())
}

pub(super) fn get_int(r: &IndexMap<String, Value>, key: &str) -> Option<i64> {
    use num_traits::ToPrimitive;
    r.get(key).and_then(|v| v.as_int()).and_then(|n| n.to_i64())
}

pub(super) fn get_bool(r: &IndexMap<String, Value>, key: &str) -> Option<bool> {
    r.get(key).and_then(|v| v.as_bool())
}

fn bi_status(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["status", "--porcelain=v2", "--branch", "-z"]) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            Ok(git_ok(super::git_status::parse_status(&raw)))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn bi_branch(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["rev-parse", "--abbrev-ref", "HEAD"]) {
        Ok(out) if out.status.success() => {
            let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(git_ok(str_val(&name)))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn bi_root(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["rev-parse", "--show-toplevel"]) {
        Ok(out) if out.status.success() => {
            let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(git_ok(str_val(&root)))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn bi_is_repo(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["rev-parse", "--is-inside-work-tree"]) {
        Ok(out) => Ok(Value::Bool(
            out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true",
        )),
        Err(_) => Ok(Value::Bool(false)),
    }
}
