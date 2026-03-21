use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::repo::{REPOSPACES, WorktreeInfo, WorktreeStatus, repo_id, run_git_in};
use super::repo_lock::{
    acquire_lock, check_conflicts, parse_lock_request, release_locks_for_agent,
};

fn get_ts() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub fn bi_checkout(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let agent = args[1]
        .str_field("agent")
        .ok_or_else(|| LxError::type_err("repo.checkout: opts.agent must be Str", span))?
        .to_string();
    let lock_vals = args[1]
        .list_field("locks")
        .ok_or_else(|| LxError::type_err("repo.checkout: opts.locks must be List", span))?
        .to_vec();
    let mut locks = Vec::new();
    for lv in &lock_vals {
        locks.push(parse_lock_request(lv, span)?);
    }
    let (root, wt_path, branch);
    {
        let mut rp = REPOSPACES
            .get_mut(&id)
            .ok_or_else(|| LxError::runtime("repo.checkout: not found", span))?;
        for lk in &locks {
            let conflicts = check_conflicts(&rp.locks, lk);
            if !conflicts.is_empty() {
                let list: Vec<Value> = conflicts
                    .iter()
                    .map(|c| {
                        record! {
                            "scope" => Value::Str(Arc::from(super::repo::lock_scope_str(&c.scope))),
                            "holder" => Value::Str(Arc::from(c.holder.as_str())),
                            "mode" => Value::Str(Arc::from(super::repo::lock_mode_str(&c.mode))),
                        }
                    })
                    .collect();
                return Ok(Value::Err(Box::new(record! {
                    "conflicts" => Value::List(Arc::new(list)),
                })));
            }
        }
        let ts = get_ts();
        branch = format!("repo-{agent}-{ts}");
        wt_path = format!("{}/.lx/worktrees/{}-{}", rp.root, agent, ts);
        root = rp.root.clone();
        let wt_dir = format!("{}/.lx/worktrees", rp.root);
        let _ = std::fs::create_dir_all(&wt_dir);
        for lk in locks.clone() {
            acquire_lock(&mut rp, lk);
        }
        rp.worktrees.insert(
            agent.clone(),
            WorktreeInfo {
                path: wt_path.clone(),
                branch: branch.clone(),
                agent: agent.clone(),
                status: WorktreeStatus::Active,
            },
        );
    }
    match run_git_in(&root, &["worktree", "add", &wt_path, "-b", &branch]) {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            if let Some(mut rp) = REPOSPACES.get_mut(&id) {
                release_locks_for_agent(&mut rp, &agent);
                rp.worktrees.shift_remove(&agent);
            }
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(stderr.trim())))));
        }
        Err(e) => {
            if let Some(mut rp) = REPOSPACES.get_mut(&id) {
                release_locks_for_agent(&mut rp, &agent);
                rp.worktrees.shift_remove(&agent);
            }
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                format!("git: {e}").as_str(),
            )))));
        }
    }
    Ok(Value::Ok(Box::new(record! {
        "path" => Value::Str(Arc::from(wt_path.as_str())),
        "branch" => Value::Str(Arc::from(branch.as_str())),
        "agent" => Value::Str(Arc::from(agent.as_str())),
        "__worktree_agent" => Value::Str(Arc::from(agent.as_str())),
    })))
}

