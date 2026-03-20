use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

use super::agent_dialogue::{
    DialogueSession, HistoryEntry, NEXT_SESSION_ID, SESSIONS, make_session_record, now_iso,
    session_id_from,
};

const DEFAULT_STORAGE: &str = ".lx/dialogues";

fn dialogues_dir() -> PathBuf {
    PathBuf::from(DEFAULT_STORAGE)
}

fn ensure_dir(span: Span) -> Result<PathBuf, LxError> {
    let dir = dialogues_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| LxError::runtime(format!("dialogue_save: mkdir: {e}"), span))?;
    Ok(dir)
}

fn session_to_json(
    id: &str,
    session: &DialogueSession,
    span: Span,
) -> Result<serde_json::Value, LxError> {
    let mut fields = IndexMap::new();
    fields.insert("id".into(), Value::Str(Arc::from(id)));

    let mut config = IndexMap::new();
    match &session.role {
        Some(r) => config.insert("role".into(), Value::Str(Arc::from(r.as_str()))),
        None => config.insert("role".into(), Value::None),
    };
    match &session.context {
        Some(c) => config.insert("context".into(), Value::Str(Arc::from(c.as_str()))),
        None => config.insert("context".into(), Value::None),
    };
    match session.max_turns {
        Some(n) => config.insert("max_turns".into(), Value::Int(BigInt::from(n))),
        None => config.insert("max_turns".into(), Value::None),
    };
    fields.insert("config".into(), Value::Record(Arc::new(config)));

    let turns: Vec<Value> = session
        .history
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let direction = if entry.role == "user" {
                "outbound"
            } else {
                "inbound"
            };
            record! {
                "index" => Value::Int(BigInt::from(i)),
                "direction" => Value::Str(Arc::from(direction)),
                "message" => entry.content.clone(),
                "timestamp" => Value::Str(Arc::from(entry.time.as_str())),
            }
        })
        .collect();
    fields.insert("turns".into(), Value::List(Arc::new(turns)));

    let now = now_iso();
    fields.insert("created".into(), Value::Str(Arc::from(now.as_str())));
    fields.insert("updated".into(), Value::Str(Arc::from(now.as_str())));

    let val = Value::Record(Arc::new(fields));
    json_conv::lx_to_json(&val, span)
}

fn load_session_from_file(
    id: &str,
    agent: Value,
    span: Span,
) -> Result<DialogueSession, LxError> {
    let path = dialogues_dir().join(format!("{id}.json"));
    let content = std::fs::read_to_string(&path).map_err(|e| {
        LxError::runtime(
            format!("dialogue_load: not found: {id} ({e})"),
            span,
        )
    })?;
    let jv: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        LxError::runtime(
            format!("dialogue_load: corrupt JSON for {id}: {e}"),
            span,
        )
    })?;
    let val = json_conv::json_to_lx(jv);
    let Value::Record(r) = &val else {
        return Err(LxError::runtime(
            format!("dialogue_load: expected Record, got {}", val.type_name()),
            span,
        ));
    };

    let config = r.get("config");
    let role = config
        .and_then(|c| match c {
            Value::Record(cr) => cr.get("role").and_then(|v| v.as_str()).map(String::from),
            _ => None,
        });
    let context = config
        .and_then(|c| match c {
            Value::Record(cr) => cr.get("context").and_then(|v| v.as_str()).map(String::from),
            _ => None,
        });
    let max_turns = config
        .and_then(|c| match c {
            Value::Record(cr) => cr
                .get("max_turns")
                .and_then(|v| v.as_int())
                .and_then(|n| n.to_usize()),
            _ => None,
        });

    let history = r
        .get("turns")
        .and_then(|v| v.as_list())
        .map(|turns| {
            turns
                .iter()
                .map(|t| {
                    let Value::Record(tr) = t else {
                        return HistoryEntry {
                            role: "user".into(),
                            content: t.clone(),
                            time: String::new(),
                        };
                    };
                    let direction = tr
                        .get("direction")
                        .and_then(|v| v.as_str())
                        .unwrap_or("outbound");
                    let role_str = if direction == "outbound" {
                        "user"
                    } else {
                        "agent"
                    };
                    let content = tr
                        .get("message")
                        .cloned()
                        .unwrap_or(Value::None);
                    let time = tr
                        .get("timestamp")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    HistoryEntry {
                        role: role_str.into(),
                        content,
                        time,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(DialogueSession {
        agent,
        history,
        role,
        context,
        max_turns,
        parent_id: None,
        fork_ids: Vec::new(),
        suspended: false,
    })
}

pub fn builtins() -> Vec<(&'static str, Value)> {
    vec![
        ("dialogue_save", mk("agent.dialogue_save", 2, bi_save)),
        ("dialogue_load", mk("agent.dialogue_load", 2, bi_load)),
        ("dialogue_list", mk("agent.dialogue_list", 1, bi_list)),
        ("dialogue_delete", mk("agent.dialogue_delete", 1, bi_delete)),
    ]
}

fn bi_save(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let sid = session_id_from(&args[0], span)?;
    let id = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("agent.dialogue_save: id must be Str", span))?;
    let dir = ensure_dir(span)?;
    let session = SESSIONS.get(&sid).ok_or_else(|| {
        LxError::runtime("agent.dialogue_save: session not found", span)
    })?;
    let jv = session_to_json(id, &session, span)?;
    let json_str = serde_json::to_string_pretty(&jv)
        .map_err(|e| LxError::runtime(format!("dialogue_save: serialize: {e}"), span))?;
    let tmp = dir.join(format!("{id}.json.tmp"));
    let final_path = dir.join(format!("{id}.json"));
    std::fs::write(&tmp, &json_str)
        .map_err(|e| LxError::runtime(format!("dialogue_save: write: {e}"), span))?;
    std::fs::rename(&tmp, &final_path)
        .map_err(|e| LxError::runtime(format!("dialogue_save: rename: {e}"), span))?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_load(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("agent.dialogue_load: id must be Str", span))?;
    let agent = &args[1];
    let session = load_session_from_file(id, agent.clone(), span);
    match session {
        Ok(s) => {
            let session_id = NEXT_SESSION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            SESSIONS.insert(session_id, s);
            Ok(Value::Ok(Box::new(make_session_record(session_id))))
        }
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            e.to_string().as_str(),
        ))))),
    }
}

