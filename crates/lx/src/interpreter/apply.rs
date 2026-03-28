use std::mem;
use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::ast::{AstArena, ExprId, Param};
use crate::env::Env;
use crate::error::{EvalResult, EvalSignal, LxError};
use crate::sym::Sym;
use crate::value::{BuiltinKind, LxFunc, LxVal};
use miette::SourceSpan;

use super::Interpreter;

impl Interpreter {
  async fn call_in_closure(&mut self, lf: &LxFunc, call_env: Arc<Env>, cross_module_errors: bool) -> EvalResult<LxVal> {
    let is_cross_module = cross_module_errors && self.source != lf.source_text.as_ref();
    let fn_source_text = Arc::clone(&lf.source_text);
    let fn_source_name = Arc::clone(&lf.source_name);
    let saved = Arc::clone(&self.env);
    let saved_source = mem::replace(&mut self.source, lf.source_text.to_string());
    let saved_arena = Arc::clone(&self.arena);
    self.env = call_env;
    self.arena = Arc::clone(&lf.arena);
    let result = self.eval(lf.body).await;
    self.env = saved;
    self.source = saved_source;
    self.arena = saved_arena;
    match result {
      Err(EvalSignal::Error(LxError::Propagate { value, .. })) => Ok(*value),
      Err(EvalSignal::Error(e)) if is_cross_module => Err(EvalSignal::Error(e.with_source(fn_source_name.to_string(), fn_source_text))),
      other => other,
    }
  }

  pub async fn apply_func(&mut self, func: LxVal, arg: LxVal, span: SourceSpan) -> EvalResult<LxVal> {
    match func {
      LxVal::Func(mut lf) => {
        if let LxVal::Unit = &arg
          && lf.arity == 0
          && lf.applied.is_empty()
        {
          return self.call_in_closure(&lf, Arc::clone(&lf.closure), true).await;
        }
        self.apply_named_args(&mut lf, arg);
        if lf.applied.len() == 1
          && lf.arity > 1
          && let LxVal::Tuple(ref elems) = lf.applied[0]
          && elems.len() == lf.arity
        {
          let elems = elems.as_ref().clone();
          lf.applied = elems;
        }
        if lf.applied.len() < lf.arity {
          return Ok(LxVal::Func(lf));
        }
        let call_env = lf.closure.child();
        call_env.bind_params(&lf.params, &lf.applied, &lf.defaults);
        let call_env = Arc::new(call_env);
        if let Some(guard_eid) = lf.guard
          && !self.eval_guard(guard_eid, &lf.arena, &call_env).await?
        {
          return Ok(LxVal::err_str("guard condition failed"));
        }
        self.call_in_closure(&lf, call_env, true).await
      },
      LxVal::MultiFunc(clauses) => self.apply_multi_func(clauses, arg, span).await,
      LxVal::BuiltinFunc(mut bf) => {
        bf.applied.push(arg);
        if bf.applied.len() < bf.arity {
          return Ok(LxVal::BuiltinFunc(bf));
        }
        match bf.kind {
          BuiltinKind::Sync(f) => Ok(f(&bf.applied, span, &self.ctx)?),
          BuiltinKind::Async(f) => Ok(f(bf.applied, span, Arc::clone(&self.ctx)).await?),
          BuiltinKind::DynAsync(ref f) => Ok(f(bf.applied.clone(), span, Arc::clone(&self.ctx)).await?),
        }
      },
      LxVal::TaggedCtor { tag, arity, mut applied } => {
        applied.push(arg);
        if applied.len() < arity { Ok(LxVal::TaggedCtor { tag, arity, applied }) } else { Ok(LxVal::Tagged { tag, values: Arc::new(applied) }) }
      },
      LxVal::Trait(ref t) if !t.fields.is_empty() => self.apply_trait_fields(t.name.as_str(), &t.fields, &arg, span).await,
      LxVal::TraitUnion { name, variants } => Ok(self.apply_trait_union(name.as_str(), &variants, &arg, span).await?),
      LxVal::Class(c) => {
        let overrides = match &arg {
          LxVal::Record(r) => r.as_ref().clone(),
          LxVal::Unit => IndexMap::new(),
          _ => {
            return Err(LxError::type_err(format!("Class {} constructor expects Record or (), got {}", c.name, arg.type_name()), span, None).into());
          },
        };
        let mut fields = c.defaults.as_ref().clone();
        for (k, v) in overrides {
          fields.insert(k, v);
        }
        for v in fields.values_mut() {
          if let LxVal::Store { id: store_id } = v {
            *store_id = crate::stdlib::store_clone(*store_id).map_err(|e| LxError::runtime(e, span))?;
          }
        }
        let id = crate::stdlib::object_insert(fields);
        Ok(LxVal::Object(Box::new(crate::value::LxObject { class_name: c.name, id, traits: c.traits, methods: c.methods })))
      },
      LxVal::Type(name) => self.apply_type_constructor(name, arg, span),
      other => Err(LxError::type_err(format!("cannot call {}, not a function", other.type_name()), span, None).into()),
    }
  }

