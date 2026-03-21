mod agents;
mod agents_mcp;
pub(crate) mod ambient;
mod apply;
mod apply_helpers;
mod collections;
mod eval;
mod eval_ops;
mod exec_stmt;
mod hints;
mod meta;
mod modules;
mod patterns;
mod receive;
mod refine;
mod shell;
mod traits;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use async_recursion::async_recursion;
use parking_lot::Mutex;

use indexmap::IndexMap;

use crate::ast::{BindTarget, Expr, Program, SExpr, Stmt};
use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::value::LxVal;

#[derive(Debug, Clone)]
pub(crate) struct ModuleExports {
    pub(crate) bindings: IndexMap<String, LxVal>,
    pub(crate) variant_ctors: Vec<String>,
}

pub struct Interpreter {
    pub(crate) env: Arc<Env>,
    source: String,
    pub(crate) source_dir: Option<PathBuf>,
    pub(crate) module_cache: Arc<Mutex<HashMap<PathBuf, ModuleExports>>>,
    pub(crate) loading: Arc<Mutex<HashSet<PathBuf>>>,
    pub(crate) ctx: Arc<RuntimeCtx>,
}

impl Interpreter {
    pub fn new(source: &str, source_dir: Option<PathBuf>, ctx: Arc<RuntimeCtx>) -> Self {
        let mut env = Env::new();
        crate::builtins::register(&mut env);
        *ctx.source_dir.lock() = source_dir.clone();
        Self {
            env: env.into_arc(),
            source: source.to_string(),
            source_dir,
            module_cache: Arc::new(Mutex::new(HashMap::new())),
            loading: Arc::new(Mutex::new(HashSet::new())),
            ctx,
        }
    }

    pub fn with_env(env: &Env, ctx: Arc<RuntimeCtx>) -> Self {
        Self {
            env: Arc::new(env.clone()),
            source: String::new(),
            source_dir: None,
            module_cache: Arc::new(Mutex::new(HashMap::new())),
            loading: Arc::new(Mutex::new(HashSet::new())),
            ctx,
        }
    }

    pub fn set_env(&mut self, env: Env) {
        self.env = env.into_arc();
    }

    pub async fn eval_expr(&mut self, expr: &SExpr) -> Result<LxVal, LxError> {
        self.eval(expr).await
    }

    pub async fn exec(&mut self, program: &Program) -> Result<LxVal, LxError> {
        let mut forward_names = Vec::new();
        for stmt in &program.stmts {
            if let Stmt::Binding(b) = &stmt.node
                && !b.exported
                && let BindTarget::Name(ref name) = b.target
                && matches!(b.value.node, Expr::Func { .. })
            {
                forward_names.push(name.clone());
            }
        }
        if !forward_names.is_empty() {
            let mut env = self.env.child();
            for name in &forward_names {
                env.bind_mut(name.clone(), LxVal::Unit);
            }
            self.env = env.into_arc();
        }
        let mut result = LxVal::Unit;
        for stmt in &program.stmts {
            result = self.eval_stmt(stmt).await?;
        }
        Ok(result)
    }

