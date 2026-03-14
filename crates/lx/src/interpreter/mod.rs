mod apply;
mod collections;
mod modules;
mod patterns;
mod shell;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use num_traits::ToPrimitive;

use crate::ast::{SExpr, Program, Expr, SStmt, Stmt, BindTarget, Literal, BinOp, UnaryOp, StrPart, SelArm, ProtocolField};
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{Value, ProtoFieldDef};

#[derive(Debug, Clone)]
pub(crate) struct ModuleExports {
  pub(crate) bindings: IndexMap<String, Value>,
  pub(crate) variant_ctors: Vec<String>,
}

fn dedent_string(s: &str) -> String {
  let lines: Vec<&str> = s.split('\n').collect();
  let trimmed: Vec<&str> = if lines.first() == Some(&"") { lines[1..].to_vec() } else { lines.to_vec() };
  if trimmed.is_empty() {
    return String::new();
  }
  let last_is_whitespace = trimmed.last().is_some_and(|l| l.chars().all(|c| c == ' ' || c == '\t'));
  let content_lines = if last_is_whitespace { &trimmed[..trimmed.len() - 1] } else { &trimmed[..] };
  let min_indent = content_lines
    .iter()
    .filter(|l| !l.is_empty())
    .map(|l| l.len() - l.trim_start().len())
    .min()
    .unwrap_or(0);
  let mut result = String::new();
  for line in content_lines {
    if line.len() >= min_indent {
      result.push_str(&line[min_indent..]);
    }
    result.push('\n');
  }
  result
}

pub struct Interpreter {
  env: Arc<Env>,
  source: String,
  pub(crate) source_dir: Option<PathBuf>,
  pub(crate) module_cache: Arc<Mutex<HashMap<PathBuf, ModuleExports>>>,
  pub(crate) loading: Arc<Mutex<HashSet<PathBuf>>>,
}

impl Interpreter {
  pub fn new(source: &str, source_dir: Option<PathBuf>) -> Self {
    let mut env = Env::new();
    crate::builtins::register(&mut env);
    Self {
      env: env.into_arc(),
      source: source.to_string(),
      source_dir,
      module_cache: Arc::new(Mutex::new(HashMap::new())),
      loading: Arc::new(Mutex::new(HashSet::new())),
    }
  }