  fn apply_type_constructor(&self, name: Sym, arg: LxVal, span: SourceSpan) -> EvalResult<LxVal> {
    match name.as_str() {
      "Str" => Ok(LxVal::str(arg.to_string())),
      "Int" => match &arg {
        LxVal::Int(_) => Ok(arg),
        LxVal::Float(f) => Ok(LxVal::int(*f as i64)),
        LxVal::Str(s) => s.parse::<BigInt>().map(LxVal::Int).map_err(|e| LxError::runtime(format!("Int: cannot parse '{s}': {e}"), span).into()),
        LxVal::Bool(b) => Ok(LxVal::Int(if *b { 1.into() } else { 0.into() })),
        _ => Err(LxError::type_err(format!("cannot construct Int from {}", arg.type_name()), span, None).into()),
      },
      "Float" => match &arg {
        LxVal::Float(_) => Ok(arg),
        LxVal::Int(n) => n.to_f64().map(LxVal::Float).ok_or_else(|| LxError::runtime("Float: int too large", span).into()),
        LxVal::Str(s) => s.parse::<f64>().map(LxVal::Float).map_err(|e| LxError::runtime(format!("Float: cannot parse '{s}': {e}"), span).into()),
        _ => Err(LxError::type_err(format!("cannot construct Float from {}", arg.type_name()), span, None).into()),
      },
      "Bool" => match &arg {
        LxVal::Bool(_) => Ok(arg),
        LxVal::Int(n) => Ok(LxVal::Bool(n != &BigInt::from(0))),
        LxVal::Str(s) => Ok(LxVal::Bool(!s.is_empty())),
        LxVal::None | LxVal::Unit => Ok(LxVal::Bool(false)),
        _ => Ok(LxVal::Bool(true)),
      },
      _ => Err(LxError::runtime(format!("cannot construct {name} from value"), span).into()),
    }
  }

  pub(super) async fn force_defaults(&mut self, val: LxVal, _span: SourceSpan) -> EvalResult<LxVal> {
    match val {
      LxVal::Func(ref lf) if lf.applied.len() < lf.arity && (lf.applied.len()..lf.arity).all(|i| matches!(lf.defaults.get(i), Some(Some(_)))) => {
        let LxVal::Func(lf) = val else { unreachable!() };
        let call_env = lf.closure.child();
        call_env.bind_params(&lf.params, &lf.applied, &lf.defaults);
        self.call_in_closure(&lf, Arc::new(call_env), false).await
      },
      other => Ok(other),
    }
  }

