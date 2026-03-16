mod agents;
mod apply;
mod collections;
mod eval;
mod modules;
mod patterns;
mod refine;
mod shell;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;

use indexmap::IndexMap;

use crate::ast::{BindTarget, Expr, Program, SExpr, SStmt, Stmt};
use crate::backends::RuntimeCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::value::Value;

#[derive(Debug, Clone)]
pub(crate) struct ModuleExports {
    pub(crate) bindings: IndexMap<String, Value>,
    pub(crate) variant_ctors: Vec<String>,
}

fn keyword_hint(name: &str) -> Option<&'static str> {
    match name {
        "if" | "else" | "then" | "elif" | "elsif" => {
            Some("lx uses `cond ? then_expr : else_expr` for conditionals")
        }
        "mut" => Some("lx uses `:=` for mutable bindings: `x := 0`"),
        "let" | "var" | "const" => Some("lx bindings use `name = value` (or `name := value` for mutable)"),
        "return" => Some("lx uses implicit returns — last expression in a block is its value"),
        "fn" | "def" | "func" | "function" => {
            Some("lx functions use `name = (params) body` or `name = (params) { body }`")
        }
        "import" | "from" | "require" | "include" => {
            Some("lx uses `use std/module` or `use ./relative/path`")
        }
        "for" | "while" | "loop" => {
            Some("lx uses `each`, `map`, `filter` for iteration, or recursion")
        }
        "match" | "switch" | "case" => {
            Some("lx uses `value ? { pattern -> body }` for pattern matching")
        }
        "print" | "println" | "console" | "echo" | "printf" => {
            Some("lx uses `emit` for output or `log.info`/`log.warn`/`log.err` for logging")
        }
        "try" | "catch" | "throw" | "raise" | "except" => {
            Some("lx uses `^` to propagate errors and `??` to coalesce: `expr ^ | process` or `expr ?? default`")
        }
        "null" | "nil" | "undefined" | "void" => {
            Some("lx uses `None` for absence and `()` for unit")
        }
        "class" | "struct" | "new" | "interface" => {
            Some("lx uses Records `{field: value}` for data, `Protocol` for contracts, `Trait` for behavior")
        }
        "async" | "await" => Some("lx uses `par`, `sel`, `pmap` for concurrency"),
        "self" | "this" => Some("lx has no `self` — use record fields or closures"),
        "break" | "continue" => Some("lx uses recursion or higher-order functions for control flow"),
        "lambda" => Some("lx lambdas use `(params) body` or `(params) { body }`"),
        "not" => Some("lx uses `!` for negation: `!expr`"),
        "and" => Some("lx uses `&&` for logical and"),
        "or" => Some("lx uses `||` for logical or"),
        "in" => Some("lx uses `contains?` for membership"),
        _ => None,
    }
}

fn binding_pattern_hint(pat_str: &str) -> Option<&'static str> {
    let first_word = pat_str.split_whitespace().next().unwrap_or("");
    let trimmed = first_word.trim_matches(|c| c == '(' || c == ')' || c == ',');
    match trimmed {
        "mut" => Some("lx uses `:=` for mutable bindings: `x := 0`"),
        "let" | "var" | "const" => {
            Some("lx bindings use `name = value` (or `name := value` for mutable)")
        }
        _ => None,
    }
}

pub struct Interpreter {
    env: Arc<Env>,
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

    pub fn eval_expr(&mut self, expr: &SExpr) -> Result<Value, LxError> {
        self.eval(expr)
    }

    pub fn exec(&mut self, program: &Program) -> Result<Value, LxError> {
        let mut forward_names = Vec::new();
        for stmt in &program.stmts {
            if let Stmt::Binding(b) = &stmt.node
                && let BindTarget::Name(ref name) = b.target
                && matches!(b.value.node, Expr::Func { .. })
            {
                forward_names.push(name.clone());
            }
        }
        if !forward_names.is_empty() {
            let mut env = self.env.child();
            for name in &forward_names {
                env.bind_mut(name.clone(), Value::Unit);
            }
            self.env = env.into_arc();
        }
        let mut result = Value::Unit;
        for stmt in &program.stmts {
            result = self.eval_stmt(stmt)?;
        }
        Ok(result)
    }

