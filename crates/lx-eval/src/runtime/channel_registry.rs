use std::sync::LazyLock;

use dashmap::DashMap;
use miette::SourceSpan;

use lx_value::{BuiltinCtx, BuiltinFunc, BuiltinKind, LxError, LxVal};

static CHANNEL_REGISTRY: LazyLock<DashMap<String, Vec<String>>> = LazyLock::new(DashMap::new);

pub fn create_channel(name: &str) {
  CHANNEL_REGISTRY.entry(name.to_string()).or_default();
}

pub fn channel_subscribe(channel_name: &str, agent_name: &str) -> Result<(), String> {
  let mut entry = CHANNEL_REGISTRY.get_mut(channel_name).ok_or_else(|| format!("channel '{channel_name}' does not exist"))?;
  if !entry.contains(&agent_name.to_string()) {
    entry.push(agent_name.to_string());
  }
  Ok(())
}

pub fn channel_unsubscribe_all(agent_name: &str) {
  for mut entry in CHANNEL_REGISTRY.iter_mut() {
    entry.value_mut().retain(|n| n != agent_name);
  }
}

pub fn channel_members(channel_name: &str) -> Option<Vec<String>> {
  CHANNEL_REGISTRY.get(channel_name).map(|e| e.value().clone())
}

pub fn channel_dispatch(channel_name: &str, method: &str, span: SourceSpan) -> Result<LxVal, LxError> {
  match method {
    "members" => match channel_members(channel_name) {
      Some(names) => Ok(LxVal::list(names.into_iter().map(LxVal::str).collect())),
      None => Err(LxError::runtime(format!("channel '{channel_name}' does not exist"), span)),
    },
    "subscribe" => Ok(LxVal::BuiltinFunc(BuiltinFunc {
      name: "channel.subscribe",
      arity: 2,
      kind: BuiltinKind::Sync(bi_channel_subscribe_impl),
      applied: vec![LxVal::str(channel_name)],
    })),
    "name" => Ok(LxVal::str(channel_name)),
    _ => Err(LxError::type_err(format!("Channel has no method '{method}'"), span, None)),
  }
}

fn bi_channel_subscribe_impl(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let channel_name = args[0].require_str("channel.subscribe", span)?;
  let agent_name = args[1].require_str("channel.subscribe", span)?;
  channel_subscribe(channel_name, agent_name).map_err(|e| LxError::runtime(e, span))?;
  Ok(LxVal::ok_unit())
}
