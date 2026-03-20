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

pub(super) struct Region {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

pub(super) struct Conflict {
    pub id: u64,
    pub region: String,
    pub old_content: String,
    pub new_content: String,
}

pub(super) struct EditEntry {
    pub region: String,
    pub at: String,
}

pub(super) struct Workspace {
    pub content: String,
    pub regions: IndexMap<String, Region>,
    pub conflicts: Vec<Conflict>,
    pub history: Vec<EditEntry>,
    pub watchers: Vec<Value>,
}

pub(super) static WORKSPACES: LazyLock<DashMap<u64, Workspace>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("workspace.create", 2, bi_create));
    m.insert(
        "claim".into(),
        mk("workspace.claim", 3, super::workspace_edit::bi_claim),
    );
    m.insert(
        "claim_pattern".into(),
        mk(
            "workspace.claim_pattern",
            3,
            super::workspace_edit::bi_claim_pattern,
        ),
    );
    m.insert(
        "edit".into(),
        mk("workspace.edit", 3, super::workspace_edit::bi_edit),
    );
    m.insert(
        "append".into(),
        mk("workspace.append", 3, super::workspace_edit::bi_append),
    );
    m.insert(
        "release".into(),
        mk("workspace.release", 2, super::workspace_edit::bi_release),
    );
    m.insert("snapshot".into(), mk("workspace.snapshot", 1, bi_snapshot));
    m.insert("regions".into(), mk("workspace.regions", 1, bi_regions));
    m.insert(
        "conflicts".into(),
        mk("workspace.conflicts", 1, bi_conflicts),
    );
    m.insert("resolve".into(), mk("workspace.resolve", 3, bi_resolve));
    m.insert("history".into(), mk("workspace.history", 1, bi_history));
    m.insert("watch".into(), mk("workspace.watch", 2, bi_watch));
    m
}

pub(super) fn ws_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__workspace_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("workspace: expected Workspace handle", span)),
        _ => Err(LxError::type_err(
            "workspace: expected Workspace Record",
            span,
        )),
    }
}

fn make_ws_handle(id: u64, name: &str) -> Value {
    record! {
        "__workspace_id" => Value::Int(BigInt::from(id)),
        "name" => Value::Str(Arc::from(name)),
    }
}

pub(super) fn now_str() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.create: name must be Str", span))?;
    let content = match &args[1] {
        Value::Record(r) => r
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    };
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    WORKSPACES.insert(
        id,
        Workspace {
            content,
            regions: IndexMap::new(),
            conflicts: Vec::new(),
            history: Vec::new(),
            watchers: Vec::new(),
        },
    );
    Ok(Value::Ok(Box::new(make_ws_handle(id, name))))
}

fn bi_snapshot(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let ws = WORKSPACES
        .get(&id)
        .ok_or_else(|| LxError::runtime("workspace.snapshot: not found", span))?;
    Ok(Value::Ok(Box::new(Value::Str(Arc::from(
        ws.content.as_str(),
    )))))
}

fn bi_regions(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let ws = WORKSPACES
        .get(&id)
        .ok_or_else(|| LxError::runtime("workspace.regions: not found", span))?;
    let list: Vec<Value> = ws
        .regions
        .values()
        .map(|r| {
            record! {
                "name" => Value::Str(Arc::from(r.name.as_str())),
                "start" => Value::Int(BigInt::from(r.start)),
                "end" => Value::Int(BigInt::from(r.end)),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(list)))
}

fn bi_conflicts(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let ws = WORKSPACES
        .get(&id)
        .ok_or_else(|| LxError::runtime("workspace.conflicts: not found", span))?;
    let list: Vec<Value> = ws
        .conflicts
        .iter()
        .map(|c| {
            record! {
                "id" => Value::Str(Arc::from(format!("{}", c.id).as_str())),
                "region" => Value::Str(Arc::from(c.region.as_str())),
                "old" => Value::Str(Arc::from(c.old_content.as_str())),
                "new" => Value::Str(Arc::from(c.new_content.as_str())),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(list)))
}

fn bi_resolve(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let conflict_id_str = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.resolve: id must be Str", span))?;
    let _resolution = args[2]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.resolve: resolution must be Str", span))?;
    let cid: u64 = conflict_id_str
        .parse()
        .map_err(|_| LxError::runtime("workspace.resolve: invalid conflict id", span))?;
    let mut ws = WORKSPACES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("workspace.resolve: not found", span))?;
    let idx = ws.conflicts.iter().position(|c| c.id == cid);
    match idx {
        Some(i) => {
            let conflict = ws.conflicts.remove(i);
            ws.history.push(EditEntry {
                region: conflict.region,
                at: now_str(),
            });
            Ok(Value::Ok(Box::new(Value::Unit)))
        }
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "conflict not found",
        ))))),
    }
}

fn bi_history(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let ws = WORKSPACES
        .get(&id)
        .ok_or_else(|| LxError::runtime("workspace.history: not found", span))?;
    let list: Vec<Value> = ws
        .history
        .iter()
        .map(|e| {
            record! {
                "region" => Value::Str(Arc::from(e.region.as_str())),
                "at" => Value::Str(Arc::from(e.at.as_str())),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(list)))
}

fn bi_watch(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let handler = args[1].clone();
    let mut ws = WORKSPACES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("workspace.watch: not found", span))?;
    ws.watchers.push(handler);
    Ok(Value::Unit)
}
