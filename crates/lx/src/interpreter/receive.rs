use std::sync::Arc;

use crate::ast::ReceiveArm;
use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::Interpreter;

impl Interpreter {
    pub(super) async fn eval_receive(
        &mut self,
        arms: &[ReceiveArm],
        span: Span,
    ) -> Result<LxVal, LxError> {
        let ready_msg = crate::record! {
            "kind" => LxVal::Str(Arc::from("ready"))
        };
        let mut msg = self.ctx.yield_.yield_value(ready_msg, span)?;

        loop {
            let action = match &msg {
                LxVal::Record(r) => r
                    .get("action")
                    .cloned()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default(),
                LxVal::None => break,
                _ => String::new(),
            };

            let mut result = LxVal::Err(Box::new(LxVal::Str(Arc::from(format!(
                "unknown action: {action}"
            )))));

            for arm in arms {
                if arm.action == action || arm.action == "_" {
                    let handler = self.eval(&arm.handler).await?;
                    result = self.apply_func(handler, msg.clone(), span).await?;
                    break;
                }
            }

            let response = crate::record! {
                "kind" => LxVal::Str(Arc::from("result")),
                "data" => result
            };
            msg = self.ctx.yield_.yield_value(response, span)?;

            if matches!(msg, LxVal::None) {
                break;
            }
        }

        Ok(LxVal::Unit)
    }
}
