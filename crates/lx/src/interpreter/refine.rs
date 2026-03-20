use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::ast::SExpr;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::Interpreter;

pub(super) struct RefineArgs<'a> {
    pub(super) initial: &'a SExpr,
    pub(super) grade: &'a SExpr,
    pub(super) revise: &'a SExpr,
    pub(super) threshold: &'a SExpr,
    pub(super) max_rounds: &'a SExpr,
    pub(super) on_round: Option<&'a SExpr>,
}

fn extract_score(grade_result: &Value, span: Span) -> Result<i64, LxError> {
    match grade_result {
        Value::Record(fields) => {
            let score = fields.get("score").ok_or_else(|| {
                LxError::runtime(
                    "refine: grade function must return record with 'score' field",
                    span,
                )
            })?;
            score
                .as_int()
                .and_then(|n| n.to_i64())
                .ok_or_else(|| LxError::type_err("refine: score must be Int", span))
        }
        _ => Err(LxError::type_err(
            "refine: grade function must return a record with 'score' and 'feedback' fields",
            span,
        )),
    }
}

fn extract_feedback(grade_result: &Value, span: Span) -> Result<Value, LxError> {
    match grade_result {
        Value::Record(fields) => {
            let feedback = fields.get("feedback").ok_or_else(|| {
                LxError::runtime("refine: grade result must have 'feedback' field", span)
            })?;
            Ok(feedback.clone())
        }
        _ => Err(LxError::type_err("refine: grade must return record", span)),
    }
}

fn make_refine_result(
    tag: &str,
    work: Value,
    rounds: i64,
    final_score: i64,
    reason: Option<&str>,
) -> Value {
    let mut fields = IndexMap::new();
    fields.insert("work".into(), work);
    fields.insert("rounds".into(), Value::Int(BigInt::from(rounds)));
    fields.insert("final_score".into(), Value::Int(BigInt::from(final_score)));
    if let Some(r) = reason {
        fields.insert("reason".into(), Value::Str(Arc::from(r)));
    }
    let record = Value::Record(Arc::new(fields));
    match tag {
        "ok" => Value::Ok(Box::new(record)),
        _ => Value::Err(Box::new(record)),
    }
}

impl Interpreter {
    pub(super) async fn eval_refine(
        &mut self,
        args: &RefineArgs<'_>,
        span: Span,
    ) -> Result<Value, LxError> {
        let mut work = self.eval(args.initial).await?;
        let grade_fn = self.eval(args.grade).await?;
        let revise_fn = self.eval(args.revise).await?;
        let threshold_val = self.eval(args.threshold).await?;
        let max_rounds_val = self.eval(args.max_rounds).await?;
        let on_round_fn = match args.on_round {
            Some(e) => Some(self.eval(e).await?),
            None => None,
        };

        let threshold = threshold_val
            .as_int()
            .and_then(|n| n.to_i64())
            .ok_or_else(|| LxError::type_err("refine: threshold must be Int", span))?;
        let max_rounds = max_rounds_val
            .as_int()
            .and_then(|n| n.to_i64())
            .ok_or_else(|| LxError::type_err("refine: max_rounds must be Int", span))?;

        let mut grade_result =
            crate::builtins::call_value(&grade_fn, work.clone(), span, &self.ctx).await?;
        let mut score = extract_score(&grade_result, span)?;

        if score >= threshold {
            return Ok(make_refine_result("ok", work, 0, score, None));
        }

        for round in 1..=max_rounds {
            let feedback = extract_feedback(&grade_result, span)?;
            let revised =
                crate::builtins::call_value(&revise_fn, work.clone(), span, &self.ctx).await?;
            let revised = crate::builtins::call_value(&revised, feedback, span, &self.ctx).await?;
            work = revised;

            grade_result =
                crate::builtins::call_value(&grade_fn, work.clone(), span, &self.ctx).await?;
            score = extract_score(&grade_result, span)?;

            if let Some(ref cb) = on_round_fn {
                let arg = Value::Tuple(Arc::new(vec![
                    Value::Int(BigInt::from(round)),
                    work.clone(),
                    Value::Int(BigInt::from(score)),
                ]));
                crate::builtins::call_value(cb, arg, span, &self.ctx).await?;
            }

            if score >= threshold {
                return Ok(make_refine_result("ok", work, round, score, None));
            }
        }

        Ok(make_refine_result(
            "err",
            work,
            max_rounds,
            score,
            Some("max_rounds"),
        ))
    }
}
