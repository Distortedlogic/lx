use std::sync::Arc;
use std::time::Duration;

use indexmap::IndexMap;
use num_traits::ToPrimitive;

use crate::ast::{BinOp, ExprId, SelArm, StmtId};
use crate::error::LxError;
use crate::sym::{Sym, intern};
use crate::value::LxVal;
use miette::SourceSpan;

use super::Interpreter;

impl Interpreter {
  pub(super) async fn eval_binary(&mut self, op: &BinOp, left: ExprId, right: ExprId, span: SourceSpan) -> Result<LxVal, LxError> {
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

  pub(super) async fn eval_block(&mut self, stmts: &[StmtId]) -> Result<LxVal, LxError> {
    let saved = Arc::clone(&self.env);
    self.env = Arc::new(self.env.child());
    let stmts = stmts.to_vec();
    let mut result = LxVal::Unit;
    for sid in &stmts {
      result = self.eval_stmt(*sid).await?;
    }
    self.env = saved;
    Ok(result)
  }

  pub(super) async fn eval_loop(&mut self, stmts: &[StmtId]) -> Result<LxVal, LxError> {
    let stmts = stmts.to_vec();
    loop {
      tokio::task::yield_now().await;
      let saved = Arc::clone(&self.env);
      self.env = Arc::new(self.env.child());
      for sid in &stmts {
        match self.eval_stmt(*sid).await {
          Ok(_) => {},
          Err(LxError::BreakSignal { value }) => {
            self.env = saved;
            return Ok(*value);
          },
          Err(e) => {
            self.env = saved;
            return Err(e);
          },
        }
      }
      self.env = saved;
    }
  }

  pub(super) async fn eval_slice(&mut self, expr: ExprId, start: Option<ExprId>, end: Option<ExprId>, span: SourceSpan) -> Result<LxVal, LxError> {
    let val = self.eval(expr).await?;
    let items = match &val {
      LxVal::List(l) => l.as_ref(),
      other => {
        return Err(LxError::type_err(format!("slice requires List, got {}", other.type_name()), span, None));
      },
    };
    let len = items.len();
    let s = match start {
      Some(e) => {
        let v = self.eval(e).await?;
        v.as_int()
          .and_then(|n| n.try_into().ok())
          .ok_or_else(|| LxError::type_err(format!("slice start index must be Int, got {} `{v}`", v.type_name()), span, None))?
      },
      None => 0usize,
    };
    let en: usize = match end {
      Some(e) => {
        let v = self.eval(e).await?;
        v.as_int()
          .and_then(|n| n.try_into().ok())
          .ok_or_else(|| LxError::type_err(format!("slice end index must be Int, got {} `{v}`", v.type_name()), span, None))?
      },
      None => len,
    };
    let s = s.min(len);
    let en = en.min(len);
    Ok(LxVal::list(items[s..en].to_vec()))
  }

  pub(super) async fn eval_par(&mut self, stmts: &[StmtId]) -> Result<LxVal, LxError> {
    let stmts_owned: Vec<StmtId> = stmts.to_vec();
    let mut futures = Vec::with_capacity(stmts_owned.len());
    for sid in stmts_owned {
      let env = Arc::clone(&self.env);
      let ctx = Arc::clone(&self.ctx);
      let module_cache = Arc::clone(&self.module_cache);
      let loading = Arc::clone(&self.loading);
      let source = self.source.clone();
      let source_dir = self.source_dir.clone();
      let arena = Arc::clone(&self.arena);
      futures.push(async move {
        let mut interp = Interpreter { env, source, source_dir, module_cache, loading, ctx, arena };
        interp.eval_stmt(sid).await
      });
    }
    let results = futures::future::join_all(futures).await;
    let vals: Result<Vec<_>, _> = results.into_iter().collect();
    Ok(LxVal::tuple(vals?))
  }

  pub(super) async fn eval_sel(&mut self, arms: &[SelArm], span: SourceSpan) -> Result<LxVal, LxError> {
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
      let arena = Arc::clone(&self.arena);
      let arm_expr = arm.expr;
      let arm_handler = arm.handler;
      futures.push(Box::pin(async move {
        let mut interp = Interpreter { env, source, source_dir, module_cache, loading, ctx, arena };
        let v = interp.eval(arm_expr).await?;
        let saved = Arc::clone(&interp.env);
        let child = interp.env.child();
        child.bind_str("it", v);
        interp.env = Arc::new(child);
        let r = interp.eval(arm_handler).await;
        interp.env = saved;
        r
      }));
    }
    let (result, _idx, _remaining) = futures::future::select_all(futures).await;
    result
  }

  pub(super) async fn eval_with_resource(&mut self, resources: &[(ExprId, Sym)], body: &[StmtId], span: SourceSpan) -> Result<LxVal, LxError> {
    let mut acquired: Vec<(Sym, LxVal)> = Vec::new();
    for &(expr, name) in resources {
      match self.eval(expr).await {
        Ok(val) => acquired.push((name, val)),
        Err(e) => {
          for (_, val) in acquired.iter().rev() {
            self.close_resource(val, span).await;
          }
          return Err(e);
        },
      }
    }
    let saved = Arc::clone(&self.env);
    let child = self.env.child();
    for (name, val) in &acquired {
      child.bind(*name, val.clone());
    }
    self.env = Arc::new(child);
    let mut result = LxVal::Unit;
    let mut body_err = None;
    for &sid in body {
      match self.eval_stmt(sid).await {
        Ok(v) => result = v,
        Err(e) => {
          body_err = Some(e);
          break;
        },
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

  pub(super) async fn eval_timeout(&mut self, ms_expr: ExprId, body: ExprId, span: SourceSpan) -> Result<LxVal, LxError> {
    let ms_val = self.eval(ms_expr).await?;
    let ms_u64 = match &ms_val {
      LxVal::Int(n) => n.to_u64().ok_or_else(|| LxError::runtime("timeout: ms must be non-negative integer", span))?,
      LxVal::Float(f) => *f as u64,
      other => return Err(LxError::type_err(format!("timeout expects Int or Float for ms, got {}", other.type_name()), span, None)),
    };
    let body_eid = body;
    let env = Arc::clone(&self.env);
    let ctx = Arc::clone(&self.ctx);
    let module_cache = Arc::clone(&self.module_cache);
    let loading = Arc::clone(&self.loading);
    let source = self.source.clone();
    let source_dir = self.source_dir.clone();
    let arena = Arc::clone(&self.arena);
    let body_fut = async move {
      let mut interp = Interpreter { env, source, source_dir, module_cache, loading, ctx, arena };
      interp.eval(body_eid).await
    };
    tokio::select! {
        result = body_fut => {
            Ok(LxVal::ok(result?))
        }
        _ = tokio::time::sleep(Duration::from_millis(ms_u64)) => {
            let mut fields = IndexMap::new();
            fields.insert(intern("kind"), LxVal::str(":timeout"));
            fields.insert(intern("ms"), LxVal::Int(ms_u64.into()));
            Ok(LxVal::err(LxVal::record(fields)))
        }
    }
  }

  pub(super) async fn eval_assert(&mut self, expr: ExprId, msg: Option<ExprId>, span: SourceSpan) -> Result<LxVal, LxError> {
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
        let expr_node = self.arena.expr(expr);
        Err(LxError::assert_fail(format!("{expr_node:?}"), message, span))
      },
      _ => Err(LxError::type_err(format!("assert requires Bool, got {} `{}`", val.type_name(), val.short_display()), span, None)),
    }
  }
}
