use std::sync::atomic::Ordering;

use indexmap::IndexMap;
use miette::SourceSpan;

use crate::runtime::agent_registry::{AgentMessage, get_agent_mailbox};
use lx_ast::ast::ExprId;
use lx_value::LxVal;
use lx_value::{EvalResult, LxError};

use super::Interpreter;

impl Interpreter {
  pub(super) async fn eval_tell(&mut self, target: ExprId, msg: ExprId, span: SourceSpan) -> EvalResult<LxVal> {
    let target_val = self.eval(target).await?;
    let target_name = match &target_val {
      LxVal::Str(s) => s.to_string(),
      other => {
        return Err(LxError::type_err(format!("tell target must be Str (agent name), got {}", other.type_name()), span, None).into());
      },
    };
    let msg_val = self.eval(msg).await?;

    let mailbox = get_agent_mailbox(&target_name).ok_or_else(|| LxError::runtime(format!("agent '{target_name}' not running"), span))?;

    let message = AgentMessage { payload: msg_val.clone(), reply: None };

    mailbox.send(message).await.map_err(|_| LxError::runtime(format!("agent '{target_name}' mailbox closed"), span))?;

    let agent_name = self.agent_name.as_deref().unwrap_or("main");
    let mut fields = IndexMap::new();
    fields.insert(lx_span::sym::intern("from"), LxVal::str(agent_name));
    fields.insert(lx_span::sym::intern("to"), LxVal::str(&target_name));
    fields.insert(lx_span::sym::intern("msg"), msg_val);
    self.ctx.event_stream.xadd("agent/tell", agent_name, None, fields);

    Ok(LxVal::Unit)
  }

  pub(super) async fn eval_ask(&mut self, target: ExprId, msg: ExprId, span: SourceSpan) -> EvalResult<LxVal> {
    let target_val = self.eval(target).await?;
    let target_name = match &target_val {
      LxVal::Str(s) => s.to_string(),
      other => {
        return Err(LxError::type_err(format!("ask target must be Str (agent name), got {}", other.type_name()), span, None).into());
      },
    };
    let msg_val = self.eval(msg).await?;

    let mailbox = get_agent_mailbox(&target_name).ok_or_else(|| LxError::runtime(format!("agent '{target_name}' not running"), span))?;

    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel::<LxVal>();

    let message = AgentMessage { payload: msg_val.clone(), reply: Some(reply_tx) };

    mailbox.send(message).await.map_err(|_| LxError::runtime(format!("agent '{target_name}' mailbox closed"), span))?;

    let ask_id = self.next_ask_id.fetch_add(1, Ordering::Relaxed);
    let agent_name = self.agent_name.as_deref().unwrap_or("main");
    let mut fields = IndexMap::new();
    fields.insert(lx_span::sym::intern("ask_id"), LxVal::int(ask_id));
    fields.insert(lx_span::sym::intern("from"), LxVal::str(agent_name));
    fields.insert(lx_span::sym::intern("to"), LxVal::str(&target_name));
    fields.insert(lx_span::sym::intern("msg"), msg_val);
    self.ctx.event_stream.xadd("agent/ask", agent_name, None, fields);

    let result = reply_rx.await.map_err(|_| LxError::runtime(format!("agent '{target_name}' did not reply (handle may have panicked)"), span))?;

    let mut reply_fields = IndexMap::new();
    reply_fields.insert(lx_span::sym::intern("ask_id"), LxVal::int(ask_id));
    reply_fields.insert(lx_span::sym::intern("from"), LxVal::str(&target_name));
    reply_fields.insert(lx_span::sym::intern("to"), LxVal::str(agent_name));
    reply_fields.insert(lx_span::sym::intern("msg"), result.clone());
    self.ctx.event_stream.xadd("agent/reply", &target_name, None, reply_fields);

    Ok(result)
  }
}
