use crate::sym::{Sym, intern};
pub(crate) mod ambient;
mod apply;
mod apply_helpers;
mod collections;
mod default_tools;
mod eval;
mod eval_ops;
mod exec_stmt;
mod hints;
mod modules;
mod patterns;
mod trait_apply;
mod traits;
mod type_apply;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use async_recursion::async_recursion;
use parking_lot::Mutex;

use indexmap::IndexMap;

use crate::ast::{
  AstArena, BindTarget, Core, Expr, ExprApply, ExprAssert, ExprBinary, ExprBlock, ExprBreak, ExprEmit, ExprFieldAccess, ExprFunc, ExprId, ExprLoop, ExprMatch,
  ExprNamedArg, ExprPar, ExprPropagate, ExprSlice, ExprTimeout, ExprTuple, ExprUnary, ExprWith, ExprYield, Program, Stmt, WithKind,
};
use crate::env::Env;
use crate::error::{EvalResult, EvalSignal, LxError};
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;

#[derive(Debug, Clone)]
pub(crate) struct ModuleExports {
  pub(crate) bindings: IndexMap<Sym, LxVal>,
  pub(crate) variant_ctors: Vec<Sym>,
}

pub struct Interpreter {
  pub(crate) env: Arc<Env>,
  source: String,
  pub(crate) source_dir: Option<PathBuf>,
  pub(crate) module_cache: Arc<Mutex<HashMap<PathBuf, ModuleExports>>>,
  pub(crate) loading: Arc<Mutex<HashSet<PathBuf>>>,
  pub(crate) ctx: Arc<RuntimeCtx>,
  pub(crate) arena: Arc<AstArena>,
  pub(crate) tool_modules: Vec<Arc<crate::tool_module::ToolModule>>,
  pub(crate) agent_name: Option<String>,
  pub(crate) agent_mailbox_rx: Option<Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<crate::runtime::agent_registry::AgentMessage>>>>,
  pub(crate) agent_handle_fn: Option<LxVal>,
}

impl Interpreter {
  pub fn new(source: &str, source_dir: Option<PathBuf>, ctx: Arc<RuntimeCtx>) -> Self {
    let env = Env::default();
    crate::builtins::register(&env);
    *ctx.source_dir.lock() = source_dir.clone();

    if !ctx.event_stream.has_jsonl()
      && let Some(ref dir) = source_dir
    {
      let jsonl_path = dir.join(".lx").join("stream.jsonl");
      ctx.event_stream.enable_jsonl(jsonl_path);
    }

    Self {
      env: Arc::new(env),
      source: source.to_string(),
      source_dir,
      module_cache: Arc::new(Mutex::new(HashMap::new())),
      loading: Arc::new(Mutex::new(HashSet::new())),
      ctx,
      arena: Arc::new(AstArena::new()),
      tool_modules: vec![],
      agent_name: None,
      agent_mailbox_rx: None,
      agent_handle_fn: None,
    }
  }

  pub fn with_env(env: &Env, arena: Arc<AstArena>, ctx: Arc<RuntimeCtx>) -> Self {
    Self {
      env: Arc::new(env.clone()),
      source: String::new(),
      source_dir: None,
      module_cache: Arc::new(Mutex::new(HashMap::new())),
      loading: Arc::new(Mutex::new(HashSet::new())),
      ctx,
      arena,
      tool_modules: vec![],
      agent_name: None,
      agent_mailbox_rx: None,
      agent_handle_fn: None,
    }
  }

  pub fn set_env(&mut self, env: Env) {
    self.env = Arc::new(env);
  }

  pub async fn eval_expr(&mut self, eid: ExprId) -> Result<LxVal, LxError> {
    let span = self.arena.expr_span(eid);
    self.eval(eid).await.map_err(|e| match e {
      EvalSignal::Error(e) => e,
      EvalSignal::Break(_) => LxError::runtime("break outside loop", span),
      EvalSignal::AgentStop => LxError::runtime("agent stopped", span),
    })
  }

