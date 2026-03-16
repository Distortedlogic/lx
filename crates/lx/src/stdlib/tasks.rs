#[path = "tasks_query.rs"]
mod tasks_query;

use std::path::PathBuf;
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
use crate::stdlib::json_conv;
use crate::value::Value;

pub(super) struct TaskStore {
    pub tasks: IndexMap<String, Value>,
    pub path: Option<PathBuf>,
}

pub(super) static STORES: LazyLock<DashMap<u64, TaskStore>> = LazyLock::new(DashMap::new);
static NEXT_STORE: AtomicU64 = AtomicU64::new(1);
static NEXT_TASK: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("empty".into(), mk("tasks.empty", 1, bi_empty));
    m.insert("load".into(), mk("tasks.load", 1, bi_load));
    m.insert("save".into(), mk("tasks.save", 2, bi_save));
    m.insert("create".into(), mk("tasks.create", 2, bi_create));
    m.insert("get".into(), mk("tasks.get", 2, bi_get));
    m.insert(
        "children".into(),
        mk("tasks.children", 2, tasks_query::bi_children),
    );
    m.insert("list".into(), mk("tasks.list", 2, tasks_query::bi_list));
    m.insert("start".into(), mk("tasks.start", 2, bi_start));
    m.insert(
        "update".into(),
        mk("tasks.update", 3, tasks_query::bi_update),
    );
    m.insert("submit".into(), mk("tasks.submit", 3, bi_submit));
    m.insert("audit".into(), mk("tasks.audit", 2, bi_audit));
    m.insert("pass".into(), mk("tasks.pass", 2, bi_pass));
    m.insert("fail".into(), mk("tasks.fail", 3, bi_fail));
    m.insert("revise".into(), mk("tasks.revise", 2, bi_revise));
    m.insert("complete".into(), mk("tasks.complete", 3, bi_complete));
    m
}

pub(super) fn store_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__store_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("tasks: expected store record", span)),
        _ => Err(LxError::type_err("tasks: expected store Record", span)),
    }
}

fn gen_id() -> String {
    format!("task_{}", NEXT_TASK.fetch_add(1, Ordering::Relaxed))
}

pub(super) fn now() -> Arc<str> {
    Arc::from(chrono::Utc::now().to_rfc3339().as_str())
}

fn store_val(id: u64) -> Value {
    Value::Ok(Box::new(record! {
        "__store_id" => Value::Int(BigInt::from(id)),
    }))
}

pub(super) fn persist(store: &TaskStore, span: Span) -> Result<(), LxError> {
    let Some(ref path) = store.path else {
        return Ok(());
    };
    let items: Vec<Value> = store.tasks.values().cloned().collect();
    let list = Value::List(Arc::new(items));
    let json = json_conv::lx_to_json(&list, span)?;
    let s = serde_json::to_string_pretty(&json)
        .map_err(|e| LxError::runtime(format!("tasks: serialize: {e}"), span))?;
    std::fs::write(path, s)
        .map_err(|e| LxError::runtime(format!("tasks: write {}: {e}", path.display()), span))
}

fn bi_empty(_args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = NEXT_STORE.fetch_add(1, Ordering::Relaxed);
    STORES.insert(
        id,
        TaskStore {
            tasks: IndexMap::new(),
            path: None,
        },
    );
    Ok(store_val(id))
}

fn bi_load(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("tasks.load expects Str path", span))?;
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                format!("tasks.load: {e}").as_str(),
            )))));
        }
    };
    let jv: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| LxError::runtime(format!("tasks.load: JSON: {e}"), span))?;
    let Value::List(items) = json_conv::json_to_lx(jv) else {
        return Err(LxError::runtime("tasks.load: expected JSON array", span));
    };
    let mut tasks = IndexMap::new();
    for item in items.iter() {
        if let Value::Record(r) = item
            && let Some(id) = r.get("id").and_then(|v| v.as_str())
        {
            tasks.insert(id.to_string(), item.clone());
        }
    }
    let sid = NEXT_STORE.fetch_add(1, Ordering::Relaxed);
    STORES.insert(
        sid,
        TaskStore {
            tasks,
            path: Some(PathBuf::from(path)),
        },
    );
    Ok(store_val(sid))
}

