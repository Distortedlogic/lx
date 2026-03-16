use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use parking_lot::Mutex;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{BuiltinFunc, Value};

static NEXT_MOCK_ID: AtomicU64 = AtomicU64::new(1);
static MOCK_CALLS: std::sync::LazyLock<DashMap<u64, Arc<Mutex<Vec<CallRecord>>>>> =
    std::sync::LazyLock::new(DashMap::new);

struct CallRecord {
    msg: Value,
    response: Value,
}

pub fn mk_mock() -> Value {
    mk("agent.mock", 1, bi_mock)
}

pub fn mk_mock_calls() -> Value {
    mk("agent.mock_calls", 1, bi_mock_calls)
}

pub fn mk_mock_assert_called() -> Value {
    mk("agent.mock_assert_called", 2, bi_mock_assert_called)
}

pub fn mk_mock_assert_not_called() -> Value {
    mk("agent.mock_assert_not_called", 2, bi_mock_assert_not_called)
}

fn bi_mock(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let rules = args[0]
        .as_list()
        .ok_or_else(|| LxError::type_err("agent.mock: first arg must be a List of rules", span))?;
    let mock_id = NEXT_MOCK_ID.fetch_add(1, Ordering::Relaxed);
    MOCK_CALLS.insert(mock_id, Arc::new(Mutex::new(Vec::new())));
    let handler = Value::BuiltinFunc(BuiltinFunc {
        name: "agent.mock.handler",
        arity: 3,
        func: bi_mock_handler,
        applied: vec![
            Value::Int(BigInt::from(mock_id)),
            Value::List(Arc::clone(rules)),
        ],
    });
    let mut rec = IndexMap::new();
    rec.insert("__mock_id".into(), Value::Int(BigInt::from(mock_id)));
    rec.insert("handler".into(), handler);
    Ok(Value::Record(Arc::new(rec)))
}

fn bi_mock_handler(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let mock_id = args[0]
        .as_int()
        .and_then(|n| n.to_u64())
        .ok_or_else(|| LxError::runtime("agent.mock.handler: invalid mock_id", span))?;
    let rules = args[1]
        .as_list()
        .ok_or_else(|| LxError::runtime("agent.mock.handler: invalid rules", span))?;
    let msg = &args[2];
    let response = find_matching_response(rules, msg, span, ctx)?;
    if let Some(calls) = MOCK_CALLS.get(&mock_id) {
        calls.lock().push(CallRecord {
            msg: msg.clone(),
            response: response.clone(),
        });
    }
    Ok(response)
}

fn find_matching_response(
    rules: &Arc<Vec<Value>>,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    for rule in rules.as_ref() {
        let Value::Record(r) = rule else { continue };
        let Some(match_val) = r.get("match") else {
            continue;
        };
        if matches_rule(match_val, msg, span, ctx)? {
            return get_response(r, msg, span, ctx);
        }
    }
    let mut err = IndexMap::new();
    err.insert("error".into(), Value::Str(Arc::from("no matching rule")));
    Ok(Value::Record(Arc::new(err)))
}

fn matches_rule(
    pattern: &Value,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<bool, LxError> {
    match pattern {
        Value::Str(s) if s.as_ref() == "any" => Ok(true),
        Value::Func(_) | Value::BuiltinFunc(_) => {
            let result = call_value(pattern, msg.clone(), span, ctx)?;
            Ok(result.as_bool().unwrap_or(false))
        }
        Value::Record(pat) => match msg {
            Value::Record(msg_rec) => {
                for (key, expected) in pat.iter() {
                    match msg_rec.get(key) {
                        Some(actual) if actual == expected => {}
                        _ => return Ok(false),
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        },
        _ => Ok(pattern == msg),
    }
}

fn get_response(
    rule: &IndexMap<String, Value>,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let Some(respond) = rule.get("respond") else {
        return Ok(Value::Unit);
    };
    match respond {
        Value::Func(_) | Value::BuiltinFunc(_) => call_value(respond, msg.clone(), span, ctx),
        _ => Ok(respond.clone()),
    }
}

fn mock_id_from(val: &Value, span: Span) -> Result<u64, LxError> {
    match val {
        Value::Record(r) => r
            .get("__mock_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.to_u64())
            .ok_or_else(|| {
                LxError::type_err("agent.mock: expected mock agent with __mock_id", span)
            }),
        _ => Err(LxError::type_err(
            "agent.mock: expected mock agent Record",
            span,
        )),
    }
}

fn bi_mock_calls(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let mock_id = mock_id_from(&args[0], span)?;
    let calls = MOCK_CALLS
        .get(&mock_id)
        .ok_or_else(|| LxError::runtime("agent.mock_calls: mock not found", span))?;
    let records: Vec<Value> = calls
        .lock()
        .iter()
        .map(|c| {
            let mut rec = IndexMap::new();
            rec.insert("msg".into(), c.msg.clone());
            rec.insert("response".into(), c.response.clone());
            Value::Record(Arc::new(rec))
        })
        .collect();
    Ok(Value::List(Arc::new(records)))
}

fn bi_mock_assert_called(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let mock_id = mock_id_from(&args[0], span)?;
    let pattern = &args[1];
    let calls = MOCK_CALLS
        .get(&mock_id)
        .ok_or_else(|| LxError::runtime("agent.mock_assert_called: mock not found", span))?;
    let found = calls.lock().iter().any(|c| record_matches(&c.msg, pattern));
    if found {
        Ok(Value::Ok(Box::new(Value::Unit)))
    } else {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "expected call not found",
        )))))
    }
}

fn bi_mock_assert_not_called(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let mock_id = mock_id_from(&args[0], span)?;
    let pattern = &args[1];
    let calls = MOCK_CALLS
        .get(&mock_id)
        .ok_or_else(|| LxError::runtime("agent.mock_assert_not_called: mock not found", span))?;
    let found = calls.lock().iter().any(|c| record_matches(&c.msg, pattern));
    if found {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "unexpected call found",
        )))))
    } else {
        Ok(Value::Ok(Box::new(Value::Unit)))
    }
}

fn record_matches(msg: &Value, pattern: &Value) -> bool {
    match (msg, pattern) {
        (Value::Record(msg_rec), Value::Record(pat)) => {
            pat.iter().all(|(k, v)| msg_rec.get(k) == Some(v))
        }
        _ => msg == pattern,
    }
}
