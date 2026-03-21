use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

#[derive(Clone)]
pub(super) enum LockScope {
    File(String),
    Folder(String),
    Repo,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum LockMode {
    Read,
    Write,
}

#[derive(Clone)]
pub(super) struct Lock {
    pub scope: LockScope,
    pub mode: LockMode,
    pub holder: String,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum WorktreeStatus {
    Active,
    Merging,
}

pub(super) struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub agent: String,
    pub status: WorktreeStatus,
}

pub(super) struct Repospace {
    pub root: String,
    pub merge_strategy: String,
    pub locks: Vec<Lock>,
    pub worktrees: IndexMap<String, WorktreeInfo>,
    pub watchers: Vec<Value>,
}

pub(super) static REPOSPACES: LazyLock<DashMap<u64, Repospace>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub(super) fn repo_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__repo_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("repo: expected Repospace handle", span)),
        _ => Err(LxError::type_err("repo: expected Record", span)),
    }
}

fn make_handle(id: u64, name: &str) -> Value {
    record! {
        "__repo_id" => Value::Int(BigInt::from(id)),
        "name" => Value::Str(Arc::from(name)),
    }
}

pub(super) fn run_git_in(dir: &str, args: &[&str]) -> std::io::Result<Output> {
    Command::new("git").arg("-C").arg(dir).args(args).output()
}

pub(super) fn lock_scope_str(scope: &LockScope) -> &str {
    match scope {
        LockScope::File(_) => "file",
        LockScope::Folder(_) => "folder",
        LockScope::Repo => "repo",
    }
}

pub(super) fn lock_scope_path(scope: &LockScope) -> &str {
    match scope {
        LockScope::File(p) | LockScope::Folder(p) => p,
        LockScope::Repo => "",
    }
}

pub(super) fn lock_mode_str(mode: &LockMode) -> &str {
    match mode {
        LockMode::Read => "read",
        LockMode::Write => "write",
    }
}

fn wt_status_str(s: WorktreeStatus) -> &'static str {
    match s {
        WorktreeStatus::Active => "active",
        WorktreeStatus::Merging => "merging",
    }
}

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("repo.create", 2, bi_create));
    m.insert("status".into(), mk("repo.status", 1, bi_status));
    m.insert("worktrees".into(), mk("repo.worktrees", 1, bi_worktrees));
    m.insert("locks".into(), mk("repo.locks", 1, bi_locks_fn));
    m.insert("on_change".into(), mk("repo.on_change", 2, bi_on_change));
    m.insert(
        "checkout".into(),
        mk("repo.checkout", 2, super::repo_worktree::bi_checkout),
    );
    m.insert(
        "submit".into(),
        mk("repo.submit", 2, super::repo_worktree::bi_submit),
    );
    m.insert(
        "rebase".into(),
        mk("repo.rebase", 2, super::repo_worktree::bi_rebase),
    );
    m.insert(
        "abandon".into(),
        mk("repo.abandon", 2, super::repo_worktree::bi_abandon),
    );
    m.insert("lock".into(), mk("repo.lock", 2, super::repo_lock::bi_lock));
    m.insert(
        "unlock".into(),
        mk("repo.unlock", 2, super::repo_lock::bi_unlock),
    );
    m.insert(
        "try_lock".into(),
        mk("repo.try_lock", 2, super::repo_lock::bi_try_lock),
    );
    m
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("repo.create: name must be Str", span))?;
    let root = args[1]
        .str_field("root")
        .ok_or_else(|| LxError::type_err("repo.create: opts.root must be Str", span))?
        .to_string();
    let strategy = args[1]
        .str_field("merge_strategy")
        .unwrap_or("rebase")
        .to_string();
    match run_git_in(&root, &["rev-parse", "--is-inside-work-tree"]) {
        Ok(out) if out.status.success() => {}
        _ => {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                "repo.create: root is not a git repository",
            )))));
        }
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    REPOSPACES.insert(
        id,
        Repospace {
            root,
            merge_strategy: strategy,
            locks: Vec::new(),
            worktrees: IndexMap::new(),
            watchers: Vec::new(),
        },
    );
    Ok(Value::Ok(Box::new(make_handle(id, name))))
}

fn bi_status(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let rp = REPOSPACES
        .get(&id)
        .ok_or_else(|| LxError::runtime("repo.status: not found", span))?;
    Ok(record! {
        "root" => Value::Str(Arc::from(rp.root.as_str())),
        "worktree_count" => Value::Int(BigInt::from(rp.worktrees.len())),
        "lock_count" => Value::Int(BigInt::from(rp.locks.len())),
        "merge_strategy" => Value::Str(Arc::from(rp.merge_strategy.as_str())),
    })
}

fn bi_worktrees(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let rp = REPOSPACES
        .get(&id)
        .ok_or_else(|| LxError::runtime("repo.worktrees: not found", span))?;
    let list: Vec<Value> = rp
        .worktrees
        .values()
        .map(|wt| {
            record! {
                "path" => Value::Str(Arc::from(wt.path.as_str())),
                "branch" => Value::Str(Arc::from(wt.branch.as_str())),
                "agent" => Value::Str(Arc::from(wt.agent.as_str())),
                "status" => Value::Str(Arc::from(wt_status_str(wt.status))),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(list)))
}

fn bi_locks_fn(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let rp = REPOSPACES
        .get(&id)
        .ok_or_else(|| LxError::runtime("repo.locks: not found", span))?;
    let list: Vec<Value> = rp
        .locks
        .iter()
        .map(|lk| {
            record! {
                "scope" => Value::Str(Arc::from(lock_scope_str(&lk.scope))),
                "path" => Value::Str(Arc::from(lock_scope_path(&lk.scope))),
                "mode" => Value::Str(Arc::from(lock_mode_str(&lk.mode))),
                "holder" => Value::Str(Arc::from(lk.holder.as_str())),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(list)))
}

fn bi_on_change(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let handler = args[1].clone();
    let mut rp = REPOSPACES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("repo.on_change: not found", span))?;
    rp.watchers.push(handler);
    Ok(Value::Unit)
}