fn bi_list(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    let dir = dialogues_dir();
    if !dir.exists() {
        return Ok(Value::Ok(Box::new(Value::List(Arc::new(Vec::new())))));
    }
    let entries = std::fs::read_dir(&dir)
        .map_err(|e| LxError::runtime(format!("dialogue_list: readdir: {e}"), span))?;
    let mut results = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|e| LxError::runtime(format!("dialogue_list: entry: {e}"), span))?;
        let path = entry.path();
        let Some(ext) = path.extension() else {
            continue;
        };
        if ext != "json" {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if stem.ends_with(".json") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let jv: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let val = json_conv::json_to_lx(jv);
        let Value::Record(r) = &val else { continue };
        let id = r
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(stem)
            .to_string();
        let role = r
            .get("config")
            .and_then(|c| match c {
                Value::Record(cr) => cr.get("role").and_then(|v| v.as_str()).map(String::from),
                _ => None,
            })
            .unwrap_or_default();
        let turns = r
            .get("turns")
            .and_then(|v| v.as_list())
            .map(|l| l.len())
            .unwrap_or(0);
        let created = r
            .get("created")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let updated = r
            .get("updated")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let context_preview = r
            .get("config")
            .and_then(|c| match c {
                Value::Record(cr) => cr.get("context").and_then(|v| v.as_str()).map(|s| {
                    if s.len() > 80 {
                        format!("{}...", &s[..77])
                    } else {
                        s.to_string()
                    }
                }),
                _ => None,
            })
            .unwrap_or_default();

        results.push(record! {
            "id" => Value::Str(Arc::from(id.as_str())),
            "role" => Value::Str(Arc::from(role.as_str())),
            "turns" => Value::Int(BigInt::from(turns)),
            "created" => Value::Str(Arc::from(created.as_str())),
            "updated" => Value::Str(Arc::from(updated.as_str())),
            "context_preview" => Value::Str(Arc::from(context_preview.as_str())),
        });
    }
    Ok(Value::Ok(Box::new(Value::List(Arc::new(results)))))
}

fn bi_delete(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("agent.dialogue_delete: id must be Str", span))?;
    let path = dialogues_dir().join(format!("{id}.json"));
    if !path.exists() {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("dialogue not found: {id}").as_str(),
        )))));
    }
    std::fs::remove_file(&path)
        .map_err(|e| LxError::runtime(format!("dialogue_delete: remove: {e}"), span))?;
    Ok(Value::Ok(Box::new(Value::Unit)))
}
