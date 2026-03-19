use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
pub(super) static SESSIONS: std::sync::LazyLock<DashMap<u64, DialogueSession>> =
    std::sync::LazyLock::new(DashMap::new);

pub(super) struct DialogueSession {
    pub(super) agent: Value,
    pub(super) history: Vec<HistoryEntry>,
    pub(super) role: Option<String>,
    context: Option<String>,
    max_turns: Option<usize>,
}

pub(super) struct HistoryEntry {
    role: String,
    content: Value,
    time: String,
}

fn session_id_from(val: &Value, span: Span) -> Result<u64, LxError> {
    match val {
        Value::Record(r) => r
            .get("__session_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.to_u64())
            .ok_or_else(|| {
                LxError::type_err("agent.dialogue: expected session with __session_id", span)
            }),
        _ => Err(LxError::type_err(
            "agent.dialogue: expected session Record",
            span,
        )),
    }
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn build_turn_message(
    session_id: u64,
    content: &Value,
    history: &[HistoryEntry],
    role: &Option<String>,
    context: &Option<String>,
) -> Value {
    let mut msg = IndexMap::new();
    msg.insert("type".into(), Value::Str(Arc::from("dialogue_turn")));
    msg.insert("session_id".into(), Value::Int(BigInt::from(session_id)));
    msg.insert("content".into(), content.clone());
    let hist_list: Vec<Value> = history.iter().map(history_entry_to_value).collect();
    msg.insert("history".into(), Value::List(Arc::new(hist_list)));
    if let Some(r) = role {
        msg.insert("role".into(), Value::Str(Arc::from(r.as_str())));
    }
    if let Some(c) = context {
        msg.insert("context".into(), Value::Str(Arc::from(c.as_str())));
    }
    Value::Record(Arc::new(msg))
}

fn history_entry_to_value(entry: &HistoryEntry) -> Value {
    record! {
        "role" => Value::Str(Arc::from(entry.role.as_str())),
        "content" => entry.content.clone(),
        "time" => Value::Str(Arc::from(entry.time.as_str())),
    }
}

fn make_session_record(session_id: u64) -> Value {
    record! {
        "__session_id" => Value::Int(BigInt::from(session_id)),
    }
}

pub fn mk_dialogue() -> Value {
    mk("agent.dialogue", 2, bi_dialogue)
}

pub fn mk_dialogue_turn() -> Value {
    mk("agent.dialogue_turn", 2, bi_dialogue_turn)
}

pub fn mk_dialogue_history() -> Value {
    mk("agent.dialogue_history", 1, bi_dialogue_history)
}

pub fn mk_dialogue_end() -> Value {
    mk("agent.dialogue_end", 1, bi_dialogue_end)
}

fn bi_dialogue(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = &args[0];
    let config = &args[1];
    let cfg = parse_dialogue_config(config, span)?;
    let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
    SESSIONS.insert(
        session_id,
        DialogueSession {
            agent: agent.clone(),
            history: Vec::new(),
            role: cfg.role,
            context: cfg.context,
            max_turns: cfg.max_turns,
        },
    );
    Ok(Value::Ok(Box::new(make_session_record(session_id))))
}

struct DialogueConfig {
    role: Option<String>,
    context: Option<String>,
    max_turns: Option<usize>,
}

fn parse_dialogue_config(config: &Value, span: Span) -> Result<DialogueConfig, LxError> {
    match config {
        Value::Unit => Ok(DialogueConfig {
            role: None,
            context: None,
            max_turns: None,
        }),
        Value::Record(r) => Ok(DialogueConfig {
            role: r.get("role").and_then(|v| v.as_str()).map(String::from),
            context: r.get("context").and_then(|v| v.as_str()).map(String::from),
            max_turns: r
                .get("max_turns")
                .and_then(|v| v.as_int())
                .and_then(|n| n.to_usize()),
        }),
        _ => Err(LxError::type_err(
            "agent.dialogue: config must be a Record or ()",
            span,
        )),
    }
}

fn ask_agent(
    agent: &Value,
    msg: Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let Value::Record(r) = agent else {
        return Err(LxError::type_err(
            "agent.dialogue: agent must be a Record",
            span,
        ));
    };
    if let Some(pid_val) = r.get("__pid") {
        let pid: u32 = pid_val
            .as_int()
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("agent.dialogue: invalid __pid", span))?;
        return super::agent::ask_subprocess(pid, &msg, span);
    }
    let handler = r.get("handler").ok_or_else(|| {
        LxError::runtime("agent.dialogue: agent has no 'handler' or '__pid'", span)
    })?;
    let result = crate::builtins::call_value(handler, msg, span, ctx)?;
    Ok(result)
}

fn bi_dialogue_turn(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = session_id_from(&args[0], span)?;
    let msg = &args[1];
    let (agent, turn_msg) = {
        let session = SESSIONS.get(&sid).ok_or_else(|| {
            LxError::runtime("agent.dialogue_turn: session not found or ended", span)
        })?;
        if let Some(max) = session.max_turns
            && session.history.len() / 2 >= max
        {
            return Ok(Value::Err(Box::new(record! {
                "exceeded" => Value::Str(Arc::from("max_turns")),
            })));
        }
        let turn_msg =
            build_turn_message(sid, msg, &session.history, &session.role, &session.context);
        (session.agent.clone(), turn_msg)
    };
    let response = ask_agent(&agent, turn_msg, span, ctx)?;
    let response_content = match &response {
        Value::Ok(inner) => (**inner).clone(),
        Value::Err(_) => {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from("disconnected")))));
        }
        other => other.clone(),
    };
    let now = now_iso();
    let mut session = SESSIONS
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("agent.dialogue_turn: session disappeared", span))?;
    session.history.push(HistoryEntry {
        role: "user".into(),
        content: msg.clone(),
        time: now.clone(),
    });
    session.history.push(HistoryEntry {
        role: "agent".into(),
        content: response_content.clone(),
        time: now,
    });
    Ok(Value::Ok(Box::new(response_content)))
}

fn bi_dialogue_history(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let sid = session_id_from(&args[0], span)?;
    let session = SESSIONS.get(&sid).ok_or_else(|| {
        LxError::runtime("agent.dialogue_history: session not found or ended", span)
    })?;
    let entries: Vec<Value> = session.history.iter().map(history_entry_to_value).collect();
    Ok(Value::List(Arc::new(entries)))
}

fn bi_dialogue_end(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = session_id_from(&args[0], span)?;
    SESSIONS.remove(&sid);
    Ok(Value::Ok(Box::new(Value::Unit)))
}