  pub async fn exec(&mut self, program: &Program<Core>) -> Result<LxVal, LxError> {
    self.arena = Arc::new(program.arena.clone());
    let mut forward_names = Vec::new();
    for &sid in &program.stmts {
      if let Stmt::Binding(b) = self.arena.stmt(sid)
        && let BindTarget::Name(name) = b.target
        && matches!(self.arena.expr(b.value), Expr::Func(_))
      {
        forward_names.push(name);
      }
    }
    if !forward_names.is_empty() {
      let env = self.env.child();
      for name in &forward_names {
        env.bind_mut(*name, LxVal::Unit);
      }
      self.env = Arc::new(env);
    }
    let mut result = LxVal::Unit;
    let stmts = program.stmts.clone();
    for sid in &stmts {
      result = self.eval_stmt(*sid).await.map_err(|e| match e {
        EvalSignal::Error(e) => e,
        EvalSignal::Break(_) => LxError::runtime("break outside loop", self.arena.stmt_span(*sid)),
        EvalSignal::AgentStop => LxError::runtime("agent stopped", self.arena.stmt_span(*sid)),
      })?;
    }
    for tm in &self.tool_modules {
      tm.shutdown().await;
    }
    Ok(result)
  }

  #[async_recursion(?Send)]
  pub(crate) async fn eval(&mut self, eid: ExprId) -> EvalResult<LxVal> {
    let span = self.arena.expr_span(eid);
    if let (Some(rx_arc), Some(handle_fn)) = (&self.agent_mailbox_rx, &self.agent_handle_fn) {
      let mut rx = rx_arc.lock().await;
      while let Ok(msg) = rx.try_recv() {
        let result = crate::builtins::call_value(handle_fn, msg.payload.clone(), span, &self.ctx).await.unwrap_or_else(|e| LxVal::err_str(e.to_string()));
        if let Some(reply) = msg.reply {
          let _ = reply.send(result);
        }
      }
      drop(rx);
    }
    let expr = self.arena.expr(eid).clone();
    match expr {
      Expr::Literal(ref lit) => self.eval_literal(lit, span).await,
      Expr::Ident(name) => Ok(self.env.get(name).ok_or_else(|| {
        let hint = hints::keyword_hint(name.as_str());
        let msg = match hint {
          Some(h) => format!("undefined variable '{name}' — {h}"),
          None => format!("undefined variable '{name}'"),
        };
        LxError::runtime(msg, span)
      })?),
      Expr::TypeConstructor(name) => {
        if let Some(val) = self.env.get(name) {
          Ok(val)
        } else {
          match name.as_str() {
            "Str" | "Int" | "Float" | "Bool" | "List" | "Record" | "Map" | "Tuple" => Ok(LxVal::Type(name)),
            _ => Err(LxError::runtime(format!("undefined constructor '{name}'"), span).into()),
          }
        }
      },
      Expr::Binary(ExprBinary { op, left, right }) => self.eval_binary(&op, left, right, span).await,
      Expr::Unary(ExprUnary { op, operand }) => self.eval_unary(&op, operand, span).await,
      Expr::Pipe(_) | Expr::Tell(_) | Expr::Ask(_) => unreachable!(),
      Expr::Apply(ExprApply { func, arg }) => {
        let f = self.eval(func).await?;
        if let Expr::NamedArg(ExprNamedArg { name, value }) = self.arena.expr(arg) {
          let name = *name;
          let value = *value;
          let v = self.eval(value).await?;
          let named = LxVal::Tagged { tag: intern("__named"), values: Arc::new(vec![LxVal::str(name.as_str()), v]) };
          self.apply_func(f, named, span).await
        } else {
          let a = self.eval(arg).await?;
          self.apply_func(f, a, span).await
        }
      },
      Expr::Section(_) => unreachable!(),
      Expr::FieldAccess(ExprFieldAccess { expr: e, ref field }) => self.eval_field_access(e, field, span).await,
      Expr::Block(ExprBlock { ref stmts }) => self.eval_block(stmts).await,
      Expr::Tuple(ExprTuple { ref elems }) => self.eval_tuple(elems).await,
      Expr::List(ref elems) => self.eval_list(elems).await,
      Expr::Record(ref fields) => self.eval_record(fields).await,
      Expr::Map(ref entries) => self.eval_map(entries).await,
      Expr::Func(ExprFunc { ref params, guard, body, .. }) => self.eval_func(params, guard, body).await,
      Expr::Match(ExprMatch { scrutinee, ref arms }) => self.eval_match(scrutinee, arms, span).await,
      Expr::Ternary(_) => unreachable!(),
      Expr::Assert(ExprAssert { expr: e, msg }) => self.eval_assert(e, msg, span).await,
      Expr::Propagate(ExprPropagate { inner }) => {
        let v = self.eval(inner).await?;
        match v {
          LxVal::Ok(v) => Ok(*v),
          LxVal::Err(_) => Err(LxError::propagate(v, span).into()),
          LxVal::Some(v) => Ok(*v),
          LxVal::None => Err(LxError::propagate(LxVal::err_str("unwrapped None"), span).into()),
          other => Err(LxError::type_err(format!("^ expects Result or Maybe, got {}", other.type_name()), span, None).into()),
        }
      },
      Expr::Coalesce(_) => unreachable!(),
      Expr::Slice(ExprSlice { expr: e, start: s, end: en }) => self.eval_slice(e, s, en, span).await,
      Expr::NamedArg(ExprNamedArg { value, .. }) => self.eval(value).await,
      Expr::Loop(ExprLoop { ref stmts }) => self.eval_loop(stmts).await,
      Expr::Break(ExprBreak { value: val }) => {
        let v = match val {
          Some(e) => self.eval(e).await?,
          None => LxVal::Unit,
        };
        Err(EvalSignal::Break(v))
      },
      Expr::Par(ExprPar { ref stmts }) => self.eval_par(stmts).await,
      Expr::Sel(ref arms) => self.eval_sel(arms, span).await,
      Expr::Timeout(ExprTimeout { ms, body }) => self.eval_timeout(ms, body, span).await,
      Expr::Spawn(class_expr) => {
        let class_val = self.eval(class_expr).await?;
        let result = crate::builtins::agent::bi_agent_spawn(vec![class_val], span, Arc::clone(&self.ctx)).await;
        result.map_err(EvalSignal::Error)
      },
      Expr::Stop => {
        let name = self.agent_name.as_ref().ok_or_else(|| LxError::runtime("stop: not inside an agent", span))?;
        crate::runtime::agent_registry::remove_agent(name);
        let mut fields = IndexMap::new();
        fields.insert(intern("agent"), LxVal::str(name));
        self.ctx.event_stream.xadd("agent/kill", name, None, fields);
        Err(EvalSignal::AgentStop)
      },
      Expr::Emit(ExprEmit { value }) => {
        let v = self.eval(value).await?;
        println!("{v}");
        let mut fields = indexmap::IndexMap::new();
        fields.insert(crate::sym::intern("value"), v);
        self.ctx.event_stream.xadd("runtime/emit", "main", None, fields);
        Ok(LxVal::Unit)
      },
      Expr::Yield(ExprYield { value }) => {
        let v = self.eval(value).await?;
        Ok(self.ctx.yield_.yield_value(v, span)?)
      },
      Expr::Grouped(inner) => self.eval(inner).await,
      Expr::With(ExprWith { ref kind, ref body }) => match kind.clone() {
        WithKind::Binding { .. } => unreachable!(),
        WithKind::Resources { resources } => {
          let body = body.clone();
          self.eval_with_resource(&resources, &body, span).await
        },
        WithKind::Context { fields } => {
          let body = body.clone();
          self.eval_with_context(&fields, &body, span).await
        },
      },
    }
  }
}
