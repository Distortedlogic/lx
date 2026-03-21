use std::sync::Arc;

use crate::ast::{BinOp, SExpr, SStmt, SelArm};
use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::Interpreter;

impl Interpreter {
    pub(super) async fn eval_binary(
        &mut self,
        op: &BinOp,
        left: &SExpr,
        right: &SExpr,
        span: Span,
    ) -> Result<LxVal, LxError> {
        if *op == BinOp::And {
            return self.eval_short_circuit(left, right, true, span).await;
        }
        if *op == BinOp::Or {
            return self.eval_short_circuit(left, right, false, span).await;
        }
        let lv = self.eval(left).await?;
        let lv = self.force_defaults(lv, span).await?;
        let rv = self.eval(right).await?;
        let rv = self.force_defaults(rv, span).await?;
        self.binary_op(op, &lv, &rv, span)
    }

    pub(super) async fn eval_block(&mut self, stmts: &[SStmt]) -> Result<LxVal, LxError> {
        let saved = Arc::clone(&self.env);
        self.env = Arc::new(self.env.child());
        let mut result = LxVal::Unit;
        for stmt in stmts {
            result = self.eval_stmt(stmt).await?;
        }
        self.env = saved;
        Ok(result)
    }

    pub(super) async fn eval_loop(&mut self, stmts: &[SStmt]) -> Result<LxVal, LxError> {
        loop {
            let saved = Arc::clone(&self.env);
            self.env = Arc::new(self.env.child());
            for stmt in stmts {
                match self.eval_stmt(stmt).await {
                    Ok(_) => {}
                    Err(LxError::BreakSignal { value }) => {
                        self.env = saved;
                        return Ok(*value);
                    }
                    Err(e) => {
                        self.env = saved;
                        return Err(e);
                    }
                }
            }
            self.env = saved;
        }
    }

    pub(super) async fn eval_slice(
        &mut self,
        expr: &SExpr,
        start: Option<&SExpr>,
        end: Option<&SExpr>,
        span: Span,
    ) -> Result<LxVal, LxError> {
        let val = self.eval(expr).await?;
        let items = match &val {
            LxVal::List(l) => l.as_ref(),
            other => {
                return Err(LxError::type_err(
                    format!("slice requires List, got {}", other.type_name()),
                    span,
                ));
            }
        };
        let len = items.len();
        let s = match start {
            Some(e) => {
                let v = self.eval(e).await?;
                v.as_int().and_then(|n| n.try_into().ok()).ok_or_else(|| {
                    LxError::type_err(
                        format!("slice start index must be Int, got {} `{v}`", v.type_name()),
                        span,
                    )
                })?
            }
            None => 0usize,
        };
        let en: usize = match end {
            Some(e) => {
                let v = self.eval(e).await?;
                v.as_int().and_then(|n| n.try_into().ok()).ok_or_else(|| {
                    LxError::type_err(
                        format!("slice end index must be Int, got {} `{v}`", v.type_name()),
                        span,
                    )
                })?
            }
            None => len,
        };
        let s = s.min(len);
        let en = en.min(len);
        Ok(LxVal::List(Arc::new(items[s..en].to_vec())))
    }

    pub(super) async fn eval_par(&mut self, stmts: &[SStmt]) -> Result<LxVal, LxError> {
        let stmts_owned: Vec<SStmt> = stmts.to_vec();
        let mut futures = Vec::with_capacity(stmts_owned.len());
        for stmt in stmts_owned {
            let env = Arc::clone(&self.env);
            let ctx = Arc::clone(&self.ctx);
            let module_cache = Arc::clone(&self.module_cache);
            let loading = Arc::clone(&self.loading);
            let source = self.source.clone();
            let source_dir = self.source_dir.clone();
            futures.push(async move {
                let mut interp = Interpreter {
                    env,
                    source,
                    source_dir,
                    module_cache,
                    loading,
                    ctx,
                };
                interp.eval_stmt(&stmt).await
            });
        }
        let results = futures::future::join_all(futures).await;
        let mut vals = Vec::with_capacity(results.len());
        for r in results {
            vals.push(r?);
        }
        Ok(LxVal::Tuple(Arc::new(vals)))
    }