    pub(crate) fn eval(&mut self, expr: &SExpr) -> Result<Value, LxError> {
        let span = expr.span;
        match &expr.node {
            Expr::Literal(lit) => self.eval_literal(lit, span),
            Expr::Ident(name) => self
                .env
                .get(name)
                .ok_or_else(|| {
                    let hint = keyword_hint(name);
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
            Expr::Binary { op, left, right } => self.eval_binary(op, left, right, span),
            Expr::Unary { op, operand } => self.eval_unary(op, operand, span),
            Expr::Pipe { left, right } => self.eval_pipe(left, right, span),
            Expr::Apply { func, arg } => {
                let f = self.eval(func)?;
                if let Expr::NamedArg { name, value } = &arg.node {
                    let v = self.eval(value)?;
                    let named = Value::Tagged {
                        tag: Arc::from("__named"),
                        values: Arc::new(vec![Value::Str(Arc::from(name.as_str())), v]),
                    };
                    self.apply_func(f, named, span)
                } else {
                    let a = self.eval(arg)?;
                    self.apply_func(f, a, span)
                }
            }
            Expr::Section(sec) => self.eval_section(sec, span),
            Expr::FieldAccess { expr: e, field } => self.eval_field_access(e, field, span),
            Expr::Block(stmts) => self.eval_block(stmts),
            Expr::Tuple(elems) => self.eval_tuple(elems),
            Expr::List(elems) => self.eval_list(elems),
            Expr::Record(fields) => self.eval_record(fields),
            Expr::Map(entries) => self.eval_map(entries),
            Expr::Func { params, body, .. } => self.eval_func(params, body),
            Expr::Match { scrutinee, arms } => self.eval_match(scrutinee, arms, span),
            Expr::Ternary { cond, then_, else_ } => self.eval_ternary(cond, then_, else_, span),
            Expr::Assert { expr: e, msg } => self.eval_assert(e, msg, span),
            Expr::Propagate(inner) => {
                let v = self.eval(inner)?;
                match v {
                    Value::Ok(v) => Ok(*v),
                    Value::Err(_) => Err(LxError::propagate(v, span)),
                    Value::Some(v) => Ok(*v),
                    Value::None => Err(LxError::propagate(
                        Value::Err(Box::new(Value::Str(Arc::from("unwrapped None")))),
                        span,
                    )),
                    other => Err(LxError::type_err(
                        format!("^ expects Result or Maybe, got {}", other.type_name()),
                        span,
                    )),
                }
            }
            Expr::Coalesce { expr: e, default } => {
                let v = self.eval(e)?;
                match v {
                    Value::Ok(inner) | Value::Some(inner) => Ok(*inner),
                    Value::Err(_) | Value::None => self.eval(default),
                    other => Ok(other),
                }
            }
            Expr::Slice {
                expr: e,
                start: s,
                end: en,
            } => self.eval_slice(e, s.as_deref(), en.as_deref(), span),
            Expr::NamedArg { name, value } => {
                let _ = name;
                self.eval(value)
            }
            Expr::Loop(stmts) => self.eval_loop(stmts),
            Expr::Break(val) => {
                let v = match val {
                    Some(e) => self.eval(e)?,
                    None => Value::Unit,
                };
                Err(LxError::break_signal(v))
            }
            Expr::Par(stmts) => self.eval_par(stmts),
            Expr::Sel(arms) => self.eval_sel(arms, span),
            Expr::AgentSend { target, msg } => self.eval_agent_send(target, msg, span),
            Expr::AgentAsk { target, msg } => self.eval_agent_ask(target, msg, span),
            Expr::Emit { value } => {
                let v = self.eval(value)?;
                self.ctx.emit.emit(&v, span)?;
                Ok(Value::Unit)
            }
            Expr::Yield { value } => {
                let v = self.eval(value)?;
                self.ctx.yield_.yield_value(v, span)
            }
            Expr::With {
                name,
                value,
                body,
                mutable,
            } => {
                let val = self.eval(value)?;
                let saved = Arc::clone(&self.env);
                let mut child = self.env.child();
                if *mutable {
                    child.bind_mut(name.clone(), val);
                } else {
                    child.bind(name.clone(), val);
                }
                self.env = child.into_arc();
                let mut result = Value::Unit;
                for stmt in body {
                    result = self.eval_stmt(stmt)?;
                }
                self.env = saved;
                Ok(result)
            }
            Expr::WithResource { resources, body } => self.eval_with_resource(resources, body, span),
            Expr::Refine {
                initial,
                grade,
                revise,
                threshold,
                max_rounds,
                on_round,
            } => self.eval_refine(
                &refine::RefineArgs {
                    initial,
                    grade,
                    revise,
                    threshold,
                    max_rounds,
                    on_round: on_round.as_deref(),
                },
                span,
            ),
            Expr::Shell { mode, parts } => self.eval_shell(mode, parts, span),
        }
    }

    pub(crate) fn eval_stmt(&mut self, stmt: &SStmt) -> Result<Value, LxError> {
        match &stmt.node {
            Stmt::Binding(b) => {
                let val = self.eval(&b.value)?;
                let val = self.force_defaults(val, stmt.span)?;
                match &b.target {
                    BindTarget::Name(name) => {
                        if self.env.has_mut(name) {
                            self.env
                                .reassign(name, val)
                                .map_err(|e| LxError::runtime(e, stmt.span))?;
                        } else {
                            let mut env = self.env.child();
                            if b.mutable {
                                env.bind_mut(name.clone(), val);
                            } else {
                                env.bind(name.clone(), val);
                            }
                            self.env = env.into_arc();
                        }
                    }
                    BindTarget::Reassign(name) => {
                        self.env
                            .reassign(name, val)
                            .map_err(|e| LxError::runtime(e, stmt.span))?;
                    }
                    BindTarget::Pattern(pat) => {
                        let bindings =
                            self.try_match_pattern(&pat.node, &val).ok_or_else(|| {
                                let pat_str = format!("{}", pat.node);
                                let hint = binding_pattern_hint(&pat_str);
                                let msg = match hint {
                                    Some(h) => format!(
                                        "cannot bind {} `{}` to pattern `{pat_str}` — {h}",
                                        val.type_name(), val.short_display(),
                                    ),
                                    None => format!(
                                        "cannot bind {} `{}` to pattern `{pat_str}`",
                                        val.type_name(), val.short_display(),
                                    ),
                                };
                                LxError::runtime(msg, stmt.span)
                            })?;
                        let mut env = self.env.child();
                        for (name, v) in bindings {
                            if b.mutable {
                                env.bind_mut(name, v);
                            } else {
                                env.bind(name, v);
                            }
                        }
                        self.env = env.into_arc();
                    }
                }
                Ok(Value::Unit)
            }
            Stmt::Use(use_stmt) => {
                self.eval_use(use_stmt, stmt.span)?;
                Ok(Value::Unit)
            }
            Stmt::TypeDef { variants, .. } => {
                let mut env = self.env.child();
                for (ctor_name, arity) in variants {
                    let tag: Arc<str> = Arc::from(ctor_name.as_str());
                    if *arity == 0 {
                        env.bind(
                            ctor_name.clone(),
                            Value::Tagged {
                                tag,
                                values: Arc::new(vec![]),
                            },
                        );
                    } else {
                        env.bind(
                            ctor_name.clone(),
                            Value::TaggedCtor {
                                tag,
                                arity: *arity,
                                applied: vec![],
                            },
                        );
                    }
                }
                self.env = env.into_arc();
                Ok(Value::Unit)
            }
            Stmt::Protocol {
                name, entries, ..
            } => self.eval_protocol_def(name, entries, stmt.span),
            Stmt::ProtocolUnion(def) => {
                self.eval_protocol_union(&def.name, &def.variants, stmt.span)
            }
            Stmt::McpDecl { name, tools, .. } => self.eval_mcp_decl(name, tools, stmt.span),
            Stmt::TraitDecl {
                name,
                handles,
                provides,
                requires,
                exported,
            } => {
                let val = Value::Trait {
                    name: Arc::from(name.as_str()),
                    handles: Arc::new(handles.iter().map(|s| Arc::from(s.as_str())).collect()),
                    provides: Arc::new(provides.iter().map(|s| Arc::from(s.as_str())).collect()),
                    requires: Arc::new(requires.iter().map(|s| Arc::from(s.as_str())).collect()),
                };
                let _ = exported;
                let mut env = self.env.child();
                env.bind(name.clone(), val);
                self.env = env.into_arc();
                Ok(Value::Unit)
            }
            Stmt::AgentDecl {
                name,
                traits,
                uses: _,
                init,
                on: _,
                methods,
                exported,
            } => {
                let mut method_map = IndexMap::new();
                for m in methods {
                    let handler = self.eval(&m.handler)?;
                    method_map.insert(m.name.clone(), handler);
                }
                let init_val = match init {
                    Some(expr) => Some(Box::new(self.eval(expr)?)),
                    None => None,
                };
                for trait_name in traits {
                    if let Some(Value::Trait { handles, .. }) = self.env.get(trait_name) {
                        for required in handles.iter() {
                            let key = required.to_string();
                            if !method_map.contains_key(&key) {
                                return Err(LxError::runtime(
                                    format!(
                                        "Agent {name} missing method '{key}' required by {trait_name}"
                                    ),
                                    stmt.span,
                                ));
                            }
                        }
                    }
                }
                let val = Value::Agent {
                    name: Arc::from(name.as_str()),
                    traits: Arc::new(
                        traits.iter().map(|s| Arc::from(s.as_str())).collect(),
                    ),
                    methods: Arc::new(method_map),
                    init: init_val,
                };
                let _ = exported;
                let mut env = self.env.child();
                env.bind(name.clone(), val);
                self.env = env.into_arc();
                Ok(Value::Unit)
            }
            Stmt::FieldUpdate {
                name,
                fields,
                value,
            } => {
                let new_val = self.eval(value)?;
                let current = self.env.get(name).ok_or_else(|| {
                    LxError::runtime(format!("undefined variable '{name}'"), stmt.span)
                })?;
                let updated = Self::update_record_field(&current, fields, new_val, stmt.span)?;
                self.env
                    .reassign(name, updated)
                    .map_err(|e| LxError::runtime(e, stmt.span))?;
                Ok(Value::Unit)
            }
            Stmt::Expr(e) => self.eval(e),
        }
    }
}
