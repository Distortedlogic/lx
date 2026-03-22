use indexmap::IndexMap;

use crate::error::LxError;
use crate::record;
use crate::value::LxVal;
use miette::SourceSpan;

use super::sandbox::Policy;

pub(super) fn make_preset(name: &str) -> Policy {
  match name {
    "pure" => Policy { fs_read: vec![], fs_write: vec![], net_allow: vec![], agent: false, mcp: false, ai: false, embed: false, pane: false, max_time_ms: 0 },
    "readonly" => {
      Policy { fs_read: vec![".".into()], fs_write: vec![], net_allow: vec![], agent: false, mcp: false, ai: false, embed: false, pane: false, max_time_ms: 0 }
    },
    "local" => Policy {
      fs_read: vec![".".into()],
      fs_write: vec![".".into()],
      net_allow: vec![],
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

fn extract_string_list(v: &LxVal) -> Vec<String> {
  match v {
    LxVal::List(items) => items
      .iter()
      .filter_map(|item| match item {
        LxVal::Str(s) => Some(s.to_string()),
        _ => None,
      })
      .collect(),
    _ => vec![],
  }
}

pub(super) fn parse_policy(config: &IndexMap<crate::sym::Sym, LxVal>, span: SourceSpan) -> Result<Policy, LxError> {
  let mut p = make_preset("pure");

  if let Some(LxVal::Record(fs)) = config.get(&crate::sym::intern("fs")) {
    if let Some(v) = fs.get(&crate::sym::intern("read")) {
      p.fs_read = extract_string_list(v);
    }
    if let Some(v) = fs.get(&crate::sym::intern("write")) {
      p.fs_write = extract_string_list(v);
    }
  }

  if let Some(LxVal::Record(net)) = config.get(&crate::sym::intern("net"))
    && let Some(v) = net.get(&crate::sym::intern("allow"))
  {
    p.net_allow = extract_string_list(v);
  }

  if let Some(LxVal::Bool(b)) = config.get(&crate::sym::intern("agent")) {
    p.agent = *b;
  }
  if let Some(LxVal::Bool(b)) = config.get(&crate::sym::intern("mcp")) {
    p.mcp = *b;
  }
  if let Some(LxVal::Bool(b)) = config.get(&crate::sym::intern("ai")) {
    p.ai = *b;
  }
  if let Some(LxVal::Bool(b)) = config.get(&crate::sym::intern("embed")) {
    p.embed = *b;
  }
  if let Some(LxVal::Bool(b)) = config.get(&crate::sym::intern("pane")) {
    p.pane = *b;
  }

  if let Some(v) = config.get(&crate::sym::intern("max_time_ms")) {
    match v {
      LxVal::Int(n) => p.max_time_ms = n.try_into().map_err(|_| LxError::type_err("sandbox: max_time_ms must be positive", span))?,
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
    result.agent = result.agent && p.agent;
    result.mcp = result.mcp && p.mcp;
    result.ai = result.ai && p.ai;
    result.embed = result.embed && p.embed;
    result.pane = result.pane && p.pane;
    if p.max_time_ms > 0 && (result.max_time_ms == 0 || p.max_time_ms < result.max_time_ms) {
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

pub(super) fn policy_to_describe(p: &Policy) -> LxVal {
  let to_list = |v: &[String]| -> LxVal { LxVal::list(v.iter().map(LxVal::str).collect()) };
  record! {
      "fs_read" => to_list(&p.fs_read),
      "fs_write" => to_list(&p.fs_write),
      "net" => to_list(&p.net_allow),
      "agent" => LxVal::Bool(p.agent),
      "mcp" => LxVal::Bool(p.mcp),
      "ai" => LxVal::Bool(p.ai),
      "embed" => LxVal::Bool(p.embed),
      "pane" => LxVal::Bool(p.pane),
  }
}

pub(super) fn permits_check(p: &Policy, capability: &str, target: &str) -> bool {
  match capability {
    "fs_read" => path_matches(&p.fs_read, target),
    "fs_write" => path_matches(&p.fs_write, target),
    "net" => path_matches(&p.net_allow, target),
    "ai" => p.ai,
    "agent" => p.agent,
    "mcp" => p.mcp,
    "embed" => p.embed,
    "pane" => p.pane,
    _ => false,
  }
}

fn path_matches(allowed: &[String], target: &str) -> bool {
  allowed.iter().any(|a| a == "*" || a == target || target.starts_with(a))
}