fn bi_save(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let path = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("tasks.save expects Str path", span))?;
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("tasks: store not found", span))?;
    store.path = Some(PathBuf::from(path));
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let Value::Record(opts) = &args[1] else {
        return Err(LxError::type_err("tasks.create expects Record", span));
    };
    let title = opts
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("tasks.create: must have 'title' field", span))?;
    let id = gen_id();
    let ts = now();
    let task = record! {
        "id" => Value::Str(Arc::from(id.as_str())),
        "title" => Value::Str(Arc::from(title)),
        "status" => Value::Str(Arc::from("todo")),
        "parent" => opts.get("parent").cloned().unwrap_or(Value::Str(Arc::from(""))),
        "tags" => opts.get("tags").cloned().unwrap_or(Value::List(Arc::new(vec![]))),
        "notes" => Value::Str(Arc::from("")),
        "output" => Value::Str(Arc::from("")),
        "feedback" => Value::Str(Arc::from("")),
        "result" => Value::Str(Arc::from("")),
        "created_at" => Value::Str(ts.clone()),
        "updated_at" => Value::Str(ts),
    };
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("tasks: store not found", span))?;
    store.tasks.insert(id.clone(), task);
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Str(Arc::from(id.as_str())))))
}

fn bi_get(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let tid = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("tasks.get: id must be Str", span))?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("tasks: store not found", span))?;
    match store.tasks.get(tid) {
        Some(t) => Ok(Value::Ok(Box::new(t.clone()))),
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("task '{tid}' not found").as_str(),
        ))))),
    }
}

fn transition(
    store_val: &Value,
    task_id_val: &Value,
    extra: Option<&Value>,
    from: &[&str],
    to: &str,
    span: Span,
) -> Result<Value, LxError> {
    let sid = store_id(store_val, span)?;
    let tid = task_id_val
        .as_str()
        .ok_or_else(|| LxError::type_err("tasks: id must be Str", span))?;
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("tasks: store not found", span))?;
    let task = store
        .tasks
        .get(tid)
        .ok_or_else(|| LxError::runtime(format!("tasks: task '{tid}' not found"), span))?
        .clone();
    let Value::Record(r) = task else {
        return Err(LxError::runtime("tasks: corrupt task record", span));
    };
    let status = r.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if !from.contains(&status) {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("tasks: cannot transition '{status}' -> '{to}'").as_str(),
        )))));
    }
    let mut fields = (*r).clone();
    fields.insert("status".into(), Value::Str(Arc::from(to)));
    fields.insert("updated_at".into(), Value::Str(now()));
    if let Some(Value::Record(ef)) = extra {
        for (k, v) in ef.iter() {
            if k != "id" && k != "status" && k != "created_at" {
                fields.insert(k.clone(), v.clone());
            }
        }
    }
    store
        .tasks
        .insert(tid.to_string(), Value::Record(Arc::new(fields)));
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_start(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    transition(&args[0], &args[1], None, &["todo"], "in_progress", span)
}

fn bi_submit(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    transition(
        &args[0],
        &args[1],
        Some(&args[2]),
        &["in_progress", "revision"],
        "submitted",
        span,
    )
}

fn bi_audit(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    transition(
        &args[0],
        &args[1],
        None,
        &["submitted"],
        "pending_audit",
        span,
    )
}

fn bi_pass(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    transition(&args[0], &args[1], None, &["pending_audit"], "passed", span)
}

fn bi_fail(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    transition(
        &args[0],
        &args[1],
        Some(&args[2]),
        &["pending_audit"],
        "failed",
        span,
    )
}

fn bi_revise(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    transition(&args[0], &args[1], None, &["failed"], "revision", span)
}

fn bi_complete(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    transition(
        &args[0],
        &args[1],
        Some(&args[2]),
        &["passed"],
        "complete",
        span,
    )
}
