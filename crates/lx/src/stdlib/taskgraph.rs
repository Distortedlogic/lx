#[path = "taskgraph_run.rs"]
mod taskgraph_run;
#[path = "taskgraph_topo.rs"]
mod taskgraph_topo;

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

pub(super) struct GraphState {
    pub name: String,
    pub nodes: IndexMap<String, TaskNode>,
}

pub(super) struct TaskNode {
    pub opts: IndexMap<String, Value>,
    pub depends: Vec<String>,
}

static GRAPHS: LazyLock<DashMap<u64, GraphState>> = LazyLock::new(DashMap::new);
static NEXT_GRAPH: AtomicU64 = AtomicU64::new(1);

pub(super) fn get_graph(
    gid: u64,
    span: Span,
) -> Result<dashmap::mapref::one::Ref<'static, u64, GraphState>, LxError> {
    GRAPHS
        .get(&gid)
        .ok_or_else(|| LxError::runtime("taskgraph: graph not found", span))
}

pub(super) fn graph_id(v: &Value, span: Span) -> Result<u64, LxError> {
    match v {
        Value::Record(r) => r
            .get("__graph_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("taskgraph: expected graph record", span)),
        _ => Err(LxError::type_err("taskgraph: expected graph Record", span)),
    }
}

fn graph_val(id: u64, name: &str) -> Value {
    Value::Ok(Box::new(record! {
        "__graph_id" => Value::Int(BigInt::from(id)),
        "name" => Value::Str(Arc::from(name)),
    }))
}

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("create".into(), mk("taskgraph.create", 1, bi_create));
    m.insert("add".into(), mk("taskgraph.add", 3, bi_add));
    m.insert("remove".into(), mk("taskgraph.remove", 2, bi_remove));
    m.insert("run".into(), mk("taskgraph.run", 1, taskgraph_run::bi_run));
    m.insert(
        "run_with".into(),
        mk("taskgraph.run_with", 2, taskgraph_run::bi_run_with),
    );
    m.insert("validate".into(), mk("taskgraph.validate", 1, bi_validate));
    m.insert("topo".into(), mk("taskgraph.topo", 1, bi_topo));
    m.insert("status".into(), mk("taskgraph.status", 1, bi_status));
    m.insert("dot".into(), mk("taskgraph.dot", 1, bi_dot));
    m
}

fn bi_create(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("taskgraph.create: name must be Str", span))?;
    let id = NEXT_GRAPH.fetch_add(1, Ordering::Relaxed);
    GRAPHS.insert(
        id,
        GraphState {
            name: name.to_string(),
            nodes: IndexMap::new(),
        },
    );
    Ok(graph_val(id, name))
}

fn bi_add(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let gid = graph_id(&args[0], span)?;
    let task_id = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("taskgraph.add: id must be Str", span))?;
    let Value::Record(opts) = &args[2] else {
        return Err(LxError::type_err(
            "taskgraph.add: opts must be Record",
            span,
        ));
    };

    let depends = extract_depends(opts, span)?;

    let mut graph = GRAPHS
        .get_mut(&gid)
        .ok_or_else(|| LxError::runtime("taskgraph: graph not found", span))?;

    if graph.nodes.contains_key(task_id) {
        return Err(LxError::runtime(
            format!("taskgraph.add: task '{task_id}' already exists"),
            span,
        ));
    }

    graph.nodes.insert(
        task_id.to_string(),
        TaskNode {
            opts: opts.as_ref().clone(),
            depends,
        },
    );
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn extract_depends(opts: &IndexMap<String, Value>, span: Span) -> Result<Vec<String>, LxError> {
    let Some(deps_val) = opts.get("depends") else {
        return Ok(Vec::new());
    };
    let Value::List(deps) = deps_val else {
        return Err(LxError::type_err(
            "taskgraph.add: depends must be a List of Str",
            span,
        ));
    };
    let mut out = Vec::with_capacity(deps.len());
    for d in deps.iter() {
        let s = d
            .as_str()
            .ok_or_else(|| LxError::type_err("taskgraph.add: depends entry must be Str", span))?;
        out.push(s.to_string());
    }
    Ok(out)
}

fn bi_remove(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let gid = graph_id(&args[0], span)?;
    let task_id = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("taskgraph.remove: id must be Str", span))?;
    let mut graph = GRAPHS
        .get_mut(&gid)
        .ok_or_else(|| LxError::runtime("taskgraph: graph not found", span))?;

    if graph.nodes.shift_remove(task_id).is_none() {
        return Err(LxError::runtime(
            format!("taskgraph.remove: task '{task_id}' not found"),
            span,
        ));
    }

    for node in graph.nodes.values_mut() {
        node.depends.retain(|d| d != task_id);
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_validate(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let gid = graph_id(&args[0], span)?;
    let graph = get_graph(gid, span)?;

    for (id, node) in &graph.nodes {
        for dep in &node.depends {
            if !graph.nodes.contains_key(dep) {
                return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                    format!("task '{id}' depends on unknown task '{dep}'").as_str(),
                )))));
            }
        }
    }

    match taskgraph_topo::topo_sort(&graph.nodes, span) {
        Ok(_) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(_) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "cycle detected in task graph",
        ))))),
    }
}

fn bi_topo(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let gid = graph_id(&args[0], span)?;
    let graph = get_graph(gid, span)?;
    let order = taskgraph_topo::topo_sort(&graph.nodes, span)?;
    let list: Vec<Value> = order
        .into_iter()
        .map(|s| Value::Str(Arc::from(s.as_str())))
        .collect();
    Ok(Value::Ok(Box::new(Value::List(Arc::new(list)))))
}

fn bi_status(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let gid = graph_id(&args[0], span)?;
    let graph = get_graph(gid, span)?;
    let mut rec = IndexMap::new();
    for id in graph.nodes.keys() {
        rec.insert(id.clone(), Value::Str(Arc::from("pending")));
    }
    Ok(Value::Record(Arc::new(rec)))
}

fn bi_dot(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let gid = graph_id(&args[0], span)?;
    let graph = get_graph(gid, span)?;
    let mut out = format!("digraph \"{}\" {{\n", graph.name);
    for (id, node) in &graph.nodes {
        out.push_str(&format!("  \"{id}\";\n"));
        for dep in &node.depends {
            out.push_str(&format!("  \"{dep}\" -> \"{id}\";\n"));
        }
    }
    out.push_str("}\n");
    Ok(Value::Str(Arc::from(out.as_str())))
}
