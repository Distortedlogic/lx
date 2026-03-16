use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::Value;

use super::profile::{KnowledgeEntry, PROFILES, Profile, profile_id};

pub(crate) fn profile_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!(".lx/profiles/{name}.json"))
}

pub(crate) fn load_from_disk(name: &str, span: Span) -> Result<Option<Profile>, LxError> {
    let path = profile_path(name);
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| LxError::runtime(format!("profile.load: read: {e}"), span))?;
    let jv: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| LxError::runtime(format!("profile.load: JSON: {e}"), span))?;
    let obj = jv
        .as_object()
        .ok_or_else(|| LxError::runtime("profile.load: expected JSON object", span))?;
    let created = obj
        .get("created")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let updated = obj
        .get("updated")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut knowledge = IndexMap::new();
    if let Some(kv) = obj.get("knowledge").and_then(|v| v.as_object()) {
        for (domain, entry) in kv {
            let data = json_conv::json_to_lx(entry.clone());
            let learned_at = match &data {
                Value::Record(r) => r
                    .get("learned_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                _ => String::new(),
            };
            knowledge.insert(domain.clone(), KnowledgeEntry { data, learned_at });
        }
    }
    let mut preferences = IndexMap::new();
    if let Some(pv) = obj.get("preferences").and_then(|v| v.as_object()) {
        for (k, v) in pv {
            preferences.insert(k.clone(), json_conv::json_to_lx(v.clone()));
        }
    }
    Ok(Some(Profile {
        name: name.to_string(),
        created,
        updated,
        knowledge,
        preferences,
    }))
}

pub(crate) fn persist(p: &Profile, span: Span) -> Result<(), LxError> {
    let path = profile_path(&p.name);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| LxError::runtime(format!("profile.save: mkdir: {e}"), span))?;
    }
    let mut knowledge = serde_json::Map::new();
    for (domain, entry) in &p.knowledge {
        let jv = json_conv::lx_to_json(&entry.data, span)?;
        knowledge.insert(domain.clone(), jv);
    }
    let mut prefs = serde_json::Map::new();
    for (k, v) in &p.preferences {
        let jv = json_conv::lx_to_json(v, span)?;
        prefs.insert(k.clone(), jv);
    }
    let obj = serde_json::json!({
        "name": p.name,
        "created": p.created,
        "updated": p.updated,
        "knowledge": knowledge,
        "preferences": prefs,
    });
    let s = serde_json::to_string_pretty(&obj)
        .map_err(|e| LxError::runtime(format!("profile.save: serialize: {e}"), span))?;
    std::fs::write(profile_path(&p.name), s)
        .map_err(|e| LxError::runtime(format!("profile.save: write: {e}"), span))
}

pub fn bi_history(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let p = PROFILES
        .get(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    let mut entries: Vec<(String, String)> = p
        .knowledge
        .iter()
        .map(|(d, e)| (d.clone(), e.learned_at.clone()))
        .collect();
    entries.sort_by(|a, b| a.1.cmp(&b.1));
    let items: Vec<Value> = entries
        .into_iter()
        .map(|(domain, learned_at)| {
            record! {
                "domain" => Value::Str(Arc::from(domain.as_str())),
                "learned_at" => Value::Str(Arc::from(learned_at.as_str())),
            }
        })
        .collect();
    Ok(Value::List(Arc::new(items)))
}

pub fn bi_merge(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id_a = profile_id(&args[0], span)?;
    let id_b = profile_id(&args[1], span)?;
    let (b_knowledge, b_preferences) = {
        let pb = PROFILES
            .get(&id_b)
            .ok_or_else(|| LxError::runtime("profile: handle B not found", span))?;
        (pb.knowledge.clone(), pb.preferences.clone())
    };
    let mut pa = PROFILES
        .get_mut(&id_a)
        .ok_or_else(|| LxError::runtime("profile: handle A not found", span))?;
    for (k, v) in b_knowledge {
        pa.knowledge.insert(k, v);
    }
    for (k, v) in b_preferences {
        pa.preferences.insert(k, v);
    }
    Ok(args[0].clone())
}

pub fn bi_age(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let domain = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("profile.age: domain must be Str", span))?;
    let p = PROFILES
        .get(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    match p.knowledge.get(domain) {
        Some(entry) => {
            let learned = chrono::DateTime::parse_from_rfc3339(&entry.learned_at)
                .map_err(|e| LxError::runtime(format!("profile.age: parse time: {e}"), span))?;
            let now = chrono::Utc::now();
            let secs = (now - learned.with_timezone(&chrono::Utc))
                .num_seconds()
                .max(0);
            Ok(Value::Ok(Box::new(Value::Int(BigInt::from(secs)))))
        }
        None => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("domain '{domain}' not found").as_str(),
        ))))),
    }
}

pub fn bi_decay(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = profile_id(&args[0], span)?;
    let max_age = args[1]
        .as_int()
        .and_then(|n| i64::try_from(n).ok())
        .ok_or_else(|| LxError::type_err("profile.decay: max_age_secs must be Int", span))?;
    let now = chrono::Utc::now();
    let mut p = PROFILES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("profile: handle not found", span))?;
    let before = p.knowledge.len();
    p.knowledge.retain(|_, entry| {
        let Ok(learned) = chrono::DateTime::parse_from_rfc3339(&entry.learned_at) else {
            return true;
        };
        let age = (now - learned.with_timezone(&chrono::Utc)).num_seconds();
        age <= max_age
    });
    let removed = before - p.knowledge.len();
    Ok(Value::Int(BigInt::from(removed)))
}
