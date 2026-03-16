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

pub(crate) struct TraceStore {
    pub(crate) spans: Vec<TraceSpan>,
    pub(crate) path: PathBuf,
}

pub(crate) struct TraceSpan {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) input: String,
    pub(crate) output: String,
    pub(crate) score: Option<f64>,
    pub(crate) duration_ms: Option<i64>,
    pub(crate) tags: Vec<String>,
    pub(crate) created_at: String,
}

pub(crate) static STORES: LazyLock<DashMap<u64, TraceStore>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
pub(crate) static NEXT_SPAN: AtomicU64 = AtomicU64::new(1);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("trace.create", 1, bi_create));
    m.insert("record".into(), mk("trace.record", 2, bi_record));
    m.insert("score".into(), mk("trace.score", 3, bi_score));
    m.insert("spans".into(), mk("trace.spans", 1, bi_spans));
    super::trace_query::register(&mut m);
    super::trace_progress::register(&mut m);
    m
}

pub(crate) fn store_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__trace_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("trace: expected trace store record", span)),
        _ => Err(LxError::type_err("trace: expected Record", span)),
    }
}

fn make_handle(id: u64) -> Value {
    Value::Ok(Box::new(record! {
        "__trace_id" => Value::Int(BigInt::from(id)),
    }))
}

pub(crate) fn now_str() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub(crate) fn span_to_value(s: &TraceSpan) -> Value {
    let mut f = IndexMap::new();
    f.insert("id".into(), Value::Int(BigInt::from(s.id)));
    f.insert("name".into(), Value::Str(Arc::from(s.name.as_str())));
    f.insert("input".into(), Value::Str(Arc::from(s.input.as_str())));
    f.insert("output".into(), Value::Str(Arc::from(s.output.as_str())));
    match s.score {
        Some(sc) => f.insert("score".into(), Value::Float(sc)),
        None => f.insert("score".into(), Value::None),
    };
    match s.duration_ms {
        Some(ms) => f.insert("duration_ms".into(), Value::Int(BigInt::from(ms))),
        None => f.insert("duration_ms".into(), Value::None),
    };
    let tags: Vec<Value> = s
        .tags
        .iter()
        .map(|t| Value::Str(Arc::from(t.as_str())))
        .collect();
    f.insert("tags".into(), Value::List(Arc::new(tags)));
    f.insert(
        "created_at".into(),
        Value::Str(Arc::from(s.created_at.as_str())),
    );
    Value::Record(Arc::new(f))
}

pub(crate) fn persist(store: &TraceStore, span: Span) -> Result<(), LxError> {
    let items: Vec<Value> = store.spans.iter().map(span_to_value).collect();
    let list = Value::List(Arc::new(items));
    let json = json_conv::lx_to_json(&list, span)?;
    let s = serde_json::to_string_pretty(&json)
        .map_err(|e| LxError::runtime(format!("trace: serialize: {e}"), span))?;
    std::fs::write(&store.path, s).map_err(|e| LxError::runtime(format!("trace: write: {e}"), span))
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("trace.create expects Str path", span))?;
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    STORES.insert(
        id,
        TraceStore {
            spans: Vec::new(),
            path: PathBuf::from(path),
        },
    );
    Ok(make_handle(id))
}

fn bi_record(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err(
            "trace.record: first arg must be Record",
            span,
        ));
    };
    let sid = store_id(&args[1], span)?;
    let name = fields
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let input = fields
        .get("input")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let output = fields
        .get("output")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let score = fields.get("score").and_then(|v| v.as_float());
    let duration_ms = fields
        .get("duration_ms")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok());
    let tags = fields
        .get("tags")
        .and_then(|v| v.as_list())
        .map(|l| {
            l.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let span_id = NEXT_SPAN.fetch_add(1, Ordering::Relaxed);
    let ts = TraceSpan {
        id: span_id,
        name,
        input,
        output,
        score,
        duration_ms,
        tags,
        created_at: now_str(),
    };
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("trace: store not found", span))?;
    store.spans.push(ts);
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Int(BigInt::from(span_id)))))
}

fn bi_score(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let span_id: u64 = args[0]
        .as_int()
        .and_then(|n| n.try_into().ok())
        .ok_or_else(|| LxError::type_err("trace.score: span_id must be Int", span))?;
    let sc = args[1]
        .as_float()
        .ok_or_else(|| LxError::type_err("trace.score: score must be Float", span))?;
    let sid = store_id(&args[2], span)?;
    let mut store = STORES
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("trace: store not found", span))?;
    let entry = store
        .spans
        .iter_mut()
        .find(|s| s.id == span_id)
        .ok_or_else(|| LxError::runtime(format!("trace: span {span_id} not found"), span))?;
    entry.score = Some(sc);
    persist(&store, span)?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_spans(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = store_id(&args[0], span)?;
    let store = STORES
        .get(&sid)
        .ok_or_else(|| LxError::runtime("trace: store not found", span))?;
    let results: Vec<Value> = store.spans.iter().map(span_to_value).collect();
    Ok(Value::List(Arc::new(results)))
}