  fn apply_named_args(&self, lf: &mut LxFunc, arg: LxVal) {
    if let LxVal::Tagged { ref tag, ref values } = arg
      && *tag == "__named"
      && values.len() == 2
      && let LxVal::Str(ref name) = values[0]
    {
      if let Some(idx) = lf.params.iter().position(|p| p.as_str() == name.as_ref()) {
        while lf.applied.len() < idx {
          lf.applied.push(LxVal::Unit);
        }
        if lf.applied.len() == idx {
          lf.applied.push(values[1].clone());
        } else {
          lf.applied[idx] = values[1].clone();
        }
      } else {
        lf.applied.push(arg);
      }
    } else {
      lf.applied.push(arg);
    }
  }

  async fn eval_guard(&mut self, guard_eid: ExprId, guard_arena: &Arc<AstArena>, guard_env: &Arc<Env>) -> EvalResult<bool> {
    let saved = Arc::clone(&self.env);
    let saved_arena = Arc::clone(&self.arena);
    self.env = Arc::clone(guard_env);
    self.arena = Arc::clone(guard_arena);
    let guard_span = self.arena.expr_span(guard_eid);
    let result = self.eval(guard_eid).await;
    self.env = saved;
    self.arena = saved_arena;
    match result {
      Ok(LxVal::Bool(b)) => Ok(b),
      Ok(other) => Err(LxError::type_err(format!("guard must return Bool, got {}", other.type_name()), guard_span, None).into()),
      Err(e) => Err(e),
    }
  }

  async fn apply_multi_func(&mut self, mut clauses: Vec<LxFunc>, arg: LxVal, _span: SourceSpan) -> EvalResult<LxVal> {
    for clause in &mut clauses {
      clause.applied.push(arg.clone());
      if clause.applied.len() == 1
        && clause.arity > 1
        && let LxVal::Tuple(ref elems) = clause.applied[0]
        && elems.len() == clause.arity
      {
        let elems = elems.as_ref().clone();
        clause.applied = elems;
      }
    }
    if clauses.iter().any(|c| c.applied.len() < c.arity) {
      return Ok(LxVal::MultiFunc(clauses));
    }
    for clause in &clauses {
      let call_env = clause.closure.child();
      call_env.bind_params(&clause.params, &clause.applied, &clause.defaults);
      let call_env = Arc::new(call_env);
      match clause.guard {
        Some(guard_eid) => {
          if self.eval_guard(guard_eid, &clause.arena, &call_env).await? {
            return self.call_in_closure(clause, call_env, true).await;
          }
        },
        None => return self.call_in_closure(clause, call_env, true).await,
      }
    }
    Ok(LxVal::err_str("no matching clause for function"))
  }

  pub(super) async fn eval_func(&mut self, params: &[Param], guard: Option<ExprId>, body: ExprId) -> EvalResult<LxVal> {
    let param_names: Vec<_> = params.iter().map(|p| p.name).collect();
    let mut defaults = Vec::new();
    for p in params {
      let d = match p.default {
        Some(d_eid) => {
          let mut tmp = Interpreter {
            env: Arc::clone(&self.env),
            source: self.source.clone(),
            source_dir: self.source_dir.clone(),
            module_cache: Arc::clone(&self.module_cache),
            loading: Arc::clone(&self.loading),
            ctx: Arc::clone(&self.ctx),
            arena: Arc::clone(&self.arena),
            tool_modules: vec![],
            agent_name: None,
            agent_mailbox_rx: None,
            agent_handle_fn: None,
            next_ask_id: std::sync::atomic::AtomicU64::new(1),
          };
          Some(tmp.eval(d_eid).await?)
        },
        None => None,
      };
      defaults.push(d);
    }
    let arity = params.len();
    let source_name = self.source_dir.as_ref().map(|d| d.display().to_string()).unwrap_or_default();
    Ok(LxVal::Func(Box::new(LxFunc {
      params: param_names,
      defaults,
      guard,
      body,
      arena: Arc::clone(&self.arena),
      closure: Arc::clone(&self.env),
      arity,
      applied: vec![],
      source_text: Arc::from(self.source.as_str()),
      source_name: Arc::from(source_name.as_str()),
    })))
  }
}
