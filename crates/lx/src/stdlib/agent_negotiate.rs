use std::sync::Arc;

use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::{call_value_sync, mk};
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub fn mk_negotiate() -> Value {
    mk("agent.negotiate", 2, bi_negotiate)
}

fn ask_agent(
    agent: &Value,
    msg: &Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let Value::Record(r) = agent else {
        return Err(LxError::type_err(
            "agent.negotiate: agent must be a Record",
            span,
        ));
    };
    if let Some(handler) = r
        .get("handler")
        .filter(|h| matches!(h, Value::Func(_) | Value::BuiltinFunc(_)))
    {
        return call_value_sync(handler, msg.clone(), span, ctx);
    }
    if let Some(pid) = r
        .get("__pid")
        .and_then(|v| v.as_int())
        .and_then(|n| n.try_into().ok())
    {
        return super::agent::ask_subprocess(pid, msg, span);
    }
    Err(LxError::type_err(
        "agent.negotiate: agent must have handler or __pid",
        span,
    ))
}

fn agent_name(agent: &Value) -> String {
    match agent {
        Value::Record(r) => r
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed")
            .to_string(),
        _ => "unnamed".into(),
    }
}

fn bi_negotiate(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let agents = args[0]
        .as_list()
        .ok_or_else(|| LxError::type_err("agent.negotiate: first arg must be List", span))?;
    let Value::Record(config) = &args[1] else {
        return Err(LxError::type_err(
            "agent.negotiate: second arg must be Record config",
            span,
        ));
    };
    let proposal = config
        .get("proposal")
        .ok_or_else(|| LxError::runtime("agent.negotiate: config missing 'proposal'", span))?;
    let max_rounds: usize = config
        .get("max_rounds")
        .and_then(|v| match v {
            Value::Int(n) => n.try_into().ok(),
            _ => None,
        })
        .unwrap_or(3);
    let converge_fn = config
        .get("converge")
        .ok_or_else(|| LxError::runtime("agent.negotiate: config missing 'converge'", span))?;
    let on_round = config.get("on_round");
    let mut positions: Vec<Value> = Vec::new();
    for round in 1..=max_rounds {
        let msg = record! {
            "round" => Value::Int(BigInt::from(round)),
            "proposal" => proposal.clone(),
            "positions" => Value::List(Arc::new(positions.clone())),
        };
        let mut responses = Vec::new();
        for agent in agents.iter() {
            let resp = ask_agent(agent, &msg, span, ctx)?;
            let resp = match resp {
                Value::Ok(inner) => *inner,
                other => other,
            };
            responses.push(record! {
                "agent" => Value::Str(Arc::from(agent_name(agent))),
                "position" => resp,
            });
        }
        if let Some(cb) = on_round {
            let partial = call_value_sync(cb, Value::Int(BigInt::from(round)), span, ctx)?;
            call_value_sync(
                &partial,
                Value::List(Arc::new(responses.clone())),
                span,
                ctx,
            )?;
        }
        let resp_list = Arc::new(responses.clone());
        let converge_result =
            call_value_sync(converge_fn, Value::List(Arc::clone(&resp_list)), span, ctx)?;
        match converge_result {
            Value::Ok(result) => {
                return Ok(Value::Ok(Box::new(record! {
                    "result" => *result,
                    "rounds" => Value::Int(BigInt::from(round)),
                    "positions" => Value::List(resp_list),
                    "unanimous" => Value::Bool(round == 1),
                })));
            }
            Value::Str(s) if s.as_ref() == "continue" => {}
            _ => {}
        }
        positions = responses;
    }
    Ok(Value::Err(Box::new(record! {
        "reason" => Value::Str(Arc::from("no_consensus")),
        "rounds" => Value::Int(BigInt::from(max_rounds)),
        "positions" => Value::List(Arc::new(positions)),
    })))
}
