use std::sync::Arc;
use std::time::Duration;

use indexmap::IndexMap;
use num_traits::ToPrimitive;

use crate::ast::{BinOp, ExprId, SelArm, StmtId};
use crate::error::{EvalResult, EvalSignal, LxError};
use crate::sym::{Sym, intern};
use crate::value::LxVal;
use miette::SourceSpan;

use super::Interpreter;

impl Interpreter {
  pub(super) async fn eval_binary(&mut self, op: &BinOp, left: ExprId, right: ExprId, span: SourceSpan) -> EvalResult<LxVal> {
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
    Ok(self.binary_op(op, &lv, &rv, span)?)
  }

  pub(super) async fn eval_block(&mut self, stmts: &[StmtId]) -> EvalResult<LxVal> {
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

  pub(super) async fn eval_loop(&mut self, stmts: &[StmtId]) -> EvalResult<LxVal> {
    let stmts = stmts.to_vec();
    loop {
      tokio::task::yield_now().await;
      let saved = Arc::clone(&self.env);
      self.env = Arc::new(self.env.child());
      for sid in &stmts {
        match self.eval_stmt(*sid).await {
          Ok(_) => {},
          Err(EvalSignal::Break(value)) => {
            self.env = saved;
            return Ok(value);
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

  pub(super) async fn eval_slice(&mut self, expr: ExprId, start: Option<ExprId>, end: Option<ExprId>, span: SourceSpan) -> EvalResult<LxVal> {
    let val = self.eval(expr).await?;
    let items = match &val {
      LxVal::List(l) => l.as_ref(),
      other => {
        return Err(LxError::type_err(format!("slice requires List, got {}", other.type_name()), span, None).into());
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

  pub(super) async fn eval_par(&mut self, stmts: &[StmtId]) -> EvalResult<LxVal> {
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
        let mut interp = Interpreter {
          env,
          source,
          source_dir,
          module_cache,
          loading,
          ctx,
          arena,
          tool_modules: vec![],
          agent_name: None,
          agent_mailbox_rx: None,
          agent_handle_fn: None,
          next_ask_id: std::sync::atomic::AtomicU64::new(1),
        };
        interp.eval_stmt(sid).await
      });
    }
    let results = futures::future::join_all(futures).await;
    let vals: Result<Vec<_>, _> = results.into_iter().collect();
    Ok(LxVal::tuple(vals?))
  }

  pub(super) async fn eval_sel(&mut self, arms: &[SelArm], span: SourceSpan) -> EvalResult<LxVal> {
    if arms.is_empty() {
      return Err(LxError::runtime("sel: no arms", span).into());
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
        let mut interp = Interpreter {
          env,
          source,
          source_dir,
          module_cache,
          loading,
          ctx,
          arena,
          tool_modules: vec![],
          agent_name: None,
          agent_mailbox_rx: None,
          agent_handle_fn: None,
          next_ask_id: std::sync::atomic::AtomicU64::new(1),
        };
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

  pub(super) async fn eval_with_resource(&mut self, resources: &[(ExprId, Sym)], body: &[StmtId], span: SourceSpan) -> EvalResult<LxVal> {
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

  pub(super) async fn eval_timeout(&mut self, ms_expr: ExprId, body: ExprId, span: SourceSpan) -> EvalResult<LxVal> {
    let ms_val = self.eval(ms_expr).await?;
    let ms_u64 = match &ms_val {
      LxVal::Int(n) => n.to_u64().ok_or_else(|| LxError::runtime("timeout: ms must be non-negative integer", span))?,
      LxVal::Float(f) => *f as u64,
      other => return Err(LxError::type_err(format!("timeout expects Int or Float for ms, got {}", other.type_name()), span, None).into()),
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
      let mut interp = Interpreter {
        env,
        source,
        source_dir,
        module_cache,
        loading,
        ctx,
        arena,
        tool_modules: vec![],
        agent_name: None,
        agent_mailbox_rx: None,
        agent_handle_fn: None,
        next_ask_id: std::sync::atomic::AtomicU64::new(1),
      };
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
}
