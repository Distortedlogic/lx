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
use crate::value::Value;

use super::durable_io;

pub(super) struct Workflow {
    pub name: String,
    pub run_id: String,
    pub dir: PathBuf,
    pub handler: Value,
    pub status: String,
    pub started_at: String,
    pub completed_steps: Vec<String>,
}

pub(super) static WORKFLOWS: LazyLock<DashMap<u64, Workflow>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

const DEFAULT_STORAGE: &str = ".lx/durable";

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("workflow".into(), mk("durable.workflow", 3, bi_workflow));
    m.insert("run".into(), super::durable_run::mk_run());
    m.insert("step".into(), super::durable_run::mk_step());
    m.insert("sleep".into(), super::durable_run::mk_sleep());
    m.insert("signal".into(), super::durable_run::mk_signal());
    m.insert("send_signal".into(), super::durable_run::mk_send_signal());
    m.insert("status".into(), mk("durable.status", 1, bi_status));
    m.insert("list".into(), mk("durable.list", 0, bi_list));
    m
}

pub(super) fn workflow_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__durable_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("durable: expected workflow handle", span)),
        _ => Err(LxError::type_err(
            "durable: expected workflow handle Record",
            span,
        )),
    }
}

fn make_handle(id: u64, name: &str, run_id: &str) -> Value {
    record! {
        "__durable_id" => Value::Int(BigInt::from(id)),
        "name" => Value::Str(Arc::from(name)),
        "run_id" => Value::Str(Arc::from(run_id)),
    }
}

fn bi_workflow(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("durable.workflow: name must be Str", span))?;
    let (storage_base, _retry_policy) = match &args[1] {
        Value::Record(r) => {
            let storage = r
                .get("storage_dir")
                .and_then(|v| v.as_str())
                .unwrap_or(DEFAULT_STORAGE)
                .to_string();
            let retry = r.get("retry_policy").cloned();
            (storage, retry)
        }
        _ => (DEFAULT_STORAGE.to_string(), None),
    };
    let handler = &args[2];
    let run_id = format!("{}", chrono::Utc::now().timestamp_millis());
    let dir = durable_io::storage_dir(&storage_base, name, &run_id);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let now = chrono::Utc::now().to_rfc3339();
    WORKFLOWS.insert(
        id,
        Workflow {
            name: name.to_string(),
            run_id: run_id.clone(),
            dir,
            handler: handler.clone(),
            status: "pending".into(),
            started_at: now,
            completed_steps: Vec::new(),
        },
    );
    Ok(make_handle(id, name, &run_id))
}

fn bi_status(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("durable.status: workflow_id must be Str", span))?;
    for entry in WORKFLOWS.iter() {
        let wf = entry.value();
        if wf.name == name {
            return Ok(record! {
                "workflow_id" => Value::Str(Arc::from(wf.name.as_str())),
                "run_id" => Value::Str(Arc::from(wf.run_id.as_str())),
                "status" => Value::Str(Arc::from(wf.status.as_str())),
                "completed_steps" => Value::Int(BigInt::from(wf.completed_steps.len())),
                "started_at" => Value::Str(Arc::from(wf.started_at.as_str())),
            });
        }
    }
    for entry in WORKFLOWS.iter() {
        let wf = entry.value();
        let dir = &wf.dir;
        if let Some(state) = durable_io::load_state(dir)
            && wf.name == name
        {
            return Ok(record! {
                "workflow_id" => Value::Str(Arc::from(name)),
                "run_id" => Value::Str(Arc::from(wf.run_id.as_str())),
                "status" => Value::Str(Arc::from(state.status.as_str())),
                "completed_steps" => Value::Int(BigInt::from(state.completed_steps.len())),
                "started_at" => Value::Str(Arc::from(state.started_at.as_str())),
            });
        }
    }
    Ok(record! {
        "workflow_id" => Value::Str(Arc::from(name)),
        "status" => Value::Str(Arc::from("not_found")),
    })
}

fn bi_list(_args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items: Vec<Value> = WORKFLOWS
        .iter()
        .map(|entry| {
            let wf = entry.value();
            record! {
                "name" => Value::Str(Arc::from(wf.name.as_str())),
                "run_id" => Value::Str(Arc::from(wf.run_id.as_str())),
                "status" => Value::Str(Arc::from(wf.status.as_str())),
                "completed_steps" => Value::Int(BigInt::from(wf.completed_steps.len())),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(items)))
}
