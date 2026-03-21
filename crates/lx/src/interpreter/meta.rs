use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::ast::SExpr;
use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::Interpreter;

pub(super) struct MetaArgs<'a> {
    pub(super) task: &'a SExpr,
    pub(super) strategies: &'a SExpr,
    pub(super) attempt: &'a SExpr,
    pub(super) evaluate: &'a SExpr,
    pub(super) select: Option<&'a SExpr>,
    pub(super) on_switch: Option<&'a SExpr>,
}

fn make_attempt_record(strategy: &LxVal, quality: i64, viable: bool, reason: LxVal) -> LxVal {
    let mut fields = IndexMap::new();
    fields.insert("strategy".into(), strategy.clone());
    fields.insert("quality".into(), LxVal::Int(BigInt::from(quality)));
    fields.insert("viable".into(), LxVal::Bool(viable));
    fields.insert("reason".into(), reason);
    LxVal::Record(Arc::new(fields))
}

fn extract_eval_fields(val: &LxVal, span: Span) -> Result<(bool, i64, LxVal), LxError> {
    match val {
        LxVal::Record(fields) => {
            let viable = fields
                .get("viable")
                .and_then(|v| {
                    if let LxVal::Bool(b) = v {
                        Some(*b)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    LxError::type_err(
                        "meta: evaluate must return record with Bool 'viable' field",
                        span,
                    )
                })?;
            let quality = fields
                .get("quality")
                .and_then(|v| v.as_int())
                .and_then(|n| n.to_i64())
                .ok_or_else(|| {
                    LxError::type_err(
                        "meta: evaluate must return record with Int 'quality' field",
                        span,
                    )
                })?;
            let reason = fields
                .get("reason")
                .cloned()
                .unwrap_or(LxVal::Str(Arc::from("")));
            Ok((viable, quality, reason))
        }
        _ => Err(LxError::type_err(
            "meta: evaluate must return a record with 'viable', 'quality', 'reason' fields",
            span,
        )),
    }
}

impl Interpreter {
    pub(super) async fn eval_meta(
        &mut self,
        args: &MetaArgs<'_>,
        span: Span,
    ) -> Result<LxVal, LxError> {
        let task_val = self.eval(args.task).await?;
        let strategies_val = self.eval(args.strategies).await?;
        let attempt_fn = self.eval(args.attempt).await?;
        let evaluate_fn = self.eval(args.evaluate).await?;
        let select_val = match args.select {
            Some(e) => Some(self.eval(e).await?),
            None => None,
        };
        let on_switch_fn = match args.on_switch {
            Some(e) => Some(self.eval(e).await?),
            None => None,
        };

        let strategy_list = match &strategies_val {
            LxVal::List(items) => items.as_ref().clone(),
            _ => {
                return Err(LxError::type_err("meta: strategies must be a list", span));
            }
        };

        if strategy_list.is_empty() {
            return Err(LxError::runtime("meta: strategies list is empty", span));
        }

        let order = build_order(&strategy_list, &select_val, span)?;

        let mut attempts = Vec::new();
        let mut best_quality: i64 = i64::MIN;
        let mut best_strategy = strategy_list[0].clone();

        for idx in order {
            let strategy = &strategy_list[idx];

            if let Some(ref cb) = on_switch_fn
                && let Some(prev) = attempts.last()
            {
                let prev_strat = match prev {
                    LxVal::Record(f) => f.get("strategy").cloned().unwrap_or(LxVal::Unit),
                    _ => LxVal::Unit,
                };
                let reason_val = match prev {
                    LxVal::Record(f) => f.get("reason").cloned().unwrap_or(LxVal::Unit),
                    _ => LxVal::Unit,
                };
                let arg = LxVal::Tuple(Arc::new(vec![prev_strat, strategy.clone(), reason_val]));
                crate::builtins::call_value(cb, arg, span, &self.ctx).await?;
            }

            let partial =
                crate::builtins::call_value(&attempt_fn, strategy.clone(), span, &self.ctx).await?;
            let result =
                crate::builtins::call_value(&partial, task_val.clone(), span, &self.ctx).await?;

            let eval_partial =
                crate::builtins::call_value(&evaluate_fn, result.clone(), span, &self.ctx).await?;
            let eval_result =
                crate::builtins::call_value(&eval_partial, strategy.clone(), span, &self.ctx)
                    .await?;

            let (viable, quality, reason) = extract_eval_fields(&eval_result, span)?;
            let attempt_rec = make_attempt_record(strategy, quality, viable, reason);
            attempts.push(attempt_rec);

            if quality > best_quality {
                best_quality = quality;
                best_strategy = strategy.clone();
            }

            if viable {
                return Ok(make_ok_result(result, strategy, &attempts));
            }
        }

        Ok(make_err_result(&attempts, &best_strategy, best_quality))
    }
}

fn select_mode_name(val: &LxVal) -> Option<&str> {
    match val {
        LxVal::Tagged { tag, .. } => Some(tag.as_ref()),
        LxVal::Str(s) => Some(s.as_ref()),
        _ => None,
    }
}

fn build_order(
    strategies: &[LxVal],
    select: &Option<LxVal>,
    span: Span,
) -> Result<Vec<usize>, LxError> {
    let n = strategies.len();
    let mode = select.as_ref().and_then(select_mode_name);
    match mode {
        None | Some("sequential") => Ok((0..n).collect()),
        Some("random") => {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut indices: Vec<usize> = (0..n).collect();
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .hash(&mut hasher);
            let seed = hasher.finish();
            for i in (1..n).rev() {
                let j = ((seed.wrapping_mul(i as u64 + 1)) >> 32) as usize % (i + 1);
                indices.swap(i, j);
            }
            Ok(indices)
        }
        Some(name) => Err(LxError::runtime(
            format!("meta: unknown select mode '{name}'"),
            span,
        )),
    }
}

fn make_ok_result(result: LxVal, strategy: &LxVal, attempts: &[LxVal]) -> LxVal {
    let mut fields = IndexMap::new();
    fields.insert("result".into(), result);
    fields.insert("strategy".into(), strategy.clone());
    fields.insert("attempts".into(), LxVal::List(Arc::new(attempts.to_vec())));
    LxVal::Ok(Box::new(LxVal::Record(Arc::new(fields))))
}

fn make_err_result(attempts: &[LxVal], best_strategy: &LxVal, best_quality: i64) -> LxVal {
    let mut best_fields = IndexMap::new();
    best_fields.insert("strategy".into(), best_strategy.clone());
    best_fields.insert("quality".into(), LxVal::Int(BigInt::from(best_quality)));

    let mut fields = IndexMap::new();
    fields.insert("reason".into(), LxVal::Str(Arc::from("all_exhausted")));
    fields.insert("attempts".into(), LxVal::List(Arc::new(attempts.to_vec())));
    fields.insert("best".into(), LxVal::Record(Arc::new(best_fields)));
    LxVal::Err(Box::new(LxVal::Record(Arc::new(fields))))
}
