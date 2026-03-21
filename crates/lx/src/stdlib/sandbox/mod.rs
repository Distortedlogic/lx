use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::LxVal;

use super::sandbox_policy::{
    intersect_policies, make_preset, parse_policy, permits_check, policy_to_describe,
};

#[derive(Clone)]
pub(super) enum ShellPolicy {
    Deny,
    Allow,
    AllowList(Vec<String>),
}

#[derive(Clone)]
pub(super) struct Policy {
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net_allow: Vec<String>,
    pub shell: ShellPolicy,
    pub agent: bool,
    pub mcp: bool,
    pub ai: bool,
    pub embed: bool,
    pub pane: bool,
    pub max_time_ms: u64,
}

pub(super) static POLICIES: LazyLock<DashMap<u64, Policy>> = LazyLock::new(DashMap::new);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub(super) fn policy_id(v: &LxVal, span: Span) -> Result<u64, LxError> {
    match v {
        LxVal::Record(r) => r
            .get("__policy_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::type_err("sandbox: expected policy handle", span)),
        _ => Err(LxError::type_err("sandbox: expected policy Record", span)),
    }
}

fn make_handle(id: u64) -> LxVal {
    record! {
        "__policy_id" => LxVal::Int(BigInt::from(id)),
    }
}

pub fn build() -> IndexMap<String, LxVal> {
    let mut m = IndexMap::new();
    m.insert("policy".into(), mk("sandbox.policy", 1, bi_policy));
    m.insert("describe".into(), mk("sandbox.describe", 1, bi_describe));
    m.insert("permits".into(), mk("sandbox.permits", 3, bi_permits));
    m.insert("merge".into(), mk("sandbox.merge", 1, bi_merge));
    m.insert("attenuate".into(), mk("sandbox.attenuate", 2, bi_attenuate));
    m.insert(
        "scope".into(),
        mk("sandbox.scope", 2, super::sandbox_scope::bi_scope),
    );
    m.insert(
        "exec".into(),
        mk("sandbox.exec", 2, super::sandbox_exec::bi_exec),
    );
    m.insert(
        "spawn".into(),
        mk("sandbox.spawn", 2, super::sandbox_exec::bi_spawn),
    );
    m
}

fn bi_policy(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let policy = match &args[0] {
        LxVal::Str(name) => make_preset(name),
        LxVal::Record(config) => parse_policy(config, span)?,
        _ => {
            return Err(LxError::type_err(
                "sandbox.policy expects Str preset name or config Record",
                span,
            ));
        }
    };
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    POLICIES.insert(id, policy);
    Ok(make_handle(id))
}

fn bi_describe(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let id = policy_id(&args[0], span)?;
    let p = POLICIES
        .get(&id)
        .ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
    Ok(policy_to_describe(&p))
}

fn bi_permits(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let id = policy_id(&args[0], span)?;
    let capability = match &args[1] {
        LxVal::Str(s) => s.to_string(),
        _ => {
            return Err(LxError::type_err(
                "sandbox.permits expects Str capability",
                span,
            ));
        }
    };
    let target = match &args[2] {
        LxVal::Str(s) => s.to_string(),
        _ => {
            return Err(LxError::type_err(
                "sandbox.permits expects Str target",
                span,
            ));
        }
    };
    let p = POLICIES
        .get(&id)
        .ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
    Ok(LxVal::Bool(permits_check(&p, &capability, &target)))
}

fn bi_merge(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let LxVal::List(handles) = &args[0] else {
        return Err(LxError::type_err(
            "sandbox.merge expects List of policy handles",
            span,
        ));
    };
    let mut policies = Vec::new();
    for h in handles.iter() {
        let id = policy_id(h, span)?;
        let p = POLICIES
            .get(&id)
            .ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
        policies.push(p.clone());
    }
    let merged = intersect_policies(&policies);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    POLICIES.insert(id, merged);
    Ok(make_handle(id))
}

fn bi_attenuate(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let parent_id = policy_id(&args[0], span)?;
    let LxVal::Record(overrides) = &args[1] else {
        return Err(LxError::type_err(
            "sandbox.attenuate expects Record overrides",
            span,
        ));
    };
    let parent = POLICIES
        .get(&parent_id)
        .ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?
        .clone();
    let child = parse_policy(overrides, span)?;
    let narrowed = intersect_policies(&[parent, child]);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    POLICIES.insert(id, narrowed);
    Ok(make_handle(id))
}