pub fn bi_submit(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let agent = args[1]
        .str_field("__worktree_agent")
        .ok_or_else(|| LxError::type_err("repo.submit: expected worktree handle", span))?
        .to_string();
    let (root, branch, wt_path);
    {
        let mut rp = REPOSPACES
            .get_mut(&id)
            .ok_or_else(|| LxError::runtime("repo.submit: not found", span))?;
        {
            let wt = rp.worktrees.get(&agent).ok_or_else(|| {
                LxError::runtime(
                    format!("repo.submit: no worktree for agent `{agent}`"),
                    span,
                )
            })?;
            if wt.status != WorktreeStatus::Active {
                return Err(LxError::runtime(
                    "repo.submit: worktree is not active",
                    span,
                ));
            }
            branch = wt.branch.clone();
            wt_path = wt.path.clone();
        }
        if let Some(wt) = rp.worktrees.get_mut(&agent) {
            wt.status = WorktreeStatus::Merging;
        }
        root = rp.root.clone();
    }
    let merge_out = run_git_in(&root, &["merge", "--no-edit", &branch]);
    match merge_out {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            if let Some(mut rp) = REPOSPACES.get_mut(&id)
                && let Some(wt) = rp.worktrees.get_mut(&agent)
            {
                wt.status = WorktreeStatus::Active;
            }
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(stderr.trim())))));
        }
        Err(e) => {
            if let Some(mut rp) = REPOSPACES.get_mut(&id)
                && let Some(wt) = rp.worktrees.get_mut(&agent)
            {
                wt.status = WorktreeStatus::Active;
            }
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                format!("git: {e}").as_str(),
            )))));
        }
    }
    let files: Vec<Value> = run_git_in(&root, &["diff", "--name-only", "HEAD~1"])
        .ok()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| Value::Str(Arc::from(l)))
                .collect()
        })
        .unwrap_or_default();
    let _ = run_git_in(&root, &["worktree", "remove", &wt_path]);
    let _ = run_git_in(&root, &["branch", "-d", &branch]);
    let watchers;
    {
        let mut rp = REPOSPACES
            .get_mut(&id)
            .ok_or_else(|| LxError::runtime("repo.submit: repospace disappeared", span))?;
        release_locks_for_agent(&mut rp, &agent);
        rp.worktrees.shift_remove(&agent);
        watchers = rp.watchers.clone();
    }
    let change = record! {
        "agent" => Value::Str(Arc::from(agent.as_str())),
        "files" => Value::List(Arc::new(files.clone())),
        "branch" => Value::Str(Arc::from(branch.as_str())),
    };
    for w in &watchers {
        let _ = call_value_sync(w, change.clone(), span, ctx);
    }
    Ok(Value::Ok(Box::new(record! {
        "merged" => Value::Bool(true),
        "files" => Value::List(Arc::new(files)),
        "conflicts" => Value::List(Arc::new(vec![])),
    })))
}

pub fn bi_rebase(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let agent = args[1]
        .str_field("__worktree_agent")
        .ok_or_else(|| LxError::type_err("repo.rebase: expected worktree handle", span))?
        .to_string();
    let (root, wt_path);
    {
        let rp = REPOSPACES
            .get(&id)
            .ok_or_else(|| LxError::runtime("repo.rebase: not found", span))?;
        let wt = rp.worktrees.get(&agent).ok_or_else(|| {
            LxError::runtime(
                format!("repo.rebase: no worktree for agent `{agent}`"),
                span,
            )
        })?;
        root = rp.root.clone();
        wt_path = wt.path.clone();
    }
    let main_branch = match run_git_in(&root, &["rev-parse", "--abbrev-ref", "HEAD"]) {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "main".to_string(),
    };
    match run_git_in(&wt_path, &["rebase", &main_branch]) {
        Ok(out) if out.status.success() => Ok(Value::Ok(Box::new(Value::Unit))),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Ok(Value::Err(Box::new(Value::Str(Arc::from(stderr.trim())))))
        }
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("git: {e}").as_str(),
        ))))),
    }
}

pub fn bi_abandon(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let agent = args[1]
        .str_field("__worktree_agent")
        .ok_or_else(|| LxError::type_err("repo.abandon: expected worktree handle", span))?
        .to_string();
    let (root, wt_path, branch);
    {
        let mut rp = REPOSPACES
            .get_mut(&id)
            .ok_or_else(|| LxError::runtime("repo.abandon: not found", span))?;
        let wt = rp.worktrees.get(&agent).ok_or_else(|| {
            LxError::runtime(
                format!("repo.abandon: no worktree for agent `{agent}`"),
                span,
            )
        })?;
        root = rp.root.clone();
        wt_path = wt.path.clone();
        branch = wt.branch.clone();
        release_locks_for_agent(&mut rp, &agent);
        rp.worktrees.shift_remove(&agent);
    }
    let _ = run_git_in(&root, &["worktree", "remove", "--force", &wt_path]);
    let _ = run_git_in(&root, &["branch", "-D", &branch]);
    Ok(Value::Ok(Box::new(Value::Unit)))
}
