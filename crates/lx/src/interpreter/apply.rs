use std::sync::Arc;

use crate::ast::{Param, SExpr};
use crate::env::Env;
use crate::error::LxError;
use crate::value::{BuiltinKind, LxFunc, LxVal};
use miette::SourceSpan;

use super::Interpreter;

impl Interpreter {
  async fn call_in_closure(&mut self, lf: &LxFunc, call_env: Arc<Env>, cross_module_errors: bool) -> Result<LxVal, LxError> {
    let is_cross_module = cross_module_errors && self.source != lf.source_text.as_ref();
    let fn_source_text = Arc::clone(&lf.source_text);
    let fn_source_name = Arc::clone(&lf.source_name);
    let saved = Arc::clone(&self.env);
    let saved_source = std::mem::replace(&mut self.source, lf.source_text.to_string());
    self.env = call_env;
    let result = self.eval(&lf.body).await;
    self.env = saved;
    self.source = saved_source;
    match result {
      Err(LxError::Propagate { value, .. }) => Ok(*value),
      Err(e) if is_cross_module => Err(e.with_source(fn_source_name.to_string(), fn_source_text)),
      other => other,
    }
  }

  pub async fn apply_func(&mut self, func: LxVal, arg: LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
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
        if let Some(ref guard) = lf.guard
          && !self.eval_guard(guard, &call_env).await?
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
          BuiltinKind::Sync(f) => f(&bf.applied, span, &self.ctx),
          BuiltinKind::Async(f) => f(bf.applied, span, Arc::clone(&self.ctx)).await,
        }
      },
      LxVal::TaggedCtor { tag, arity, mut applied } => {
        applied.push(arg);
        if applied.len() < arity { Ok(LxVal::TaggedCtor { tag, arity, applied }) } else { Ok(LxVal::Tagged { tag, values: Arc::new(applied) }) }
      },
      LxVal::Trait(ref t) if !t.fields.is_empty() => self.apply_trait_fields(t.name.as_str(), &t.fields, &arg, span).await,
      LxVal::TraitUnion { name, variants } => self.apply_trait_union(name.as_str(), &variants, &arg, span).await,
      LxVal::Class(c) => {
        let overrides = match &arg {
          LxVal::Record(r) => r.as_ref().clone(),
          LxVal::Unit => indexmap::IndexMap::new(),
          _ => {
            return Err(LxError::type_err(format!("Class {} constructor expects Record or (), got {}", c.name, arg.type_name()), span, None));
          },
        };
        let mut fields = c.defaults.as_ref().clone();
        for (k, v) in overrides {
          fields.insert(k, v);
        }
        for v in fields.values_mut() {
          if let LxVal::Store { id: store_id } = v {
            *store_id = crate::stdlib::store_clone(*store_id);
          }
        }
        let id = crate::stdlib::object_insert(fields);
        Ok(LxVal::Object(Box::new(crate::value::LxObject { class_name: c.name, id, traits: c.traits, methods: c.methods })))
      },
      other => Err(LxError::type_err(format!("cannot call {}, not a function", other.type_name()), span, None)),
    }
  }

  pub(super) async fn force_defaults(&mut self, val: LxVal, _span: SourceSpan) -> Result<LxVal, LxError> {
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

  pub(super) async fn eval_pipe(&mut self, left: &SExpr, right: &SExpr, span: SourceSpan) -> Result<LxVal, LxError> {
    let val = self.eval(left).await?;
    let val = self.force_defaults(val, span).await?;
    let func = self.eval(right).await?;
    self.apply_func(func, val, span).await
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

  async fn eval_guard(&mut self, guard: &SExpr, guard_env: &Arc<Env>) -> Result<bool, LxError> {
    let saved = Arc::clone(&self.env);
    self.env = Arc::clone(guard_env);
    let result = self.eval(guard).await;
    self.env = saved;
    match result {
      Ok(LxVal::Bool(b)) => Ok(b),
      Ok(other) => Err(LxError::type_err(format!("guard must return Bool, got {}", other.type_name()), guard.span, None)),
      Err(e) => Err(e),
    }
  }

  async fn apply_multi_func(&mut self, mut clauses: Vec<LxFunc>, arg: LxVal, _span: SourceSpan) -> Result<LxVal, LxError> {
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
      match &clause.guard {
        Some(guard) => {
          if self.eval_guard(guard, &call_env).await? {
            return self.call_in_closure(clause, call_env, true).await;
          }
        },
        None => return self.call_in_closure(clause, call_env, true).await,
      }
    }
    Ok(LxVal::err_str("no matching clause for function"))
  }

  pub(super) async fn eval_func(&mut self, params: &[Param], guard: Option<&SExpr>, body: &SExpr) -> Result<LxVal, LxError> {
    let param_names: Vec<_> = params.iter().map(|p| p.name).collect();
    let mut defaults = Vec::new();
    for p in params {
      let d = match &p.default {
        Some(d) => {
          let mut tmp = Interpreter {
            env: Arc::clone(&self.env),
            source: self.source.clone(),
            source_dir: self.source_dir.clone(),
            module_cache: Arc::clone(&self.module_cache),
            loading: Arc::clone(&self.loading),
            ctx: Arc::clone(&self.ctx),
          };
          Some(tmp.eval(d).await?)
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
      guard: guard.map(|g| Arc::new(g.clone())),
      body: Arc::new(body.clone()),
      closure: Arc::clone(&self.env),
      arity,
      applied: vec![],
      source_text: Arc::from(self.source.as_str()),
      source_name: Arc::from(source_name.as_str()),
    })))
  }
}
