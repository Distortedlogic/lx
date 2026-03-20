use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::sandbox::{POLICIES, ShellPolicy, policy_id};

pub fn bi_exec(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = policy_id(&args[0], span)?;
    let cmd = match &args[1] {
        Value::Str(s) => s.to_string(),
        _ => return Err(LxError::type_err("sandbox.exec expects Str command", span)),
    };

    let policy = POLICIES
        .get(&pid)
        .ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;

    match &policy.shell {
        ShellPolicy::Deny => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "shell access denied by sandbox policy",
        ))))),
        ShellPolicy::AllowList(cmds) => {
            let first_word = cmd.split_whitespace().next().unwrap_or("");
            if cmds.iter().any(|c| c == first_word) {
                ctx.shell.exec(&cmd, span)
            } else {
                Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
                    "command '{first_word}' not allowed by sandbox policy"
                ))))))
            }
        }
        ShellPolicy::Allow => ctx.shell.exec(&cmd, span),
    }
}

pub fn bi_spawn(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = policy_id(&args[0], span)?;
    let policy = POLICIES
        .get(&pid)
        .ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;

    if !policy.agent {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "agent spawning denied by sandbox policy",
        )))));
    }

    Ok(Value::Err(Box::new(Value::Str(Arc::from(
        "sandbox.spawn: OS-level sandboxing not yet implemented — use sandbox.scope for lx-level restriction",
    )))))
}
