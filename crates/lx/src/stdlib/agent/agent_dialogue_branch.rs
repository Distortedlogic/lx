use std::sync::Arc;
use std::sync::atomic::Ordering;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::agent_dialogue::{
    DialogueSession, HistoryEntry, NEXT_SESSION_ID, SESSIONS, make_session_record, now_iso,
    session_id_from,
};

pub fn builtins() -> Vec<(&'static str, Value)> {
    vec![
        (
            "dialogue_fork",
            mk("agent.dialogue_fork", 2, bi_dialogue_fork),
        ),
        (
            "dialogue_compare",
            mk("agent.dialogue_compare", 2, bi_dialogue_compare),
        ),
        (
            "dialogue_merge",
            mk("agent.dialogue_merge", 2, bi_dialogue_merge),
        ),
        (
            "dialogue_branches",
            mk("agent.dialogue_branches", 1, bi_dialogue_branches),
        ),
    ]
}

fn bi_dialogue_fork(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = session_id_from(&args[0], span)?;
    let prompts = args[1]
        .as_list()
        .ok_or_else(|| LxError::type_err("agent.dialogue_fork: prompts must be a List", span))?;
    if prompts.is_empty() {
        return Err(LxError::runtime(
            "agent.dialogue_fork: prompts list must not be empty",
            span,
        ));
    }
    let (agent, history_snapshot, role, context, max_turns) = {
        let session = SESSIONS
            .get(&sid)
            .ok_or_else(|| LxError::runtime("agent.dialogue_fork: session not found", span))?;
        if session.suspended {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                "session already has active forks",
            )))));
        }
        (
            session.agent.clone(),
            session.history.clone(),
            session.role.clone(),
            session.context.clone(),
            session.max_turns,
        )
    };
    let now = now_iso();
    let mut fork_sessions = Vec::new();
    let mut fork_ids = Vec::new();
    for prompt in prompts.iter() {
        let fork_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        let mut fork_history = history_snapshot.clone();
        fork_history.push(HistoryEntry {
            role: "user".into(),
            content: prompt.clone(),
            time: now.clone(),
        });
        SESSIONS.insert(
            fork_id,
            DialogueSession {
                agent: agent.clone(),
                history: fork_history,
                role: role.clone(),
                context: context.clone(),
                max_turns,
                parent_id: Some(sid),
                fork_ids: Vec::new(),
                suspended: false,
            },
        );
        fork_ids.push(fork_id);
        fork_sessions.push(make_session_record(fork_id));
    }
    let mut parent = SESSIONS
        .get_mut(&sid)
        .ok_or_else(|| LxError::runtime("agent.dialogue_fork: parent session disappeared", span))?;
    parent.suspended = true;
    parent.fork_ids = fork_ids;
    Ok(Value::Ok(Box::new(Value::Tuple(Arc::new(fork_sessions)))))
}

fn bi_dialogue_compare(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let sessions = args[0].as_list().ok_or_else(|| {
        LxError::type_err(
            "agent.dialogue_compare: first arg must be a List of sessions",
            span,
        )
    })?;
    let opts = match &args[1] {
        Value::Record(r) => r.clone(),
        _ => {
            return Err(LxError::type_err(
                "agent.dialogue_compare: second arg must be a Record with `grade` function",
                span,
            ));
        }
    };
    let grade_fn = opts.get("grade").ok_or_else(|| {
        LxError::runtime("agent.dialogue_compare: opts must have `grade` field", span)
    })?;
    let mut rankings: Vec<(Value, f64, Value)> = Vec::new();
    for session_val in sessions.iter() {
        let grade_result =
            crate::builtins::call_value_sync(grade_fn, session_val.clone(), span, ctx)?;
        let grade_rec = match &grade_result {
            Value::Record(r) => r.clone(),
            _ => {
                return Err(LxError::runtime(
                    "agent.dialogue_compare: grade function must return a Record with `score`",
                    span,
                ));
            }
        };
        let score = grade_rec
            .get("score")
            .and_then(|v| match v {
                Value::Float(f) => Some(*f),
                Value::Int(n) => n.to_f64(),
                _ => None,
            })
            .ok_or_else(|| {
                LxError::runtime(
                    "agent.dialogue_compare: grade result must have numeric `score`",
                    span,
                )
            })?;
        let summary = grade_rec
            .get("summary")
            .cloned()
            .unwrap_or(Value::Str(Arc::from("")));
        rankings.push((session_val.clone(), score, summary));
    }
    rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let best = rankings
        .first()
        .map(|(s, _, _)| s.clone())
        .unwrap_or(Value::Unit);
    let scores: Vec<f64> = rankings.iter().map(|(_, s, _)| *s).collect();
    let spread = scores.iter().cloned().reduce(f64::max).unwrap_or(0.0)
        - scores.iter().cloned().reduce(f64::min).unwrap_or(0.0);
    let ranking_vals: Vec<Value> = rankings
        .iter()
        .map(|(session, score, summary)| {
            record! {
                "session" => session.clone(),
                "score" => Value::Float(*score),
                "summary" => summary.clone(),
            }
        })
        .collect();
    Ok(Value::Ok(Box::new(record! {
        "rankings" => Value::List(Arc::new(ranking_vals)),
        "best" => best,
        "spread" => Value::Float(spread),
    })))
}

fn bi_dialogue_merge(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let parent_sid = session_id_from(&args[0], span)?;
    let winner_sid = session_id_from(&args[1], span)?;
    let (fork_ids, parent_history_len) = {
        let parent = SESSIONS.get(&parent_sid).ok_or_else(|| {
            LxError::runtime("agent.dialogue_merge: parent session not found", span)
        })?;
        if !parent.suspended {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                "parent session has no active forks",
            )))));
        }
        (parent.fork_ids.clone(), parent.history.len())
    };
    let winner_parent = SESSIONS.get(&winner_sid).and_then(|s| s.parent_id);
    if !fork_ids.contains(&winner_sid) || winner_parent != Some(parent_sid) {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "winner is not a fork of this parent",
        )))));
    }
    let post_fork_history = {
        let winner = SESSIONS.get(&winner_sid).ok_or_else(|| {
            LxError::runtime("agent.dialogue_merge: winner session not found", span)
        })?;
        winner.history[parent_history_len..].to_vec()
    };
    for fid in &fork_ids {
        cleanup_fork_tree(*fid);
    }
    let mut parent = SESSIONS.get_mut(&parent_sid).ok_or_else(|| {
        LxError::runtime("agent.dialogue_merge: parent session disappeared", span)
    })?;
    parent.history.extend(post_fork_history);
    parent.suspended = false;
    parent.fork_ids.clear();
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn cleanup_fork_tree(sid: u64) {
    let child_forks = SESSIONS
        .get(&sid)
        .map(|s| s.fork_ids.clone())
        .unwrap_or_default();
    for child in &child_forks {
        cleanup_fork_tree(*child);
    }
    SESSIONS.remove(&sid);
}

fn bi_dialogue_branches(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let sid = session_id_from(&args[0], span)?;
    let session = SESSIONS
        .get(&sid)
        .ok_or_else(|| LxError::runtime("agent.dialogue_branches: session not found", span))?;
    let branches: Vec<Value> = session
        .fork_ids
        .iter()
        .map(|fid| {
            record! {
                "__session_id" => Value::Int(BigInt::from(*fid)),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(branches)))
}
