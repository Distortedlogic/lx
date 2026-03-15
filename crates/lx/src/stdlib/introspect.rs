use std::sync::{Arc, LazyLock};
use std::time::Instant;

use indexmap::IndexMap;
use num_bigint::BigInt;
use parking_lot::Mutex;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

struct State {
    start: Instant,
    actions: Vec<ActionEntry>,
    markers: IndexMap<String, usize>,
    turns: u64,
}

struct ActionEntry {
    action_type: String,
    target: String,
    timestamp: String,
    is_marker: bool,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State {
    start: Instant::now(),
    actions: Vec::new(),
    markers: IndexMap::new(),
    turns: 0,
}));

const MAX_ACTIONS: usize = 1000;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("self".into(), mk("introspect.self", 1, bi_self));
    m.insert("elapsed".into(), mk("introspect.elapsed", 1, bi_elapsed));
    m.insert("turn_count".into(), mk("introspect.turn_count", 1, bi_turn_count));
    m.insert("tick_turn".into(), mk("introspect.tick_turn", 1, bi_tick_turn));
    m.insert("budget".into(), mk("introspect.budget", 1, bi_budget));
    m.insert("actions".into(), mk("introspect.actions", 1, bi_actions));
    m.insert("actions_since".into(), mk("introspect.actions_since", 1, bi_actions_since));
    m.insert("mark".into(), mk("introspect.mark", 1, bi_mark));
    m.insert("record".into(), mk("introspect.record", 1, bi_record));
    m.insert("is_stuck".into(), mk("introspect.is_stuck", 1, bi_is_stuck));
    m.insert("strategy_shift".into(), mk("introspect.strategy_shift", 1, bi_strategy_shift));
    m.insert("similar_actions".into(), mk("introspect.similar_actions", 1, bi_similar));
    m
}

fn now_str() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn action_to_value(e: &ActionEntry) -> Value {
    let mut f = IndexMap::new();
    f.insert("type".into(), Value::Str(Arc::from(e.action_type.as_str())));
    f.insert("target".into(), Value::Str(Arc::from(e.target.as_str())));
    f.insert("time".into(), Value::Str(Arc::from(e.timestamp.as_str())));
    Value::Record(Arc::new(f))
}

fn bi_self(_args: &[Value], _span: Span) -> Result<Value, LxError> {
    let pid = std::process::id();
    let mut f = IndexMap::new();
    f.insert("name".into(), Value::Str(Arc::from("main")));
    f.insert("role".into(), Value::Str(Arc::from("main")));
    f.insert("pid".into(), Value::Int(BigInt::from(pid)));
    Ok(Value::Record(Arc::new(f)))
}

fn bi_elapsed(_args: &[Value], _span: Span) -> Result<Value, LxError> {
    let state = STATE.lock();
    Ok(Value::Float(state.start.elapsed().as_secs_f64()))
}

fn bi_turn_count(_args: &[Value], _span: Span) -> Result<Value, LxError> {
    let state = STATE.lock();
    Ok(Value::Int(BigInt::from(state.turns)))
}

fn bi_tick_turn(_args: &[Value], _span: Span) -> Result<Value, LxError> {
    let mut state = STATE.lock();
    state.turns += 1;
    Ok(Value::Int(BigInt::from(state.turns)))
}

fn bi_budget(_args: &[Value], _span: Span) -> Result<Value, LxError> {
    let mut f = IndexMap::new();
    f.insert("total".into(), Value::Int(BigInt::from(-1)));
    f.insert("spent".into(), Value::Int(BigInt::from(0)));
    f.insert("remaining".into(), Value::Int(BigInt::from(-1)));
    Ok(Value::Record(Arc::new(f)))
}

fn bi_actions(_args: &[Value], _span: Span) -> Result<Value, LxError> {
    let state = STATE.lock();
    let items: Vec<Value> = state.actions.iter()
        .filter(|a| !a.is_marker)
        .map(action_to_value)
        .collect();
    Ok(Value::List(Arc::new(items)))
}

