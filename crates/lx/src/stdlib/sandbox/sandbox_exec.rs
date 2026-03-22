use std::sync::Arc;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

use super::sandbox::{POLICIES, policy_id};

pub fn bi_exec(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let _pid = policy_id(&args[0], span)?;
  let _cmd = match &args[1] {
    LxVal::Str(s) => s.to_string(),
    _ => return Err(LxError::type_err("sandbox.exec expects Str command", span)),
  };

  Ok(LxVal::err_str("shell commands have been removed from lx"))
}

pub fn bi_spawn(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let pid = policy_id(&args[0], span)?;
  let policy = POLICIES.get(&pid).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;

  if !policy.agent {
    return Ok(LxVal::err_str("agent spawning denied by sandbox policy"));
  }

  Ok(LxVal::err_str("sandbox.spawn: OS-level sandboxing not yet implemented — use sandbox.scope for lx-level restriction"))
}
