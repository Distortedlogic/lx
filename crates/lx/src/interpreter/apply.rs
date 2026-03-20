use std::sync::Arc;

use crate::ast::{Param, SExpr};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{BuiltinKind, LxFunc, Value};

use super::Interpreter;

impl Interpreter {
    pub async fn apply_func(
        &mut self,
        func: Value,
        arg: Value,
        span: Span,
    ) -> Result<Value, LxError> {
        match func {
            Value::Func(mut lf) => {
                if let Value::Unit = &arg
                    && lf.arity == 0
                    && lf.applied.is_empty()
                {
                    let is_cross_module = self.source != lf.source_text.as_ref();
                    let fn_source_text = Arc::clone(&lf.source_text);
                    let fn_source_name = Arc::clone(&lf.source_name);
                    let saved = Arc::clone(&self.env);
                    let saved_source =
                        std::mem::replace(&mut self.source, lf.source_text.to_string());
                    self.env = Arc::clone(&lf.closure);
                    let result = self.eval(&lf.body).await;
                    self.env = saved;
                    self.source = saved_source;
                    return match result {
                        Err(LxError::Propagate { value, .. }) => Ok(*value),
                        Err(e) if is_cross_module => {
                            Err(e.with_source(fn_source_name.to_string(), fn_source_text))
                        }
                        other => other,
                    };
                }
                self.apply_named_args(&mut lf, arg);
                if lf.applied.len() == 1
                    && lf.arity > 1
                    && let Value::Tuple(ref elems) = lf.applied[0]
                    && elems.len() == lf.arity
                {
                    let elems = elems.as_ref().clone();
                    lf.applied = elems;
                }
                if lf.applied.len() < lf.arity {
                    return Ok(Value::Func(lf));
                }
                let is_cross_module = self.source != lf.source_text.as_ref();
                let fn_source_text = Arc::clone(&lf.source_text);
                let fn_source_name = Arc::clone(&lf.source_name);
                let saved = Arc::clone(&self.env);
                let saved_source = std::mem::replace(&mut self.source, lf.source_text.to_string());
                let mut call_env = lf.closure.child();
                for (i, name) in lf.params.iter().enumerate() {
                    if i < lf.applied.len() {
                        call_env.bind(name.clone(), lf.applied[i].clone());
                    } else if let Some(Some(def)) = lf.defaults.get(i) {
                        call_env.bind(name.clone(), def.clone());
                    }
                }
                self.env = call_env.into_arc();
                let result = self.eval(&lf.body).await;
                self.env = saved;
                self.source = saved_source;
                match result {
                    Err(LxError::Propagate { value, .. }) => Ok(*value),
                    Err(e) if is_cross_module => {
                        Err(e.with_source(fn_source_name.to_string(), fn_source_text))
                    }
                    other => other,
                }
            }
            Value::BuiltinFunc(mut bf) => {
                bf.applied.push(arg);
                if bf.applied.len() < bf.arity {
                    return Ok(Value::BuiltinFunc(bf));
                }
                match bf.kind {
                    BuiltinKind::Sync(f) => f(&bf.applied, span, &self.ctx),
                    BuiltinKind::Async(f) => f(bf.applied, span, Arc::clone(&self.ctx)).await,
                }
            }
            Value::TaggedCtor {
                tag,
                arity,
                mut applied,
            } => {
                applied.push(arg);
                if applied.len() < arity {
                    Ok(Value::TaggedCtor {
                        tag,
                        arity,
                        applied,
                    })
                } else {
                    Ok(Value::Tagged {
                        tag,
                        values: Arc::new(applied),
                    })
                }
            }
            Value::Trait { name, fields, .. } if !fields.is_empty() => {
                self.apply_protocol(&name, &fields, &arg, span).await
            }
            Value::ProtocolUnion { name, variants } => {
                self.apply_protocol_union(&name, &variants, &arg, span)
                    .await
            }
            Value::McpDecl { name, tools } => self.apply_mcp_decl(&name, &tools, &arg, span),
            Value::Class {
                name,
                traits,
                defaults,
                methods,
            } => {
                let overrides = match &arg {
                    Value::Record(r) => r.as_ref().clone(),
                    Value::Unit => indexmap::IndexMap::new(),
                    _ => {
                        return Err(LxError::type_err(
                            format!(
                                "Class {name} constructor expects Record or (), got {}",
                                arg.type_name()
                            ),
                            span,
                        ));
                    }
                };
                let mut fields = defaults.as_ref().clone();
                for (k, v) in overrides {
                    fields.insert(k, v);
                }
                for v in fields.values_mut() {
                    if let Value::Store { id: store_id } = v {
                        *store_id = crate::stdlib::store_clone(*store_id);
                    }
                }
                let id = crate::stdlib::object_insert(fields);
                Ok(Value::Object {
                    class_name: name,
                    id,
                    traits,
                    methods,
                })
            }
            other => Err(LxError::type_err(
                format!("cannot call {}, not a function", other.type_name()),
                span,
            )),
        }
    }

