use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::sandbox::{Policy, ShellPolicy};

pub(super) fn make_preset(name: &str) -> Policy {
    match name {
        "pure" => Policy {
            fs_read: vec![],
            fs_write: vec![],
            net_allow: vec![],
            shell: ShellPolicy::Deny,
            agent: false,
            mcp: false,
            ai: false,
            embed: false,
            pane: false,
            max_time_ms: 0,
        },
        "readonly" => Policy {
            fs_read: vec![".".into()],
            fs_write: vec![],
            net_allow: vec![],
            shell: ShellPolicy::Deny,
            agent: false,
            mcp: false,
            ai: false,
            embed: false,
            pane: false,
            max_time_ms: 0,
        },
        "local" => Policy {
            fs_read: vec![".".into()],
            fs_write: vec![".".into()],
            net_allow: vec![],
            shell: ShellPolicy::Allow,
            agent: false,
            mcp: false,
            ai: false,
            embed: false,
            pane: false,
            max_time_ms: 0,
        },
        "network" => Policy {
            fs_read: vec![".".into()],
            fs_write: vec![".".into()],
            net_allow: vec!["*".into()],
            shell: ShellPolicy::Deny,
            agent: false,
            mcp: false,
            ai: true,
            embed: false,
            pane: false,
            max_time_ms: 0,
        },
        "full" => Policy {
            fs_read: vec!["*".into()],
            fs_write: vec!["*".into()],
            net_allow: vec!["*".into()],
            shell: ShellPolicy::Allow,
            agent: true,
            mcp: true,
            ai: true,
            embed: true,
            pane: true,
            max_time_ms: 0,
        },
        _ => make_preset("pure"),
    }
}

fn extract_string_list(v: &Value) -> Vec<String> {
    match v {
        Value::List(items) => items
            .iter()
            .filter_map(|item| match item {
                Value::Str(s) => Some(s.to_string()),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

pub(super) fn parse_policy(
    config: &IndexMap<String, Value>,
    span: Span,
) -> Result<Policy, LxError> {
    let mut p = make_preset("pure");

    if let Some(Value::Record(fs)) = config.get("fs") {
        if let Some(v) = fs.get("read") {
            p.fs_read = extract_string_list(v);
        }
        if let Some(v) = fs.get("write") {
            p.fs_write = extract_string_list(v);
        }
    }

    if let Some(Value::Record(net)) = config.get("net")
        && let Some(v) = net.get("allow")
    {
        p.net_allow = extract_string_list(v);
    }

    match config.get("shell") {
        Some(Value::Bool(true)) => p.shell = ShellPolicy::Allow,
        Some(Value::Bool(false)) => p.shell = ShellPolicy::Deny,
        Some(Value::Record(r)) => {
            if let Some(v) = r.get("allow") {
                p.shell = ShellPolicy::AllowList(extract_string_list(v));
            }
        }
        _ => {}
    }

    if let Some(Value::Bool(b)) = config.get("agent") {
        p.agent = *b;
    }
    if let Some(Value::Bool(b)) = config.get("mcp") {
        p.mcp = *b;
    }
    if let Some(Value::Bool(b)) = config.get("ai") {
        p.ai = *b;
    }
    if let Some(Value::Bool(b)) = config.get("embed") {
        p.embed = *b;
    }
    if let Some(Value::Bool(b)) = config.get("pane") {
        p.pane = *b;
    }

    if let Some(v) = config.get("max_time_ms") {
        match v {
            Value::Int(n) => {
                p.max_time_ms = n
                    .try_into()
                    .map_err(|_| LxError::type_err("sandbox: max_time_ms must be positive", span))?
            }
            _ => return Err(LxError::type_err("sandbox: max_time_ms must be Int", span)),
        }
    }

    Ok(p)
}

pub(super) fn intersect_policies(policies: &[Policy]) -> Policy {
    let mut result = make_preset("full");
    for p in policies {
        result.fs_read = intersect_paths(&result.fs_read, &p.fs_read);
        result.fs_write = intersect_paths(&result.fs_write, &p.fs_write);
        result.net_allow = intersect_paths(&result.net_allow, &p.net_allow);
        result.shell = intersect_shell(&result.shell, &p.shell);
        result.agent = result.agent && p.agent;
        result.mcp = result.mcp && p.mcp;
        result.ai = result.ai && p.ai;
        result.embed = result.embed && p.embed;
        result.pane = result.pane && p.pane;
        if p.max_time_ms > 0
            && (result.max_time_ms == 0 || p.max_time_ms < result.max_time_ms)
        {
            result.max_time_ms = p.max_time_ms;
        }
    }
    result
}

fn intersect_paths(a: &[String], b: &[String]) -> Vec<String> {
    if a.iter().any(|s| s == "*") {
        return b.to_vec();
    }
    if b.iter().any(|s| s == "*") {
        return a.to_vec();
    }
    a.iter().filter(|x| b.contains(x)).cloned().collect()
}

fn intersect_shell(a: &ShellPolicy, b: &ShellPolicy) -> ShellPolicy {
    match (a, b) {
        (ShellPolicy::Deny, _) | (_, ShellPolicy::Deny) => ShellPolicy::Deny,
        (ShellPolicy::AllowList(la), ShellPolicy::AllowList(lb)) => {
            ShellPolicy::AllowList(la.iter().filter(|x| lb.contains(x)).cloned().collect())
        }
        (ShellPolicy::AllowList(l), ShellPolicy::Allow) => ShellPolicy::AllowList(l.clone()),
        (ShellPolicy::Allow, ShellPolicy::AllowList(l)) => ShellPolicy::AllowList(l.clone()),
        (ShellPolicy::Allow, ShellPolicy::Allow) => ShellPolicy::Allow,
    }
}

pub(super) fn policy_to_describe(p: &Policy) -> Value {
    let shell_val = match &p.shell {
        ShellPolicy::Deny => Value::Bool(false),
        ShellPolicy::Allow => Value::Bool(true),
        ShellPolicy::AllowList(cmds) => Value::List(Arc::new(
            cmds.iter()
                .map(|s| Value::Str(Arc::from(s.as_str())))
                .collect(),
        )),
    };
    let to_list = |v: &[String]| -> Value {
        Value::List(Arc::new(
            v.iter()
                .map(|s| Value::Str(Arc::from(s.as_str())))
                .collect(),
        ))
    };
    record! {
        "fs_read" => to_list(&p.fs_read),
        "fs_write" => to_list(&p.fs_write),
        "net" => to_list(&p.net_allow),
        "shell" => shell_val,
        "agent" => Value::Bool(p.agent),
        "mcp" => Value::Bool(p.mcp),
        "ai" => Value::Bool(p.ai),
        "embed" => Value::Bool(p.embed),
        "pane" => Value::Bool(p.pane),
    }
}

pub(super) fn permits_check(p: &Policy, capability: &str, target: &str) -> bool {
    match capability {
        "fs_read" => path_matches(&p.fs_read, target),
        "fs_write" => path_matches(&p.fs_write, target),
        "net" => path_matches(&p.net_allow, target),
        "shell" => match &p.shell {
            ShellPolicy::Deny => false,
            ShellPolicy::Allow => true,
            ShellPolicy::AllowList(cmds) => cmds.iter().any(|c| c == target),
        },
        "ai" => p.ai,
        "agent" => p.agent,
        "mcp" => p.mcp,
        "embed" => p.embed,
        "pane" => p.pane,
        _ => false,
    }
}

fn path_matches(allowed: &[String], target: &str) -> bool {
    allowed
        .iter()
        .any(|a| a == "*" || a == target || target.starts_with(a))
}
