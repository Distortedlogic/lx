use std::sync::Arc;
use std::time::Instant;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("retry".into(), mk("retry.retry", 1, bi_retry));
    m.insert(
        "retry_with".into(),
        mk("retry.retry_with", 2, bi_retry_with),
    );
    m
}

struct RetryOpts {
    max: u64,
    backoff: Backoff,
    base_ms: u64,
    max_delay_ms: u64,
    jitter: bool,
    retry_on: Option<Value>,
}

enum Backoff {
    Constant,
    Linear,
    Exponential,
}

fn parse_opts(opts: &IndexMap<String, Value>, span: Span) -> Result<RetryOpts, LxError> {
    let max = opts
        .get("max")
        .and_then(|v| v.as_int())
        .and_then(|n| u64::try_from(n).ok())
        .unwrap_or(3);

    let backoff = match opts.get("backoff").and_then(|v| v.as_str()) {
        Some(s) => match s.trim_start_matches(':') {
            "constant" => Backoff::Constant,
            "linear" => Backoff::Linear,
            "exponential" => Backoff::Exponential,
            other => {
                return Err(LxError::runtime(
                    format!("retry: unknown backoff strategy :{other}"),
                    span,
                ));
            }
        },
        None => Backoff::Exponential,
    };

    let base_ms = opts
        .get("base_ms")
        .and_then(|v| v.as_int())
        .and_then(|n| u64::try_from(n).ok())
        .unwrap_or(100);

    let max_delay_ms = opts
        .get("max_delay_ms")
        .and_then(|v| v.as_int())
        .and_then(|n| u64::try_from(n).ok())
        .unwrap_or(30_000);

    let jitter = match opts.get("jitter") {
        Some(v) => match v {
            Value::Bool(b) => *b,
            _ => return Err(LxError::type_err("retry: jitter must be Bool", span)),
        },
        None => true,
    };

    let retry_on = opts.get("retry_on").cloned();

    Ok(RetryOpts {
        max,
        backoff,
        base_ms,
        max_delay_ms,
        jitter,
        retry_on,
    })
}

fn compute_delay(opts: &RetryOpts, attempt: u64) -> u64 {
    let raw = match opts.backoff {
        Backoff::Constant => opts.base_ms,
        Backoff::Linear => opts.base_ms.saturating_mul(attempt),
        Backoff::Exponential => opts.base_ms.saturating_mul(1u64 << attempt.min(32)),
    };
    let capped = raw.min(opts.max_delay_ms);
    if opts.jitter {
        let lo = capped / 2;
        let hi = capped.saturating_add(capped / 2);
        if hi <= lo {
            capped
        } else {
            fastrand::u64(lo..=hi)
        }
    } else {
        capped
    }
}

fn should_retry(
    retry_on: &Option<Value>,
    err_val: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<bool, LxError> {
    match retry_on {
        None => Ok(true),
        Some(pred) => {
            let result = call_value(pred, err_val.clone(), span, ctx)?;
            match result {
                Value::Bool(b) => Ok(b),
                _ => Err(LxError::type_err("retry: retry_on must return Bool", span)),
            }
        }
    }
}

fn run_retry_loop(
    f: &Value,
    opts: &RetryOpts,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let start = Instant::now();
    let mut last_err = Value::Unit;

    for attempt in 0..opts.max {
        if attempt > 0 {
            let delay = compute_delay(opts, attempt);
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }

        match call_value(f, Value::Unit, span, ctx) {
            Ok(Value::Ok(v)) => return Ok(Value::Ok(Box::new(*v))),
            Ok(Value::Err(e)) => {
                last_err = *e;
                if attempt + 1 < opts.max && !should_retry(&opts.retry_on, &last_err, span, ctx)? {
                    break;
                }
            }
            Ok(other) => return Ok(Value::Ok(Box::new(other))),
            Err(LxError::Propagate { value, .. }) => {
                last_err = *value;
                if attempt + 1 < opts.max && !should_retry(&opts.retry_on, &last_err, span, ctx)? {
                    break;
                }
            }
            Err(e) => return Err(e),
        }
    }

    let elapsed_ms = start.elapsed().as_millis() as u64;
    let mut fields = IndexMap::new();
    fields.insert("attempts".into(), Value::Int(BigInt::from(opts.max)));
    fields.insert("last_error".into(), last_err);
    fields.insert("elapsed_ms".into(), Value::Int(BigInt::from(elapsed_ms)));
    let exhausted = Value::Tagged {
        tag: Arc::from("Exhausted"),
        values: Arc::new(vec![Value::Record(Arc::new(fields))]),
    };
    Ok(Value::Err(Box::new(exhausted)))
}

fn bi_retry(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let f = &args[0];
    let opts = RetryOpts {
        max: 3,
        backoff: Backoff::Exponential,
        base_ms: 100,
        max_delay_ms: 30_000,
        jitter: true,
        retry_on: None,
    };
    run_retry_loop(f, &opts, span, ctx)
}

fn bi_retry_with(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(opts_rec) = &args[0] else {
        return Err(LxError::type_err(
            "retry_with: first arg must be Record",
            span,
        ));
    };
    let opts = parse_opts(opts_rec, span)?;
    let f = &args[1];
    run_retry_loop(f, &opts, span, ctx)
}