    pub(super) async fn force_defaults(
        &mut self,
        val: Value,
        _span: Span,
    ) -> Result<Value, LxError> {
        match val {
            Value::Func(ref lf)
                if lf.applied.len() < lf.arity
                    && (lf.applied.len()..lf.arity)
                        .all(|i| matches!(lf.defaults.get(i), Some(Some(_)))) =>
            {
                let Value::Func(lf) = val else { unreachable!() };
                let saved = Arc::clone(&self.env);
                let saved_source = std::mem::replace(&mut self.source, lf.source_text.to_string());
                let mut call_env = lf.closure.child();
                for (i, name) in lf.params.iter().enumerate() {
                    if i < lf.applied.len() {
                        call_env.bind(name.clone(), lf.applied[i].clone());
                    } else if let Some(Some(def)) = lf.defaults.get(i) {
                        call_env.bind(name.clone(), def.clone());
                    }
                }
                self.env = call_env.into_arc();
                let result = self.eval(&lf.body).await;
                self.env = saved;
                self.source = saved_source;
                match result {
                    Err(LxError::Propagate { value, .. }) => Ok(*value),
                    other => other,
                }
            }
            other => Ok(other),
        }
    }

    pub(super) async fn eval_pipe(
        &mut self,
        left: &SExpr,
        right: &SExpr,
        span: Span,
    ) -> Result<Value, LxError> {
        let val = self.eval(left).await?;
        let val = self.force_defaults(val, span).await?;
        let func = self.eval(right).await?;
        self.apply_func(func, val, span).await
    }

    fn apply_named_args(&self, lf: &mut LxFunc, arg: Value) {
        if let Value::Tagged {
            ref tag,
            ref values,
        } = arg
            && tag.as_ref() == "__named"
            && values.len() == 2
            && let Value::Str(ref name) = values[0]
        {
            if let Some(idx) = lf.params.iter().position(|p| p == name.as_ref()) {
                while lf.applied.len() < idx {
                    lf.applied.push(Value::Unit);
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

    pub(super) async fn eval_func(
        &mut self,
        params: &[Param],
        body: &SExpr,
    ) -> Result<Value, LxError> {
        let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
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
                }
                None => None,
            };
            defaults.push(d);
        }
        let arity = params.len();
        let source_name = self
            .source_dir
            .as_ref()
            .map(|d| d.display().to_string())
            .unwrap_or_default();
        Ok(Value::Func(Box::new(LxFunc {
            params: param_names,
            defaults,
            body: Arc::new(body.clone()),
            closure: Arc::clone(&self.env),
            arity,
            applied: vec![],
            source_text: Arc::from(self.source.as_str()),
            source_name: Arc::from(source_name.as_str()),
        })))
    }
}
