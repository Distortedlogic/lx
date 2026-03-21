use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub(crate) fn bi_run(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let gid = super::graph_id(&args[0], span)?;
    let graph = super::get_graph(gid, span)?;
    let waves = super::taskgraph_topo::topo_waves(&graph.nodes, span)?;

    let mut results: IndexMap<String, Value> = IndexMap::new();
    for wave in &waves {
        for task_id in wave {
            let node = &graph.nodes[task_id];
            let input = resolve_input(&node.opts, &results, span, ctx)?;
            let handler = node.opts.get("handler");
            let result = execute_task(task_id, handler, &input, &node.opts, span, ctx)?;
            results.insert(task_id.clone(), result);
        }
    }
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(results)))))
}

pub(crate) fn bi_run_with(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let gid = super::graph_id(&args[0], span)?;
    let Value::Record(run_opts) = &args[1] else {
        return Err(LxError::type_err(
            "taskgraph.run_with: opts must be Record",
            span,
        ));
    };
    let on_complete = run_opts.get("on_complete");
    let on_fail = run_opts.get("on_fail");
    let max_parallel = run_opts
        .get("max_parallel")
        .and_then(|v| v.as_int())
        .and_then(|n| usize::try_from(n).ok());

    let graph = super::get_graph(gid, span)?;
    let waves = super::taskgraph_topo::topo_waves(&graph.nodes, span)?;

    let mut results: IndexMap<String, Value> = IndexMap::new();
    for wave in &waves {
        let effective_wave: Vec<&String> = if let Some(max) = max_parallel {
            wave.iter().take(max).collect()
        } else {
            wave.iter().collect()
        };

        for task_id in &effective_wave {
            let node = &graph.nodes[task_id.as_str()];
            let input = resolve_input(&node.opts, &results, span, ctx)?;
            let handler = node.opts.get("handler");
            match execute_task(task_id, handler, &input, &node.opts, span, ctx) {
                Ok(result) => {
                    if let Some(cb) = on_complete {
                        let _ = call_value_sync(
                            cb,
                            Value::Tuple(Arc::new(vec![
                                Value::Str(Arc::from(task_id.as_str())),
                                result.clone(),
                            ])),
                            span,
                            ctx,
                        );
                    }
                    results.insert((*task_id).clone(), result);
                }
                Err(e) => {
                    let err_str = format!("{e}");
                    if let Some(cb) = on_fail {
                        let _ = call_value_sync(
                            cb,
                            Value::Tuple(Arc::new(vec![
                                Value::Str(Arc::from(task_id.as_str())),
                                Value::Str(Arc::from(err_str.as_str())),
                            ])),
                            span,
                            ctx,
                        );
                    }
                    let on_fail_policy = node
                        .opts
                        .get("on_fail")
                        .and_then(|v| v.as_str())
                        .unwrap_or("fail");
                    match on_fail_policy {
                        "skip" => {
                            results.insert(
                                (*task_id).clone(),
                                Value::Err(Box::new(Value::Str(Arc::from(err_str.as_str())))),
                            );
                        }
                        _ => {
                            return Err(LxError::runtime(
                                format!("taskgraph: task '{task_id}' failed: {err_str}"),
                                span,
                            ));
                        }
                    }
                }
            }
        }

        if let Some(max) = max_parallel {
            let remaining: Vec<&String> = wave.iter().skip(max).collect();
            for task_id in remaining {
                let node = &graph.nodes[task_id.as_str()];
                let input = resolve_input(&node.opts, &results, span, ctx)?;
                let handler = node.opts.get("handler");
                let result = execute_task(task_id, handler, &input, &node.opts, span, ctx)?;
                if let Some(cb) = on_complete {
                    let _ = call_value_sync(
                        cb,
                        Value::Tuple(Arc::new(vec![
                            Value::Str(Arc::from(task_id.as_str())),
                            result.clone(),
                        ])),
                        span,
                        ctx,
                    );
                }
                results.insert(task_id.clone(), result);
            }
        }
    }
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(results)))))
}

fn resolve_input(
    opts: &IndexMap<String, Value>,
    results: &IndexMap<String, Value>,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    if let Some(input_from) = opts.get("input_from") {
        let results_rec = Value::Record(Arc::new(results.clone()));
        return call_value_sync(input_from, results_rec, span, ctx);
    }
    if let Some(input) = opts.get("input") {
        return Ok(input.clone());
    }
    let mut dep_results = IndexMap::new();
    if let Some(Value::List(deps)) = opts.get("depends") {
        for dep in deps.iter() {
            if let Some(dep_id) = dep.as_str()
                && let Some(result) = results.get(dep_id)
            {
                dep_results.insert(dep_id.to_string(), result.clone());
            }
        }
    }
    if dep_results.is_empty() {
        Ok(Value::Record(Arc::new(IndexMap::new())))
    } else {
        Ok(Value::Record(Arc::new(dep_results)))
    }
}

fn execute_task(
    task_id: &str,
    handler: Option<&Value>,
    input: &Value,
    opts: &IndexMap<String, Value>,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let retry_count = opts
        .get("retry")
        .and_then(|v| v.as_int())
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(0);

    let timeout_ms = opts
        .get("timeout")
        .and_then(|v| v.as_int())
        .and_then(|n| u64::try_from(n).ok());

    let Some(handler) = handler else {
        return Ok(input.clone());
    };

    let _guard = timeout_ms.map(crate::stdlib::deadline::scoped);
    let retry_opts = crate::stdlib::retry_helpers::RetryOpts::exponential();
    let mut last_err = None;
    for attempt in 0..=retry_count {
        match call_value_sync(handler, input.clone(), span, ctx) {
            Ok(result) => {
                if let Some(ref g) = _guard
                    && g.is_expired()
                {
                    return Err(LxError::runtime(
                        format!("taskgraph: task '{task_id}' exceeded timeout"),
                        span,
                    ));
                }
                return Ok(result);
            }
            Err(e) => {
                last_err = Some(e);
                if attempt < retry_count {
                    let delay = crate::stdlib::retry_helpers::compute_delay(&retry_opts, attempt as u64);
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| {
        LxError::runtime(
            format!("taskgraph: task '{task_id}' failed after retries"),
            span,
        )
    }))
}
