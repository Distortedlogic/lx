use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;

use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::std_module;
use crate::stdlib::helpers::extract_handle_id;
use crate::value::LxVal;
use miette::SourceSpan;

use super::sandbox_policy::{intersect_policies, make_preset, parse_policy, permits_check, policy_to_describe};

#[derive(Clone)]
pub(super) struct Policy {
  pub fs_read: Vec<String>,
  pub fs_write: Vec<String>,
  pub net_allow: Vec<String>,
  pub agent: bool,
  pub mcp: bool,
  pub ai: bool,
  pub embed: bool,
  pub pane: bool,
  pub max_time_ms: u64,
}

pub(super) static POLICIES: LazyLock<DashMap<u64, Policy>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub(super) fn policy_id(v: &LxVal, span: SourceSpan) -> Result<u64, LxError> {
  extract_handle_id(v, "__policy_id", "sandbox", span)
}

fn make_handle(id: u64) -> LxVal {
  record! {
      "__policy_id" => LxVal::int(id),
  }
}

pub fn build() -> IndexMap<crate::sym::Sym, LxVal> {
  std_module! {
    "policy"    => "sandbox.policy",    1, bi_policy;
    "describe"  => "sandbox.describe",  1, bi_describe;
    "permits"   => "sandbox.permits",   3, bi_permits;
    "merge"     => "sandbox.merge",     1, bi_merge;
    "attenuate" => "sandbox.attenuate", 2, bi_attenuate;
    "scope"     => "sandbox.scope",     2, super::sandbox_scope::bi_scope;
    "exec"      => "sandbox.exec",      2, super::sandbox_exec::bi_exec;
    "spawn"     => "sandbox.spawn",     2, super::sandbox_exec::bi_spawn
  }
}

fn bi_policy(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let policy = match &args[0] {
    LxVal::Str(name) => make_preset(name),
    LxVal::Record(config) => parse_policy(config, span)?,
    _ => {
      return Err(LxError::type_err("sandbox.policy expects Str preset name or config Record", span));
    },
  };
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  POLICIES.insert(id, policy);
  Ok(make_handle(id))
}

fn bi_describe(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = policy_id(&args[0], span)?;
  let p = POLICIES.get(&id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
  Ok(policy_to_describe(&p))
}

fn bi_permits(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = policy_id(&args[0], span)?;
  let capability = match &args[1] {
    LxVal::Str(s) => s.to_string(),
    _ => {
      return Err(LxError::type_err("sandbox.permits expects Str capability", span));
    },
  };
  let target = match &args[2] {
    LxVal::Str(s) => s.to_string(),
    _ => {
      return Err(LxError::type_err("sandbox.permits expects Str target", span));
    },
  };
  let p = POLICIES.get(&id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
  Ok(LxVal::Bool(permits_check(&p, &capability, &target)))
}

fn bi_merge(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let LxVal::List(handles) = &args[0] else {
    return Err(LxError::type_err("sandbox.merge expects List of policy handles", span));
  };
  let mut policies = Vec::new();
  for h in handles.iter() {
    let id = policy_id(h, span)?;
    let p = POLICIES.get(&id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
    policies.push(p.clone());
  }
  let merged = intersect_policies(&policies);
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  POLICIES.insert(id, merged);
  Ok(make_handle(id))
}

fn bi_attenuate(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let parent_id = policy_id(&args[0], span)?;
  let LxVal::Record(overrides) = &args[1] else {
    return Err(LxError::type_err("sandbox.attenuate expects Record overrides", span));
  };
  let parent = POLICIES.get(&parent_id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?.clone();
  let child = parse_policy(overrides, span)?;
  let narrowed = intersect_policies(&[parent, child]);
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  POLICIES.insert(id, narrowed);
  Ok(make_handle(id))
}