    #[async_recursion(?Send)]
    pub(crate) async fn eval(&mut self, expr: &SExpr) -> Result<LxVal, LxError> {
        let span = expr.span;
        match &expr.node {
            Expr::Literal(lit) => self.eval_literal(lit, span).await,
            Expr::Ident(name) => self.env.get(name).ok_or_else(|| {
                let hint = hints::keyword_hint(name);
                let msg = match hint {
                    Some(h) => format!("undefined variable '{name}' — {h}"),
                    None => format!("undefined variable '{name}'"),
                };
                LxError::runtime(msg, span)
            }),
            Expr::TypeConstructor(name) => self
                .env
                .get(name)
                .ok_or_else(|| LxError::runtime(format!("undefined constructor '{name}'"), span)),
            Expr::Binary { op, left, right } => self.eval_binary(op, left, right, span).await,
            Expr::Unary { op, operand } => self.eval_unary(op, operand, span).await,
            Expr::Pipe { left, right } => self.eval_pipe(left, right, span).await,
            Expr::Apply { func, arg } => {
                let f = self.eval(func).await?;
                if let Expr::NamedArg { name, value } = &arg.node {
                    let v = self.eval(value).await?;
                    let named = LxVal::Tagged {
                        tag: Arc::from("__named"),
                        values: Arc::new(vec![LxVal::Str(Arc::from(name.as_str())), v]),
                    };
                    self.apply_func(f, named, span).await
                } else {
                    let a = self.eval(arg).await?;
                    self.apply_func(f, a, span).await
                }
            }
            Expr::Section(sec) => self.eval_section(sec, span),
            Expr::FieldAccess { expr: e, field } => self.eval_field_access(e, field, span).await,
            Expr::Block(stmts) => self.eval_block(stmts).await,
            Expr::Tuple(elems) => self.eval_tuple(elems).await,
            Expr::List(elems) => self.eval_list(elems).await,
            Expr::Record(fields) => self.eval_record(fields).await,
            Expr::Map(entries) => self.eval_map(entries).await,
            Expr::Func { params, body, .. } => self.eval_func(params, body).await,
            Expr::Match { scrutinee, arms } => self.eval_match(scrutinee, arms, span).await,
            Expr::Ternary { cond, then_, else_ } => {
                self.eval_ternary(cond, then_, else_, span).await
            }
            Expr::Assert { expr: e, msg } => self.eval_assert(e, msg, span).await,
            Expr::Propagate(inner) => {
                let v = self.eval(inner).await?;
                match v {
                    LxVal::Ok(v) => Ok(*v),
                    LxVal::Err(_) => Err(LxError::propagate(v, span)),
                    LxVal::Some(v) => Ok(*v),
                    LxVal::None => Err(LxError::propagate(
                        LxVal::Err(Box::new(LxVal::Str(Arc::from("unwrapped None")))),
                        span,
                    )),
                    other => Err(LxError::type_err(
                        format!("^ expects Result or Maybe, got {}", other.type_name()),
                        span,
                    )),
                }
            }
            Expr::Coalesce { expr: e, default } => {
                let v = self.eval(e).await?;
                match v {
                    LxVal::Ok(inner) | LxVal::Some(inner) => Ok(*inner),
                    LxVal::Err(_) | LxVal::None => self.eval(default).await,
                    other => Ok(other),
                }
            }
            Expr::Slice {
                expr: e,
                start: s,
                end: en,
            } => self.eval_slice(e, s.as_deref(), en.as_deref(), span).await,
            Expr::NamedArg { name, value } => {
                let _ = name;
                self.eval(value).await
            }
            Expr::Loop(stmts) => self.eval_loop(stmts).await,
            Expr::Break(val) => {
                let v = match val {
                    Some(e) => self.eval(e).await?,
                    None => LxVal::Unit,
                };
                Err(LxError::break_signal(v))
            }
            Expr::Par(stmts) => self.eval_par(stmts).await,
            Expr::Sel(arms) => self.eval_sel(arms, span).await,
            Expr::AgentSend { target, msg } => self.eval_agent_send(target, msg, span).await,
            Expr::AgentAsk { target, msg } => self.eval_agent_ask(target, msg, span).await,
            Expr::StreamAsk { target, msg } => self.eval_stream_ask(target, msg, span).await,
            Expr::Emit { value } => {
                let v = self.eval(value).await?;
                self.ctx.emit.emit(&v, span)?;
                Ok(LxVal::Unit)
            }
            Expr::Yield { value } => {
                let v = self.eval(value).await?;
                self.ctx.yield_.yield_value(v, span)
            }
            Expr::With {
                name,
                value,
                body,
                mutable,
            } => {
                let val = self.eval(value).await?;
                let saved = Arc::clone(&self.env);
                let mut child = self.env.child();
                if *mutable {
                    child.bind_mut(name.clone(), val);
                } else {
                    child.bind(name.clone(), val);
                }
                self.env = child.into_arc();
                let mut result = LxVal::Unit;
                for stmt in body {
                    result = self.eval_stmt(stmt).await?;
                }
                self.env = saved;
                Ok(result)
            }
            Expr::WithResource { resources, body } => {
                self.eval_with_resource(resources, body, span).await
            }
            Expr::WithContext { fields, body } => self.eval_with_context(fields, body, span).await,
            Expr::Refine {
                initial,
                grade,
                revise,
                threshold,
                max_rounds,
                on_round,
            } => {
                self.eval_refine(
                    &refine::RefineArgs {
                        initial,
                        grade,
                        revise,
                        threshold,
                        max_rounds,
                        on_round: on_round.as_deref(),
                    },
                    span,
                )
                .await
            }
            Expr::Meta {
                task,
                strategies,
                attempt,
                evaluate,
                select,
                on_switch,
            } => {
                self.eval_meta(
                    &meta::MetaArgs {
                        task,
                        strategies,
                        attempt,
                        evaluate,
                        select: select.as_deref(),
                        on_switch: on_switch.as_deref(),
                    },
                    span,
                )
                .await
            }
            Expr::Shell { mode, parts } => self.eval_shell(mode, parts, span).await,
            Expr::Receive(arms) => self.eval_receive(arms, span).await,
        }
    }
}
