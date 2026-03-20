use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::repo::{Lock, LockMode, LockScope, REPOSPACES, Repospace, repo_id};

pub(super) fn scope_overlaps(a: &LockScope, b: &LockScope) -> bool {
    match (a, b) {
        (LockScope::File(a), LockScope::File(b)) => a == b,
        (LockScope::File(a), LockScope::Folder(b)) => a.starts_with(b.as_str()),
        (LockScope::Folder(a), LockScope::File(b)) => b.starts_with(a.as_str()),
        (LockScope::Folder(a), LockScope::Folder(b)) => {
            a.starts_with(b.as_str()) || b.starts_with(a.as_str())
        }
        (LockScope::Repo, _) | (_, LockScope::Repo) => true,
    }
}

pub(super) fn check_conflicts(existing: &[Lock], requested: &Lock) -> Vec<Lock> {
    existing
        .iter()
        .filter(|lk| {
            scope_overlaps(&lk.scope, &requested.scope)
                && !(lk.mode == LockMode::Read && requested.mode == LockMode::Read)
        })
        .cloned()
        .collect()
}

pub(super) fn acquire_lock(repospace: &mut Repospace, lock: Lock) {
    repospace.locks.push(lock);
}

pub(super) fn release_locks_for_agent(repospace: &mut Repospace, agent: &str) {
    repospace.locks.retain(|lk| lk.holder != agent);
}

pub(super) fn parse_lock_request(val: &Value, span: Span) -> Result<Lock, LxError> {
    let path = val.str_field("path").unwrap_or("").to_string();
    let scope_str = val
        .str_field("scope")
        .ok_or_else(|| LxError::type_err("lock: scope must be Str", span))?;
    let mode_str = val
        .str_field("mode")
        .ok_or_else(|| LxError::type_err("lock: mode must be Str", span))?;
    let holder = val
        .str_field("holder")
        .ok_or_else(|| LxError::type_err("lock: holder must be Str", span))?
        .to_string();
    let scope = match scope_str {
        "file" => LockScope::File(path),
        "folder" => LockScope::Folder(path),
        "repo" => LockScope::Repo,
        other => {
            return Err(LxError::type_err(
                format!("lock: unknown scope `{other}`, expected file/folder/repo"),
                span,
            ));
        }
    };
    let mode = match mode_str {
        "read" => LockMode::Read,
        "write" => LockMode::Write,
        other => {
            return Err(LxError::type_err(
                format!("lock: unknown mode `{other}`, expected read/write"),
                span,
            ));
        }
    };
    Ok(Lock {
        scope,
        mode,
        holder,
    })
}

fn lock_to_record(lk: &Lock) -> Value {
    record! {
        "scope" => Value::Str(Arc::from(super::repo::lock_scope_str(&lk.scope))),
        "path" => Value::Str(Arc::from(super::repo::lock_scope_path(&lk.scope))),
        "mode" => Value::Str(Arc::from(super::repo::lock_mode_str(&lk.mode))),
        "holder" => Value::Str(Arc::from(lk.holder.as_str())),
    }
}

fn conflicts_err(conflicts: &[Lock]) -> Value {
    let list: Vec<Value> = conflicts.iter().map(lock_to_record).collect();
    Value::Err(Box::new(record! {
        "conflicts" => Value::List(Arc::new(list)),
    }))
}

pub fn bi_lock(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let lock = parse_lock_request(&args[1], span)?;
    let mut rp = REPOSPACES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("repo.lock: not found", span))?;
    let conflicts = check_conflicts(&rp.locks, &lock);
    if !conflicts.is_empty() {
        return Ok(conflicts_err(&conflicts));
    }
    acquire_lock(&mut rp, lock);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

pub fn bi_unlock(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let agent = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("repo.unlock: agent must be Str", span))?;
    let mut rp = REPOSPACES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("repo.unlock: not found", span))?;
    release_locks_for_agent(&mut rp, agent);
    Ok(Value::Ok(Box::new(Value::Unit)))
}

pub fn bi_try_lock(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = repo_id(&args[0], span)?;
    let lock = parse_lock_request(&args[1], span)?;
    let mut rp = REPOSPACES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("repo.try_lock: not found", span))?;
    let conflicts = check_conflicts(&rp.locks, &lock);
    if !conflicts.is_empty() {
        return Ok(conflicts_err(&conflicts));
    }
    acquire_lock(&mut rp, lock);
    Ok(Value::Ok(Box::new(Value::Unit)))
}
