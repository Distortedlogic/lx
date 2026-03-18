use std::io::{BufReader, BufWriter};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::{AgentEvent, RuntimeCtx};
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub use super::agent_ipc::{ask_subprocess, send_subprocess};

pub(super) struct AgentProcess {
    pub(super) _child: Child,
    pub(super) stdin: BufWriter<ChildStdin>,
    pub(super) stdout: BufReader<ChildStdout>,
}

pub(super) static REGISTRY: LazyLock<DashMap<u32, AgentProcess>> = LazyLock::new(DashMap::new);

pub(super) fn get_pid(agent: &Value, span: Span) -> Result<u32, LxError> {
    match agent {
        Value::Record(r) => r
            .get("__pid")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("agent: expected agent record with __pid", span)),
        _ => Err(LxError::type_err("agent: expected agent Record", span)),
    }
}

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("spawn".into(), mk("agent.spawn", 1, bi_spawn));
    m.insert("kill".into(), mk("agent.kill", 1, bi_kill));
    for (name, val) in super::agent_ipc::builtins() {
        m.insert(name.into(), val);
    }
    m.insert("reconcile".into(), super::agent_reconcile::mk_reconcile());
    m.insert("intercept".into(), super::agent_intercept::mk_intercept());
    m.insert(
        "Handoff".into(),
        super::agent_handoff::mk_handoff_protocol(),
    );
    m.insert("as_context".into(), super::agent_handoff::mk_as_context());
    m.insert(
        "Capabilities".into(),
        super::agent_capability::mk_capabilities_protocol(),
    );
    m.insert(
        "capabilities".into(),
        super::agent_capability::mk_capabilities(),
    );
    m.insert("advertise".into(), super::agent_capability::mk_advertise());
    m.insert(
        "GateResult".into(),
        super::agent_gate::mk_gate_result_protocol(),
    );
    m.insert("gate".into(), super::agent_gate::mk_gate());
    m.insert("dispatch".into(), super::agent_dispatch::mk_dispatch());
    m.insert(
        "dispatch_multi".into(),
        super::agent_dispatch::mk_dispatch_multi(),
    );
    m.insert("mock".into(), super::agent_mock::mk_mock());
    m.insert("mock_calls".into(), super::agent_mock::mk_mock_calls());
    m.insert(
        "mock_assert_called".into(),
        super::agent_mock::mk_mock_assert_called(),
    );
    m.insert(
        "mock_assert_not_called".into(),
        super::agent_mock::mk_mock_assert_not_called(),
    );
    m.insert("supervise".into(), super::agent_supervise::mk_supervise());
    m.insert("child".into(), super::agent_supervise::mk_child());
    m.insert(
        "supervise_stop".into(),
        super::agent_supervise::mk_supervise_stop(),
    );
    m.insert("dialogue".into(), super::agent_dialogue::mk_dialogue());
    m.insert(
        "dialogue_turn".into(),
        super::agent_dialogue::mk_dialogue_turn(),
    );
    m.insert(
        "dialogue_history".into(),
        super::agent_dialogue::mk_dialogue_history(),
    );
    m.insert(
        "dialogue_end".into(),
        super::agent_dialogue::mk_dialogue_end(),
    );
    m.insert("negotiate".into(), super::agent_negotiate::mk_negotiate());
    m.insert("register".into(), super::agent_route::mk_register());
    m.insert("unregister".into(), super::agent_route::mk_unregister());
    m.insert("registered".into(), super::agent_route::mk_registered());
    m.insert("route".into(), super::agent_route::mk_route());
    m.insert("route_multi".into(), super::agent_route::mk_route_multi());
    m.insert("topic".into(), super::agent_pubsub::mk_topic());
    m.insert("subscribe".into(), super::agent_pubsub::mk_subscribe());
    m.insert(
        "subscribe_filtered".into(),
        super::agent_pubsub::mk_subscribe_filtered(),
    );
    m.insert("unsubscribe".into(), super::agent_pubsub::mk_unsubscribe());
    m.insert("publish".into(), super::agent_pubsub::mk_publish());
    m.insert(
        "publish_collect".into(),
        super::agent_pubsub::mk_publish_collect(),
    );
    m.insert("subscribers".into(), super::agent_pubsub::mk_subscribers());
    m.insert("topics".into(), super::agent_pubsub::mk_topics());
    for (name, val) in super::agent_errors::tagged_ctors() {
        m.insert(name.into(), val);
    }
    m
}

fn bi_spawn(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(config) = &args[0] else {
        return Err(LxError::type_err("agent.spawn expects Record config", span));
    };
    let script = config
        .get("script")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            LxError::runtime("agent.spawn: config must have 'script' field (Str)", span)
        })?;
    let name = config
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed")
        .to_string();
    let lx_bin = std::env::current_exe()
        .map_err(|e| LxError::runtime(format!("agent.spawn: cannot find lx binary: {e}"), span))?;
    let mut child = Command::new(lx_bin)
        .arg("agent")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| LxError::runtime(format!("agent.spawn: failed: {e}"), span))?;
    let stdin = BufWriter::new(
        child
            .stdin
            .take()
            .ok_or_else(|| LxError::runtime("agent.spawn: no stdin pipe", span))?,
    );
    let stdout = BufReader::new(
        child
            .stdout
            .take()
            .ok_or_else(|| LxError::runtime("agent.spawn: no stdout pipe", span))?,
    );
    let pid = child.id();
    REGISTRY.insert(
        pid,
        AgentProcess {
            _child: child,
            stdin,
            stdout,
        },
    );
    let mut rec = IndexMap::new();
    rec.insert("__pid".into(), Value::Int(BigInt::from(pid)));
    rec.insert("name".into(), Value::Str(Arc::from(name.as_str())));
    if let Some(Value::List(traits)) = config.get("implements") {
        let trait_names: Vec<Value> = traits
            .iter()
            .filter_map(|t| {
                if let Value::Trait { name, .. } = t {
                    Some(Value::Str(Arc::clone(name)))
                } else {
                    None
                }
            })
            .collect();
        rec.insert("__traits".into(), Value::List(Arc::new(trait_names)));
    }
    if let Some(ref cb) = ctx.on_agent_event {
        cb(AgentEvent::Spawned {
            id: pid.to_string(),
            name: name.clone(),
        });
    }
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(rec)))))
}

fn bi_kill(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = get_pid(&args[0], span)?;
    match REGISTRY.remove(&pid) {
        Some((_, mut agent)) => {
            if let Err(e) = agent._child.kill() {
                eprintln!("agent.kill: kill failed for pid {pid}: {e}");
            }
            if let Err(e) = agent._child.wait() {
                eprintln!("agent.kill: wait failed for pid {pid}: {e}");
            }
            if let Some(ref cb) = ctx.on_agent_event {
                cb(AgentEvent::Killed {
                    id: pid.to_string(),
                });
            }
            Ok(Value::Ok(Box::new(Value::Unit)))
        }
        None => Ok(Value::Err(Box::new(super::agent_errors::unavailable(
            &format!("pid:{pid}"),
            "agent not found in registry",
        )))),
    }
}
