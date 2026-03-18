use std::sync::Arc;

use crate::ast::ReceiveArm;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::Interpreter;

impl Interpreter {
    pub(super) fn eval_receive(
        &mut self,
        arms: &[ReceiveArm],
        span: Span,
    ) -> Result<Value, LxError> {
        let ready_msg = crate::record! {
            "kind" => Value::Str(Arc::from("ready"))
        };
        let mut msg = self.ctx.yield_.yield_value(ready_msg, span)?;

        loop {
            let action = match &msg {
                Value::Record(r) => r
                    .get("action")
                    .cloned()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default(),
                Value::None => break,
                _ => String::new(),
            };

            let mut result = Value::Err(Box::new(Value::Str(Arc::from(format!(
                "unknown action: {action}"
            )))));

            for arm in arms {
                if arm.action == action || arm.action == "_" {
                    let handler = self.eval(&arm.handler)?;
                    result = self.apply_func(handler, msg.clone(), span)?;
                    break;
                }
            }

            let response = crate::record! {
                "kind" => Value::Str(Arc::from("result")),
                "data" => result
            };
            msg = self.ctx.yield_.yield_value(response, span)?;

            if matches!(msg, Value::None) {
                break;
            }
        }

        Ok(Value::Unit)
    }
}