    pub(super) async fn eval_sel(&mut self, arms: &[SelArm], span: Span) -> Result<LxVal, LxError> {
        if arms.is_empty() {
            return Err(LxError::runtime("sel: no arms", span));
        }
        let arms_owned: Vec<SelArm> = arms.to_vec();
        let mut futures = Vec::with_capacity(arms_owned.len());
        for arm in &arms_owned {
            let env = Arc::clone(&self.env);
            let ctx = Arc::clone(&self.ctx);
            let module_cache = Arc::clone(&self.module_cache);
            let loading = Arc::clone(&self.loading);
            let source = self.source.clone();
            let source_dir = self.source_dir.clone();
            let arm_expr = arm.expr.clone();
            let arm_handler = arm.handler.clone();
            futures.push(Box::pin(async move {
                let mut interp = Interpreter {
                    env,
                    source,
                    source_dir,
                    module_cache,
                    loading,
                    ctx,
                };
                let val = interp.eval(&arm_expr).await;
                match val {
                    Ok(v) => {
                        let saved = Arc::clone(&interp.env);
                        let mut child = interp.env.child();
                        child.bind("it".into(), v);
                        interp.env = Arc::new(child);
                        let r = interp.eval(&arm_handler).await;
                        interp.env = saved;
                        r
                    }
                    Err(e) => Err(e),
                }
            }));
        }
        let (result, _idx, _remaining) = futures::future::select_all(futures).await;
        result
    }

    pub(super) async fn eval_with_resource(
        &mut self,
        resources: &[(SExpr, String)],
        body: &[SStmt],
        span: Span,
    ) -> Result<LxVal, LxError> {
        let mut acquired: Vec<(String, LxVal)> = Vec::new();
        for (expr, name) in resources {
            match self.eval(expr).await {
                Ok(val) => acquired.push((name.clone(), val)),
                Err(e) => {
                    for (_, val) in acquired.iter().rev() {
                        self.close_resource(val, span).await;
                    }
                    return Err(e);
                }
            }
        }
        let saved = Arc::clone(&self.env);
        let mut child = self.env.child();
        for (name, val) in &acquired {
            child.bind(name.clone(), val.clone());
        }
        self.env = child.into_arc();
        let mut result = LxVal::Unit;
        let mut body_err = None;
        for stmt in body {
            match self.eval_stmt(stmt).await {
                Ok(v) => result = v,
                Err(e) => {
                    body_err = Some(e);
                    break;
                }
            }
        }
        self.env = saved;
        for (_, val) in acquired.iter().rev() {
            self.close_resource(val, span).await;
        }
        match body_err {
            Some(e) => Err(e),
            None => Ok(result),
        }
    }

    pub(super) async fn eval_assert(
        &mut self,
        expr: &SExpr,
        msg: &Option<Box<SExpr>>,
        span: Span,
    ) -> Result<LxVal, LxError> {
        let val = self.eval(expr).await?;
        let val = self.force_defaults(val, span).await?;
        match val.as_bool() {
            Some(true) => Ok(LxVal::Unit),
            Some(false) => {
                let message = match msg {
                    Some(m) => {
                        let mv = self.eval(m).await?;
                        Some(format!("{mv}"))
                    }
                    None => None,
                };
                Err(LxError::assert_fail(
                    format!("{:?}", expr.node),
                    message,
                    span,
                ))
            }
            _ => Err(LxError::type_err(
                format!(
                    "assert requires Bool, got {} `{}`",
                    val.type_name(),
                    val.short_display()
                ),
                span,
            )),
        }
    }
}