  pub fn with_env(env: &Env) -> Self {
    Self {
      env: Arc::new(env.clone()),
      source: String::new(),
      source_dir: None,
      module_cache: Arc::new(Mutex::new(HashMap::new())),
      loading: Arc::new(Mutex::new(HashSet::new())),
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
        && matches!(b.value.node, Expr::Func { .. }) {
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

  fn eval(&mut self, expr: &SExpr) -> Result<Value, LxError> {
    let span = expr.span;
    match &expr.node {
      Expr::Literal(lit) => self.eval_literal(lit, span),
      Expr::Ident(name) => self.env.get(name).ok_or_else(|| LxError::runtime(format!("undefined variable '{name}'"), span)),
      Expr::TypeConstructor(name) => self.env.get(name).ok_or_else(|| LxError::runtime(format!("undefined constructor '{name}'"), span)),
      Expr::Binary { op, left, right } => self.eval_binary(op, left, right, span),
      Expr::Unary { op, operand } => self.eval_unary(op, operand, span),
      Expr::Pipe { left, right } => self.eval_pipe(left, right, span),
      Expr::Apply { func, arg } => {
        let f = self.eval(func)?;
        if let Expr::NamedArg { name, value } = &arg.node {
          let v = self.eval(value)?;
          let named = Value::Tagged { tag: Arc::from("__named"), values: Arc::new(vec![Value::Str(Arc::from(name.as_str())), v]) };
          self.apply_func(f, named, span)
        } else {
          let a = self.eval(arg)?;
          self.apply_func(f, a, span)
        }
      },
      Expr::Section(sec) => self.eval_section(sec, span),
      Expr::Compose { left, right } => self.eval_compose(left, right, span),
      Expr::FieldAccess { expr: e, field } => self.eval_field_access(e, field, span),
      Expr::Block(stmts) => self.eval_block(stmts),
      Expr::Tuple(elems) => self.eval_tuple(elems),
      Expr::List(elems) => self.eval_list(elems),
      Expr::Record(fields) => self.eval_record(fields),
      Expr::Map(entries) => self.eval_map(entries),
      Expr::Set(elems) => self.eval_set(elems),
      Expr::Func { params, body, returns_result } => self.eval_func(params, body, *returns_result),
      Expr::Match { scrutinee, arms } => self.eval_match(scrutinee, arms, span),
      Expr::Ternary { cond, then_, else_ } => self.eval_ternary(cond, then_, else_, span),
      Expr::Assert { expr: e, msg } => self.eval_assert(e, msg, span),
      Expr::Propagate(inner) => {
        let v = self.eval(inner)?;
        match v {
          Value::Ok(v) => Ok(*v),
          Value::Err(_) => Err(LxError::propagate(v, span)),
          Value::Some(v) => Ok(*v),
          Value::None => Err(LxError::propagate(Value::Err(Box::new(Value::Str(Arc::from("unwrapped None")))), span)),
          other => Err(LxError::type_err(format!("^ expects Result or Maybe, got {}", other.type_name()), span)),
        }
      },
      Expr::Coalesce { expr: e, default } => {
        let v = self.eval(e)?;
        match v {
          Value::Ok(inner) | Value::Some(inner) => Ok(*inner),
          Value::Err(_) | Value::None => self.eval(default),
          other => Ok(other),
        }
      },
      Expr::Slice { expr: e, start: s, end: en } => self.eval_slice(e, s.as_deref(), en.as_deref(), span),
      Expr::NamedArg { name, value } => {
        let _ = name;
        self.eval(value)
      },
      Expr::Loop(stmts) => self.eval_loop(stmts),
      Expr::Break(val) => {
        let v = match val {
          Some(e) => self.eval(e)?,
          None => Value::Unit,
        };
        Err(LxError::break_signal(v))
      },
      Expr::Par(stmts) => self.eval_par(stmts),
      Expr::Sel(arms) => self.eval_sel(arms, span),
      Expr::AgentSend { target, msg } => self.eval_agent_send(target, msg, span),
      Expr::AgentAsk { target, msg } => self.eval_agent_ask(target, msg, span),
      Expr::Shell { mode, parts } => self.eval_shell(mode, parts, span),
    }
  }

  fn eval_stmt(&mut self, stmt: &SStmt) -> Result<Value, LxError> {
    match &stmt.node {
      Stmt::Binding(b) => {
        let val = self.eval(&b.value)?;
        let val = self.force_defaults(val, stmt.span)?;
        match &b.target {
          BindTarget::Name(name) => {
            if self.env.has_mut(name) {
              self.env.reassign(name, val).map_err(|e| LxError::runtime(e, stmt.span))?;
            } else {
              let mut env = self.env.child();
              if b.mutable {
                env.bind_mut(name.clone(), val);
              } else {
                env.bind(name.clone(), val);
              }
              self.env = env.into_arc();
            }
          },
          BindTarget::Reassign(name) => {
            self.env.reassign(name, val).map_err(|e| LxError::runtime(e, stmt.span))?;
          },
          BindTarget::Pattern(pat) => {
            let bindings = self.try_match_pattern(&pat.node, &val).ok_or_else(|| LxError::runtime("pattern match failed in binding", stmt.span))?;
            let mut env = self.env.child();
            for (name, v) in bindings {
              if b.mutable {
                env.bind_mut(name, v);
              } else {
                env.bind(name, v);
              }
            }
            self.env = env.into_arc();
          },
        }
        Ok(Value::Unit)
      },
      Stmt::Use(use_stmt) => {
        self.eval_use(use_stmt, stmt.span)?;
        Ok(Value::Unit)
      },
      Stmt::TypeDef { variants, .. } => {
        let mut env = self.env.child();
        for (ctor_name, arity) in variants {
          let tag: Arc<str> = Arc::from(ctor_name.as_str());
          if *arity == 0 {
            env.bind(ctor_name.clone(), Value::Tagged { tag, values: Arc::new(vec![]) });
          } else {
            env.bind(ctor_name.clone(), Value::TaggedCtor { tag, arity: *arity, applied: vec![] });
          }
        }
        self.env = env.into_arc();
        Ok(Value::Unit)
      },
      Stmt::Protocol { name, fields, .. } => {
        self.eval_protocol_def(name, fields, stmt.span)
      },
      Stmt::Expr(e) => self.eval(e),
    }
  }

  fn eval_literal(&mut self, lit: &Literal, span: Span) -> Result<Value, LxError> {
    match lit {
      Literal::Int(n) => Ok(Value::Int(n.clone())),
      Literal::Float(f) => Ok(Value::Float(*f)),
      Literal::Bool(b) => Ok(Value::Bool(*b)),
      Literal::Str(parts) => self.eval_string_parts(parts),
      Literal::RawStr(s) => Ok(Value::Str(Arc::from(s.as_str()))),
      Literal::Regex { pattern, flags } => Ok(Value::Regex { pattern: Arc::from(pattern.as_str()), flags: Arc::from(flags.as_str()) }),
      Literal::Unit => {
        let _ = span;
        Ok(Value::Unit)
      },
    }
  }

  fn eval_binary(&mut self, op: &BinOp, left: &SExpr, right: &SExpr, span: Span) -> Result<Value, LxError> {
    if *op == BinOp::And {
      let l = self.eval(left)?;
      let l = self.force_defaults(l, span)?;
      return match l.as_bool() {
        Some(false) => Ok(Value::Bool(false)),
        Some(true) => {
          let r = self.eval(right)?;
          self.force_defaults(r, span)
        },
        _ => Err(LxError::type_err("&& requires Bool operands", span)),
      };
    }
    if *op == BinOp::Or {
      let l = self.eval(left)?;
      let l = self.force_defaults(l, span)?;
      return match l.as_bool() {
        Some(true) => Ok(Value::Bool(true)),
        Some(false) => {
          let r = self.eval(right)?;
          self.force_defaults(r, span)
        },
        _ => Err(LxError::type_err("|| requires Bool operands", span)),
      };
    }
    let lv = self.eval(left)?;
    let lv = self.force_defaults(lv, span)?;
    let rv = self.eval(right)?;
    let rv = self.force_defaults(rv, span)?;
    self.binary_op(op, &lv, &rv, span)
  }

  fn binary_op(&self, op: &BinOp, lv: &Value, rv: &Value, span: Span) -> Result<Value, LxError> {
    match op {
      BinOp::Eq => return Ok(Value::Bool(lv == rv)),
      BinOp::NotEq => return Ok(Value::Bool(lv != rv)),
      _ => {},
    }
    match (op, lv, rv) {
      (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
      (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
      (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
      (BinOp::Div, Value::Int(a), Value::Int(b)) => {
        if b.sign() == num_bigint::Sign::NoSign {
          return Err(LxError::division_by_zero(span));
        }
        Ok(Value::Int(a / b))
      },
      (BinOp::IntDiv, Value::Int(a), Value::Int(b)) => {
        if b.sign() == num_bigint::Sign::NoSign {
          return Err(LxError::division_by_zero(span));
        }
        let (q, r) = num_integer::div_rem(a.clone(), b.clone());
        if r.sign() != num_bigint::Sign::NoSign && (a.sign() != b.sign()) { Ok(Value::Int(q - 1)) } else { Ok(Value::Int(q)) }
      },
      (BinOp::Mod, Value::Int(a), Value::Int(b)) => {
        if b.sign() == num_bigint::Sign::NoSign {
          return Err(LxError::division_by_zero(span));
        }
        Ok(Value::Int(a % b))
      },
      (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
      (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
      (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
      (BinOp::Div, Value::Float(_), Value::Float(b)) if *b == 0.0 => Err(LxError::division_by_zero(span)),
      (BinOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
      (BinOp::IntDiv, Value::Float(_), Value::Float(b)) if *b == 0.0 => Err(LxError::division_by_zero(span)),
      (BinOp::IntDiv, Value::Float(a), Value::Float(b)) => Ok(Value::Float((a / b).floor())),
      (BinOp::Mod, Value::Float(_), Value::Float(b)) if *b == 0.0 => Err(LxError::division_by_zero(span)),
      (BinOp::Mod, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
      (op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::IntDiv | BinOp::Mod), Value::Int(a), Value::Float(b)) => {
        let af = a.to_f64().ok_or_else(|| LxError::runtime("int too large for float", span))?;
        self.binary_op(op, &Value::Float(af), &Value::Float(*b), span)
      },
      (op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::IntDiv | BinOp::Mod), Value::Float(a), Value::Int(b)) => {
        let bf = b.to_f64().ok_or_else(|| LxError::runtime("int too large for float", span))?;
        self.binary_op(op, &Value::Float(*a), &Value::Float(bf), span)
      },
      (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
      (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
      (BinOp::LtEq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
      (BinOp::GtEq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
      (BinOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
      (BinOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
      (BinOp::LtEq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
      (BinOp::GtEq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
      (BinOp::Lt, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a < b)),
      (BinOp::Gt, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a > b)),
      (BinOp::LtEq, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a <= b)),
      (BinOp::GtEq, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a >= b)),
      (BinOp::Concat, Value::Str(a), Value::Str(b)) => {
        let mut s = String::from(a.as_ref());
        s.push_str(b);
        Ok(Value::Str(Arc::from(s)))
      },
      (BinOp::Range, Value::Int(a), Value::Int(b)) => {
        let s = a.to_i64().ok_or_else(|| LxError::runtime("range start too large", span))?;
        let e = b.to_i64().ok_or_else(|| LxError::runtime("range end too large", span))?;
        Ok(Value::Range { start: s, end: e, inclusive: false })
      },
      (BinOp::RangeInclusive, Value::Int(a), Value::Int(b)) => {
        let s = a.to_i64().ok_or_else(|| LxError::runtime("range start too large", span))?;
        let e = b.to_i64().ok_or_else(|| LxError::runtime("range end too large", span))?;
        Ok(Value::Range { start: s, end: e, inclusive: true })
      },
      (BinOp::Concat, Value::List(a), Value::List(b)) => {
        let mut v = a.as_ref().clone();
        v.extend(b.as_ref().iter().cloned());
        Ok(Value::List(Arc::new(v)))
      },
      _ => Err(LxError::type_err(format!("cannot apply '{op}' to {} and {}", lv.type_name(), rv.type_name()), span)),
    }
  }

  fn eval_unary(&mut self, op: &UnaryOp, operand: &SExpr, span: Span) -> Result<Value, LxError> {
    let v = self.eval(operand)?;
    match (op, &v) {
      (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
      (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
      (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
      _ => Err(LxError::type_err(format!("cannot apply '{op}' to {}", v.type_name()), span)),
    }
  }

  fn eval_string_parts(&mut self, parts: &[StrPart]) -> Result<Value, LxError> {
    let mut buf = String::new();
    for part in parts {
      match part {
        StrPart::Text(t) => buf.push_str(t),
        StrPart::Interp(e) => {
          let v = self.eval(e)?;
          let v = self.force_defaults(v, e.span)?;
          buf.push_str(&format!("{v}"));
        },
      }
    }
    if buf.starts_with('\n') {
      buf = dedent_string(&buf);
    }
    Ok(Value::Str(Arc::from(buf)))
  }

  fn eval_block(&mut self, stmts: &[SStmt]) -> Result<Value, LxError> {
    let saved = Arc::clone(&self.env);
    self.env = Arc::new(self.env.child());
    let mut result = Value::Unit;
    for stmt in stmts {
      result = self.eval_stmt(stmt)?;
    }
    self.env = saved;
    Ok(result)
  }

  fn eval_loop(&mut self, stmts: &[SStmt]) -> Result<Value, LxError> {
    loop {
      let saved = Arc::clone(&self.env);
      self.env = Arc::new(self.env.child());
      for stmt in stmts {
        match self.eval_stmt(stmt) {
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

  fn eval_slice(&mut self, expr: &SExpr, start: Option<&SExpr>, end: Option<&SExpr>, span: Span) -> Result<Value, LxError> {
    let val = self.eval(expr)?;
    let items = match &val {
      Value::List(l) => l.as_ref(),
      other => return Err(LxError::type_err(format!("slice requires List, got {}", other.type_name()), span)),
    };
    let len = items.len();
    let s = match start {
      Some(e) => {
        let v = self.eval(e)?;
        v.as_int().and_then(|n| n.try_into().ok()).ok_or_else(|| LxError::type_err("slice index must be Int", span))?
      },
      None => 0usize,
    };
    let en: usize = match end {
      Some(e) => {
        let v = self.eval(e)?;
        v.as_int().and_then(|n| n.try_into().ok()).ok_or_else(|| LxError::type_err("slice index must be Int", span))?
      },
      None => len,
    };
    let s = s.min(len);
    let en = en.min(len);
    Ok(Value::List(Arc::new(items[s..en].to_vec())))
  }

  fn eval_par(&mut self, stmts: &[SStmt]) -> Result<Value, LxError> {
    let saved = Arc::clone(&self.env);
    self.env = Arc::new(self.env.child());
    let mut results = Vec::new();
    for stmt in stmts {
      let val = self.eval_stmt(stmt)?;
      results.push(val);
    }
    self.env = saved;
    Ok(Value::Tuple(Arc::new(results)))
  }

  fn eval_sel(&mut self, arms: &[SelArm], span: Span) -> Result<Value, LxError> {
    if arms.is_empty() {
      return Err(LxError::runtime("sel: no arms", span));
    }
    let val = self.eval(&arms[0].expr)?;
    let saved = Arc::clone(&self.env);
    let mut child = self.env.child();
    child.bind("it".into(), val);
    self.env = Arc::new(child);
    let result = self.eval(&arms[0].handler);
    self.env = saved;
    result
  }

  fn get_agent_handler(&self, target: &Value, span: Span) -> Result<Value, LxError> {
    match target {
      Value::Record(fields) => {
        fields.get("handler").cloned().ok_or_else(|| LxError::runtime("agent has no 'handler' field", span))
      },
      other => Err(LxError::type_err(format!("~> target must be an agent (Record with handler), got {}", other.type_name()), span)),
    }
  }

  pub fn call(&mut self, func: Value, arg: Value) -> Result<Value, LxError> {
    self.apply_func(func, arg, Span::default())
  }

  fn eval_agent_send(&mut self, target_expr: &SExpr, msg_expr: &SExpr, span: Span) -> Result<Value, LxError> {
    let target = self.eval(target_expr)?;
    let msg = self.eval(msg_expr)?;
    if let Value::Record(ref fields) = target
      && let Some(pid_val) = fields.get("__pid") {
        let pid: u32 = pid_val.as_int()
          .and_then(|n| n.try_into().ok())
          .ok_or_else(|| LxError::runtime("agent: invalid __pid", span))?;
        crate::stdlib::agent::send_subprocess(pid, &msg, span)?;
        return Ok(Value::Unit);
      }
    let handler = self.get_agent_handler(&target, span)?;
    self.apply_func(handler, msg, span)?;
    Ok(Value::Unit)
  }

  fn eval_agent_ask(&mut self, target_expr: &SExpr, msg_expr: &SExpr, span: Span) -> Result<Value, LxError> {
    let target = self.eval(target_expr)?;
    let msg = self.eval(msg_expr)?;
    if let Value::Record(ref fields) = target
      && let Some(pid_val) = fields.get("__pid") {
        let pid: u32 = pid_val.as_int()
          .and_then(|n| n.try_into().ok())
          .ok_or_else(|| LxError::runtime("agent: invalid __pid", span))?;
        return crate::stdlib::agent::ask_subprocess(pid, &msg, span);
      }
    let handler = self.get_agent_handler(&target, span)?;
    self.apply_func(handler, msg, span)
  }

  fn eval_protocol_def(&mut self, name: &str, fields: &[ProtocolField], span: Span) -> Result<Value, LxError> {
    let mut proto_fields = Vec::new();
    for f in fields {
      let default = match &f.default {
        Some(e) => Some(self.eval(e)?),
        None => None,
      };
      proto_fields.push(ProtoFieldDef { name: f.name.clone(), type_name: f.type_name.clone(), default });
    }
    let val = Value::Protocol { name: Arc::from(name), fields: Arc::new(proto_fields) };
    let mut env = self.env.child();
    env.bind(name.to_string(), val);
    self.env = env.into_arc();
    let _ = span;
    Ok(Value::Unit)
  }

  fn eval_assert(&mut self, expr: &SExpr, msg: &Option<Box<SExpr>>, span: Span) -> Result<Value, LxError> {
    let val = self.eval(expr)?;
    let val = self.force_defaults(val, span)?;
    match val.as_bool() {
      Some(true) => Ok(Value::Unit),
      Some(false) => {
        let message = match msg {
          Some(m) => {
            let mv = self.eval(m)?;
            Some(format!("{mv}"))
          },
          None => None,
        };
        Err(LxError::assert_fail(format!("{:?}", expr.node), message, span))
      },
      _ => Err(LxError::type_err("assert requires Bool", span)),
    }
  }
}
