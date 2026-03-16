use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value, mk};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

struct Subscription {
    agent: Value,
    filter: Option<Value>,
}

struct Topic {
    subscribers: Vec<Subscription>,
}

static TOPICS: LazyLock<DashMap<String, Topic>> = LazyLock::new(DashMap::new);

pub fn mk_topic() -> Value {
    mk("agent.topic", 1, bi_topic)
}

pub fn mk_subscribe() -> Value {
    mk("agent.subscribe", 2, bi_subscribe)
}

pub fn mk_subscribe_filtered() -> Value {
    mk("agent.subscribe_filtered", 3, bi_subscribe_filtered)
}

pub fn mk_unsubscribe() -> Value {
    mk("agent.unsubscribe", 2, bi_unsubscribe)
}

pub fn mk_publish() -> Value {
    mk("agent.publish", 2, bi_publish)
}

pub fn mk_publish_collect() -> Value {
    mk("agent.publish_collect", 2, bi_publish_collect)
}

pub fn mk_subscribers() -> Value {
    mk("agent.subscribers", 1, bi_subscribers)
}

pub fn mk_topics() -> Value {
    mk("agent.topics", 1, bi_topics)
}

fn topic_name(v: &Value, span: Span) -> Result<String, LxError> {
    v.as_str()
        .map(String::from)
        .ok_or_else(|| LxError::type_err("agent.topic: name must be Str", span))
}

fn agent_identity(agent: &Value) -> String {
    if let Value::Record(r) = agent {
        if let Some(Value::Str(s)) = r.get("name") {
            return s.to_string();
        }
        if let Some(v) = r.get("__pid") {
            return format!("pid:{v}");
        }
        if let Some(v) = r.get("__mock_id") {
            return format!("mock:{v}");
        }
    }
    format!("{:p}", agent as *const _)
}

fn ask_agent(
    agent: &Value,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    if let Value::Record(r) = agent {
        if let Some(handler) = r.get("handler").filter(|h| {
            matches!(h, Value::Func(_) | Value::BuiltinFunc(_))
        }) {
            return call_value(handler, msg.clone(), span, ctx);
        }
        if let Some(pid) = r
            .get("__pid")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
        {
            return super::agent::ask_subprocess(pid, msg, span);
        }
    }
    Err(LxError::type_err(
        "agent: expected agent with handler or __pid",
        span,
    ))
}

fn bi_topic(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = topic_name(&args[0], span)?;
    TOPICS
        .entry(name.clone())
        .or_insert_with(|| Topic {
            subscribers: Vec::new(),
        });
    Ok(Value::Str(Arc::from(name.as_str())))
}

fn bi_subscribe(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agent = args[0].clone();
    let topic = topic_name(&args[1], span)?;
    let mut entry = TOPICS.entry(topic).or_insert_with(|| Topic {
        subscribers: Vec::new(),
    });
    entry.subscribers.push(Subscription {
        agent,
        filter: None,
    });
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_subscribe_filtered(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let agent = args[0].clone();
    let topic = topic_name(&args[1], span)?;
    let filter = args[2].clone();
    let mut entry = TOPICS.entry(topic).or_insert_with(|| Topic {
        subscribers: Vec::new(),
    });
    entry.subscribers.push(Subscription {
        agent,
        filter: Some(filter),
    });
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn inject_topic(msg: &Value, topic: &str) -> Value {
    if let Value::Record(r) = msg {
        let mut fields = r.as_ref().clone();
        fields.insert("_topic".into(), Value::Str(Arc::from(topic)));
        Value::Record(Arc::new(fields))
    } else {
        let mut fields = IndexMap::new();
        fields.insert("_topic".into(), Value::Str(Arc::from(topic)));
        fields.insert("_value".into(), msg.clone());
        Value::Record(Arc::new(fields))
    }
}

fn bi_publish(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let topic = topic_name(&args[0], span)?;
    let msg = &args[1];
    let enriched = inject_topic(msg, &topic);
    if let Some(t) = TOPICS.get(&topic) {
        for sub in &t.subscribers {
            if let Some(ref filter) = sub.filter {
                let pass = call_value(filter, msg.clone(), span, ctx)?;
                if pass.as_bool() != Some(true) {
                    continue;
                }
            }
            let _ = ask_agent(&sub.agent, &enriched, span, ctx);
        }
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_publish_collect(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let topic = topic_name(&args[0], span)?;
    let msg = &args[1];
    let enriched = inject_topic(msg, &topic);
    let mut results = Vec::new();
    if let Some(t) = TOPICS.get(&topic) {
        for sub in &t.subscribers {
            if let Some(ref filter) = sub.filter {
                let pass = call_value(filter, msg.clone(), span, ctx)?;
                if pass.as_bool() != Some(true) {
                    continue;
                }
            }
            let result = ask_agent(&sub.agent, &enriched, span, ctx)?;
            let result = match result {
                Value::Ok(inner) => *inner,
                other => other,
            };
            let mut rec = IndexMap::new();
            rec.insert(
                "agent".into(),
                Value::Str(Arc::from(agent_identity(&sub.agent))),
            );
            rec.insert("result".into(), result);
            results.push(Value::Record(Arc::new(rec)));
        }
    }
    Ok(Value::Ok(Box::new(Value::List(Arc::new(results)))))
}

fn bi_unsubscribe(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let agent = &args[0];
    let topic = topic_name(&args[1], span)?;
    let agent_id = agent_identity(agent);
    if let Some(mut t) = TOPICS.get_mut(&topic) {
        t.subscribers
            .retain(|s| agent_identity(&s.agent) != agent_id);
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}

fn bi_subscribers(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let topic = topic_name(&args[0], span)?;
    let subs = match TOPICS.get(&topic) {
        Some(t) => t
            .subscribers
            .iter()
            .map(|s| {
                let mut rec = IndexMap::new();
                rec.insert(
                    "agent".into(),
                    Value::Str(Arc::from(agent_identity(&s.agent))),
                );
                rec.insert("filtered".into(), Value::Bool(s.filter.is_some()));
                Value::Record(Arc::new(rec))
            })
            .collect(),
        None => Vec::new(),
    };
    Ok(Value::List(Arc::new(subs)))
}

fn bi_topics(_args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let names: Vec<Value> = TOPICS
        .iter()
        .map(|e| Value::Str(Arc::from(e.key().as_str())))
        .collect();
    Ok(Value::List(Arc::new(names)))
}
