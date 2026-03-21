use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub(crate) fn bi_run(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let flow = &args[0];
    let input = &args[1];
    run_flow(flow, input, span, ctx)
}

pub(super) fn run_flow(
    flow: &Value,
    input: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    match flow {
        Value::Record(r) => match r.get("__flow").and_then(|v| v.as_str()) {
            Some("single") => run_single(r, input, span, ctx),
            Some("pipe") => run_pipe(r, input, span, ctx),
            Some("par") => run_par(r, input, span, ctx),
            Some("branch") => run_branch(r, input, span, ctx),
            Some("retry") => run_retry(r, input, span, ctx),
            Some("timeout") => run_timeout(r, input, span, ctx),
            Some("fallback") => run_fallback(r, input, span, ctx),
            _ => Err(LxError::type_err(
                format!(
                    "flow.run: expected Flow record, got {} `{}`",
                    flow.type_name(),
                    flow.short_display()
                ),
                span,
            )),
        },
        _ => Err(LxError::type_err(
            format!(
                "flow.run: expected Flow, got {} `{}`",
                flow.type_name(),
                flow.short_display()
            ),
            span,
        )),
    }
}

fn run_single(
    r: &IndexMap<String, Value>,
    input: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let source = r
        .get("source")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("flow.run: flow missing 'source'", span))?;
    let path = r
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LxError::runtime("flow.run: flow missing 'path'", span))?;

    let tokens = crate::lexer::lex(source)
        .map_err(|e| LxError::runtime(format!("flow.run: lex error in '{path}': {e}"), span))?;
    let program = crate::parser::parse(tokens)
        .map_err(|e| LxError::runtime(format!("flow.run: parse error in '{path}': {e}"), span))?;
    let module_dir = std::path::Path::new(path).parent().map(|p| p.to_path_buf());
    let mut interp = crate::interpreter::Interpreter::new(source, module_dir, Arc::clone(ctx));
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            interp.exec(&program).await.map_err(|e| {
                LxError::runtime(format!("flow.run: exec error in '{path}': {e}"), span)
            })?;

            let entry_name = super::find_entry(&program).ok_or_else(|| {
                LxError::runtime(
                    format!("flow.run: flow '{path}' must export +run or +main"),
                    span,
                )
            })?;
            let entry = interp.env.get(&entry_name).ok_or_else(|| {
                LxError::runtime(
                    format!("flow.run: +{entry_name} not found in '{path}'"),
                    span,
                )
            })?;
            let result = interp.apply_func(entry, input.clone(), span).await?;
            Ok(Value::Ok(Box::new(result)))
        })
    })
}

fn unwrap_ok(v: Value) -> Value {
    if let Value::Ok(inner) = v { *inner } else { v }
}

fn run_pipe(
    r: &IndexMap<String, Value>,
    input: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let flows = match r.get("flows") {
        Some(Value::List(l)) => l.as_ref(),
        _ => {
            return Err(LxError::runtime(
                "flow.run: pipe missing 'flows' list",
                span,
            ));
        }
    };
    let mut current = input.clone();
    for flow in flows {
        let result = run_flow(flow, &current, span, ctx)?;
        current = unwrap_ok(result);
    }
    Ok(Value::Ok(Box::new(current)))
}

fn run_par(
    r: &IndexMap<String, Value>,
    input: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let flows = match r.get("flows") {
        Some(Value::List(l)) => l.as_ref(),
        _ => return Err(LxError::runtime("flow.run: par missing 'flows' list", span)),
    };
    let mut results = Vec::new();
    for flow in flows {
        let result = run_flow(flow, input, span, ctx)?;
        results.push(unwrap_ok(result));
    }
    Ok(Value::Ok(Box::new(Value::List(Arc::new(results)))))
}

fn run_branch(
    r: &IndexMap<String, Value>,
    input: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let router = r
        .get("router")
        .ok_or_else(|| LxError::runtime("flow.run: branch missing 'router' function", span))?;
    let chosen = call_value_sync(router, input.clone(), span, ctx)?;
    run_flow(&chosen, input, span, ctx)
}

fn run_retry(
    r: &IndexMap<String, Value>,
    input: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let inner = r
        .get("inner")
        .ok_or_else(|| LxError::runtime("flow.run: retry missing 'inner' flow", span))?;
    let max = r
        .get("opts")
        .and_then(|v| match v {
            Value::Record(opts) => opts
                .get("max")
                .and_then(|v| v.as_int())
                .and_then(|n| u64::try_from(n).ok()),
            _ => None,
        })
        .unwrap_or(3);

    let mut last_err = None;
    for attempt in 0..max {
        match run_flow(inner, input, span, ctx) {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_err = Some(e);
                if attempt + 1 < max {
                    let delay = 100u64.saturating_mul(1u64 << attempt.min(32)).min(30_000);
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| LxError::runtime("flow.run: retry exhausted", span)))
}

fn run_timeout(
    r: &IndexMap<String, Value>,
    input: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let inner = r
        .get("inner")
        .ok_or_else(|| LxError::runtime("flow.run: timeout missing 'inner' flow", span))?;
    let seconds = r
        .get("seconds")
        .and_then(|v| {
            v.as_float()
                .map(|f| f as i64)
                .or_else(|| v.as_int().and_then(|n| i64::try_from(n).ok()))
        })
        .unwrap_or(300);

    let guard = crate::stdlib::deadline::scoped(seconds as u64 * 1000);
    let result = run_flow(inner, input, span, ctx)?;
    if guard.is_expired() {
        return Err(LxError::runtime(
            format!("flow.run: exceeded {seconds}s timeout"),
            span,
        ));
    }
    Ok(result)
}

fn run_fallback(
    r: &IndexMap<String, Value>,
    input: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let inner = r
        .get("inner")
        .ok_or_else(|| LxError::runtime("flow.run: fallback missing 'inner' flow", span))?;
    let fallback = r
        .get("fallback")
        .ok_or_else(|| LxError::runtime("flow.run: fallback missing 'fallback' flow", span))?;
    match run_flow(inner, input, span, ctx) {
        Ok(result) => Ok(result),
        Err(_) => run_flow(fallback, input, span, ctx),
    }
}