fn bi_actions_since(args: &[Value], span: Span) -> Result<Value, LxError> {
    let marker = args[0].as_str()
        .ok_or_else(|| LxError::type_err("introspect.actions_since: expects Str marker", span))?;
    let state = STATE.lock();
    let start_idx = state.markers.get(marker).copied().unwrap_or(0);
    let items: Vec<Value> = state.actions[start_idx..].iter()
        .filter(|a| !a.is_marker)
        .map(action_to_value)
        .collect();
    Ok(Value::List(Arc::new(items)))
}

fn bi_mark(args: &[Value], span: Span) -> Result<Value, LxError> {
    let name = args[0].as_str()
        .ok_or_else(|| LxError::type_err("introspect.mark: expects Str name", span))?;
    let mut state = STATE.lock();
    let idx = state.actions.len();
    state.markers.insert(name.to_string(), idx);
    state.actions.push(ActionEntry {
        action_type: "marker".to_string(),
        target: name.to_string(),
        timestamp: now_str(),
        is_marker: true,
    });
    trim_actions(&mut state);
    Ok(Value::Unit)
}

fn bi_record(args: &[Value], span: Span) -> Result<Value, LxError> {
    let Value::Record(r) = &args[0] else {
        return Err(LxError::type_err("introspect.record expects Record", span));
    };
    let action_type = r.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
    let target = r.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let mut state = STATE.lock();
    state.actions.push(ActionEntry {
        action_type,
        target,
        timestamp: now_str(),
        is_marker: false,
    });
    trim_actions(&mut state);
    Ok(Value::Unit)
}

fn bi_is_stuck(_args: &[Value], _span: Span) -> Result<Value, LxError> {
    let state = STATE.lock();
    let real_actions: Vec<&ActionEntry> = state.actions.iter()
        .filter(|a| !a.is_marker)
        .collect();
    if real_actions.len() < 5 {
        return Ok(Value::Bool(false));
    }
    let last5 = &real_actions[real_actions.len() - 5..];
    let first = &last5[0];
    let stuck = last5.iter().all(|a| a.action_type == first.action_type && a.target == first.target);
    Ok(Value::Bool(stuck))
}

fn bi_strategy_shift(args: &[Value], span: Span) -> Result<Value, LxError> {
    let reason = args[0].as_str()
        .ok_or_else(|| LxError::type_err("introspect.strategy_shift: expects Str", span))?;
    let mut state = STATE.lock();
    state.actions.push(ActionEntry {
        action_type: "strategy_shift".to_string(),
        target: reason.to_string(),
        timestamp: now_str(),
        is_marker: true,
    });
    trim_actions(&mut state);
    Ok(Value::Unit)
}

fn bi_similar(args: &[Value], span: Span) -> Result<Value, LxError> {
    let n: usize = args[0].as_int()
        .ok_or_else(|| LxError::type_err("introspect.similar_actions: expects Int", span))?
        .try_into()
        .map_err(|_| LxError::runtime("introspect.similar_actions: n too large", span))?;
    let state = STATE.lock();
    let real: Vec<&ActionEntry> = state.actions.iter()
        .filter(|a| !a.is_marker)
        .collect();
    if real.is_empty() {
        return Ok(Value::Int(BigInt::from(0)));
    }
    let window = if n > real.len() { &real[..] } else { &real[real.len() - n..] };
    let Some(last) = window.last() else {
        return Ok(Value::Int(BigInt::from(0)));
    };
    let count = window.iter()
        .filter(|a| a.action_type == last.action_type && a.target == last.target)
        .count();
    Ok(Value::Int(BigInt::from(count)))
}

fn trim_actions(state: &mut State) {
    if state.actions.len() > MAX_ACTIONS {
        let excess = state.actions.len() - MAX_ACTIONS;
        state.actions.drain(..excess);
        state.markers.retain(|_, idx| {
            if *idx >= excess { *idx -= excess; true } else { false }
        });
    }
}

