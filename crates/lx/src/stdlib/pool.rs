use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

static NEXT_POOL_ID: LazyLock<std::sync::atomic::AtomicU64> =
    LazyLock::new(|| std::sync::atomic::AtomicU64::new(1));

struct PoolState {
    workers: Vec<Value>,
    next_worker: usize,
    completed: u64,
    failed: u64,
}

static POOLS: LazyLock<DashMap<u64, PoolState>> = LazyLock::new(DashMap::new);

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("pool.create", 1, bi_create));
    m.insert("fan_out".into(), mk("pool.fan_out", 2, bi_fan_out));
    m.insert("map".into(), mk("pool.map", 3, bi_map));
    m.insert("submit".into(), mk("pool.submit", 2, bi_submit));
    m.insert("status".into(), mk("pool.status", 1, bi_status));
    m.insert("shutdown".into(), mk("pool.shutdown", 1, bi_shutdown));
    m
}

fn get_pool_id(pool: &Value, span: Span) -> Result<u64, LxError> {
    match pool {
        Value::Record(r) => r
            .get("__pool_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("expected pool record", span)),
        _ => Err(LxError::type_err("expected pool record", span)),
    }
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(config) = &args[0] else {
        return Err(LxError::type_err("pool.create expects Record config", span));
    };
    let size = config
        .get("size")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
        .unwrap_or(1usize);
    let handler = config.get("handler").cloned();
    let trait_constraint = config.get("trait").cloned();
    let name = config
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("pool")
        .to_string();
    let mut workers = Vec::with_capacity(size);
    for i in 0..size {
        let worker = if let Some(ref h) = handler {
            let mut rec = IndexMap::new();
            rec.insert("name".into(), Value::Str(Arc::from(format!("{name}-{i}"))));
            rec.insert("handler".into(), h.clone());
            if let Some(Value::Trait { name: tn, .. }) = trait_constraint.as_ref() {
                rec.insert(
                    "__traits".into(),
                    Value::List(Arc::new(vec![Value::Str(Arc::clone(tn))])),
                );
            }
            Value::Record(Arc::new(rec))
        } else {
            record! {
                "name" => Value::Str(Arc::from(format!("{name}-{i}"))),
                "handler" => Value::Unit,
            }
        };
        workers.push(worker);
    }
    let pool_id = NEXT_POOL_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    POOLS.insert(
        pool_id,
        PoolState {
            workers,
            next_worker: 0,
            completed: 0,
            failed: 0,
        },
    );
    Ok(Value::Ok(Box::new(record! {
        "__pool_id" => Value::Int(BigInt::from(pool_id)),
        "name" => Value::Str(Arc::from(name)),
        "size" => Value::Int(BigInt::from(size)),
    })))
}

fn dispatch_to_worker(
    worker: &Value,
    task: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    if let Value::Record(r) = worker
        && let Some(handler) = r.get("handler")
        && *handler != Value::Unit
    {
        return call_value(handler, task.clone(), span, ctx);
    }
    Err(LxError::runtime("pool: worker has no handler", span))
}

fn bi_fan_out(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pool_id = get_pool_id(&args[0], span)?;
    let Value::List(tasks) = &args[1] else {
        return Err(LxError::type_err(
            "pool.fan_out: tasks must be a List",
            span,
        ));
    };
    let mut results = Vec::with_capacity(tasks.len());
    for task in tasks.iter() {
        let worker = {
            let mut pool = POOLS
                .get_mut(&pool_id)
                .ok_or_else(|| LxError::runtime("pool not found", span))?;
            let idx = pool.next_worker % pool.workers.len();
            pool.next_worker = idx + 1;
            pool.workers[idx].clone()
        };
        match dispatch_to_worker(&worker, task, span, ctx) {
            Ok(v) => {
                if let Some(mut pool) = POOLS.get_mut(&pool_id) {
                    pool.completed += 1;
                }
                results.push(Value::Ok(Box::new(v)));
            }
            Err(e) => {
                if let Some(mut pool) = POOLS.get_mut(&pool_id) {
                    pool.failed += 1;
                }
                results.push(Value::Err(Box::new(Value::Str(Arc::from(format!("{e}"))))));
            }
        }
    }
    Ok(Value::Ok(Box::new(Value::List(Arc::new(results)))))
}

fn bi_map(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pool_id = get_pool_id(&args[0], span)?;
    let Value::List(items) = &args[1] else {
        return Err(LxError::type_err("pool.map: items must be a List", span));
    };
    let mapper = &args[2];
    let mut results = Vec::with_capacity(items.len());
    for item in items.iter() {
        let task = call_value(mapper, item.clone(), span, ctx)?;
        let worker = {
            let mut pool = POOLS
                .get_mut(&pool_id)
                .ok_or_else(|| LxError::runtime("pool not found", span))?;
            let idx = pool.next_worker % pool.workers.len();
            pool.next_worker = idx + 1;
            pool.workers[idx].clone()
        };
        match dispatch_to_worker(&worker, &task, span, ctx) {
            Ok(v) => {
                if let Some(mut pool) = POOLS.get_mut(&pool_id) {
                    pool.completed += 1;
                }
                results.push(Value::Ok(Box::new(v)));
            }
            Err(e) => {
                if let Some(mut pool) = POOLS.get_mut(&pool_id) {
                    pool.failed += 1;
                }
                results.push(Value::Err(Box::new(Value::Str(Arc::from(format!("{e}"))))));
            }
        }
    }
    Ok(Value::Ok(Box::new(Value::List(Arc::new(results)))))
}

fn bi_submit(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pool_id = get_pool_id(&args[0], span)?;
    let task = &args[1];
    let worker = {
        let mut pool = POOLS
            .get_mut(&pool_id)
            .ok_or_else(|| LxError::runtime("pool not found", span))?;
        let idx = pool.next_worker % pool.workers.len();
        pool.next_worker = idx + 1;
        pool.workers[idx].clone()
    };
    match dispatch_to_worker(&worker, task, span, ctx) {
        Ok(_) => {
            if let Some(mut pool) = POOLS.get_mut(&pool_id) {
                pool.completed += 1;
            }
        }
        Err(_) => {
            if let Some(mut pool) = POOLS.get_mut(&pool_id) {
                pool.failed += 1;
            }
        }
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_status(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pool_id = get_pool_id(&args[0], span)?;
    let pool = POOLS
        .get(&pool_id)
        .ok_or_else(|| LxError::runtime("pool not found", span))?;
    Ok(record! {
        "size" => Value::Int(BigInt::from(pool.workers.len())),
        "completed" => Value::Int(BigInt::from(pool.completed)),
        "failed" => Value::Int(BigInt::from(pool.failed)),
    })
}

fn bi_shutdown(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pool_id = get_pool_id(&args[0], span)?;
    POOLS
        .remove(&pool_id)
        .ok_or_else(|| LxError::runtime("pool not found", span))?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}
