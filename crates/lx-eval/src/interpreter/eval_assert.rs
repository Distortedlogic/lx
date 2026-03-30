use lx_ast::ast::{BinOp, Expr, ExprId};
use lx_value::LxVal;
use lx_value::{EvalResult, LxError};
use miette::SourceSpan;

use super::Interpreter;

impl Interpreter {
  pub(super) async fn eval_assert(&mut self, expr: ExprId, msg: Option<ExprId>, span: SourceSpan) -> EvalResult<LxVal> {
    let expr_node = self.arena.expr(expr).clone();

    if let Expr::Binary(binary) = &expr_node {
      match binary.op {
        BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
          let left_val = self.eval(binary.left).await?;
          let left_val = self.force_defaults(left_val, span).await?;
          let right_val = self.eval(binary.right).await?;
          let right_val = self.force_defaults(right_val, span).await?;
          let result = self.binary_op(&binary.op, &left_val, &right_val, span)?;

          match result.as_bool() {
            Some(true) => return Ok(LxVal::Unit),
            Some(false) => {
              let message = match msg {
                Some(m) => {
                  let mv = self.eval(m).await?;
                  Some(mv.to_string())
                },
                None => None,
              };
              let expr_text = if span.offset() + span.len() <= self.source.len() {
                self.source[span.offset()..span.offset() + span.len()].to_string()
              } else {
                format!("{} {} {}", left_val.short_display(), binary.op, right_val.short_display())
              };
              return Err(LxError::assert_fail(expr_text, message, Some(right_val.short_display()), Some(left_val.short_display()), span).into());
            },
            _ => {
              return Err(LxError::type_err(format!("assert requires Bool, got {} `{}`", result.type_name(), result.short_display()), span, None).into());
            },
          }
        },
        _ => {},
      }
    }

    let val = self.eval(expr).await?;
    let val = self.force_defaults(val, span).await?;
    match val.as_bool() {
      Some(true) => Ok(LxVal::Unit),
      Some(false) => {
        let message = match msg {
          Some(m) => {
            let mv = self.eval(m).await?;
            Some(mv.to_string())
          },
          None => None,
        };
        let expr_text = if span.offset() + span.len() <= self.source.len() {
          self.source[span.offset()..span.offset() + span.len()].to_string()
        } else {
          format!("{expr_node:?}")
        };
        Err(LxError::assert_fail(expr_text, message, None, None, span).into())
      },
      _ => Err(LxError::type_err(format!("assert requires Bool, got {} `{}`", val.type_name(), val.short_display()), span, None).into()),
    }
  }
}
