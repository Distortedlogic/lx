use std::sync::{Arc, mpsc};

use num_bigint::BigInt;
use parking_lot::Mutex;

use super::Interpreter;
use crate::ast::{FieldDecl, SExpr, TraitEntry};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{FieldDef, LxVal};

fn inject_deadline(msg: LxVal) -> LxVal {
    let Some(ms) = crate::stdlib::deadline::current_remaining_ms() else {
        return msg;
    };
    let LxVal::Record(fields) = msg else {
        return msg;
    };
    let mut fields = (*fields).clone();
    fields.insert("_deadline_ms".into(), LxVal::Int(BigInt::from(ms)));
    LxVal::Record(Arc::new(fields))
}

fn extract_pid(agent: &LxVal, span: Span) -> Result<u32, LxError> {
    let LxVal::Record(fields) = agent else {
        return Err(LxError::runtime("agent: expected Record with __pid", span));
    };
    let pid_val = fields
        .get("__pid")
        .ok_or_else(|| LxError::runtime("agent: missing __pid", span))?;
    pid_val
        .as_int()
        .and_then(|n| n.try_into().ok())
        .ok_or_else(|| LxError::runtime("agent: invalid __pid", span))
}

impl Interpreter {
    fn get_agent_handler(&self, target: &LxVal, span: Span) -> Result<LxVal, LxError> {
        match target {
            LxVal::Record(fields) => {
                if let Some(handler) = crate::stdlib::agent_reload::handler_id_from_agent(target)
                    .and_then(crate::stdlib::agent_reload::lookup_handler)
                {
                    return Ok(handler);
                }
                fields
                    .get("handler")
                    .cloned()
                    .ok_or_else(|| LxError::runtime("agent has no 'handler' field", span))
            }
            other => Err(LxError::type_err(
                format!(
                    "~> target must be an agent (Record with handler), got {}",
                    other.type_name()
                ),
                span,
            )),
        }
    }

    pub async fn call(&mut self, func: LxVal, arg: LxVal) -> Result<LxVal, LxError> {
        self.apply_func(func, arg, Span::default()).await
    }
    pub(super) async fn eval_agent_send(
        &mut self,
        target_expr: &SExpr,
        msg_expr: &SExpr,
        span: Span,
    ) -> Result<LxVal, LxError> {
        let target = self.eval(target_expr).await?;
        let msg = inject_deadline(self.eval(msg_expr).await?);
        if matches!(target, LxVal::Record(ref f) if f.contains_key("__pid")) {
            let pid = extract_pid(&target, span)?;
            crate::stdlib::agent::send_subprocess(pid, &msg, span)?;
            return Ok(LxVal::Unit);
        }
        if let Some(rejection) =
            crate::stdlib::agent_lifecycle_run::run_message_hooks(&target, &msg, span, &self.ctx)?
        {
            return Ok(rejection);
        }
        let handler_id = crate::stdlib::agent_reload::handler_id_from_agent(&target);
        let handler = self.get_agent_handler(&target, span)?;
        crate::stdlib::agent_reload::set_current_handler_id(handler_id);
        let result = self.apply_func(handler.clone(), msg.clone(), span).await;
        crate::stdlib::agent_reload::set_current_handler_id(None);
        if let Some(hid) = handler_id {
            crate::stdlib::agent_reload::apply_pending_evolve(hid);
        }
        match result {
            Ok(_) => Ok(LxVal::Unit),
            Err(e) => {
                let err_val = LxVal::Str(Arc::from(e.to_string()));
                if crate::stdlib::agent_lifecycle_run::run_error_hooks(
                    &target, &err_val, &msg, span, &self.ctx,
                )?
                .is_some()
                {
                    return Ok(LxVal::Unit);
                }
                Err(e)
            }
        }
    }
    pub(super) async fn eval_agent_ask(
        &mut self,
        target_expr: &SExpr,
        msg_expr: &SExpr,
        span: Span,
    ) -> Result<LxVal, LxError> {
        let target = self.eval(target_expr).await?;
        let msg = inject_deadline(self.eval(msg_expr).await?);
        if matches!(target, LxVal::Record(ref f) if f.contains_key("__pid")) {
            let pid = extract_pid(&target, span)?;
            return crate::stdlib::agent::ask_subprocess(pid, &msg, span);
        }
        if let Some(rejection) =
            crate::stdlib::agent_lifecycle_run::run_message_hooks(&target, &msg, span, &self.ctx)?
        {
            return Ok(rejection);
        }
        let handler_id = crate::stdlib::agent_reload::handler_id_from_agent(&target);
        let handler = self.get_agent_handler(&target, span)?;
        crate::stdlib::agent_reload::set_current_handler_id(handler_id);
        let result = self.apply_func(handler.clone(), msg.clone(), span).await;
        crate::stdlib::agent_reload::set_current_handler_id(None);
        if let Some(hid) = handler_id {
            crate::stdlib::agent_reload::apply_pending_evolve(hid);
        }
        match &result {
            Err(e) => {
                let err_val = LxVal::Str(Arc::from(e.to_string()));
                if let Some(handled) = crate::stdlib::agent_lifecycle_run::run_error_hooks(
                    &target, &err_val, &msg, span, &self.ctx,
                )? {
                    return Ok(handled);
                }
                result
            }
            _ => result,
        }
    }

    pub(super) async fn eval_stream_ask(
        &mut self,
        target_expr: &SExpr,
        msg_expr: &SExpr,
        span: Span,
    ) -> Result<LxVal, LxError> {
        let target = self.eval(target_expr).await?;
        let msg = inject_deadline(self.eval(msg_expr).await?);
        if matches!(target, LxVal::Record(ref f) if f.contains_key("__pid")) {
            let pid = extract_pid(&target, span)?;
            return crate::stdlib::agent_stream::stream_ask_subprocess(pid, &msg, span);
        }
        let handler = self.get_agent_handler(&target, span)?;
        let result = self.apply_func(handler, msg, span).await?;
        let items = match result {
            LxVal::List(l) => (*l).clone(),
            LxVal::Ok(v) => match *v {
                LxVal::List(l) => (*l).clone(),
                other => vec![LxVal::Ok(Box::new(other))],
            },
            other => vec![other],
        };
        let (tx, rx) = mpsc::channel();
        for item in items {
            let _ = tx.send(item);
        }
        drop(tx);
        Ok(LxVal::Stream {
            rx: Arc::new(Mutex::new(rx)),
            cancel_tx: Arc::new(Mutex::new(None)),
        })
    }

    pub(super) async fn eval_trait_fields(
        &mut self,
        name: &str,
        entries: &[TraitEntry],
        span: Span,
    ) -> Result<Vec<FieldDef>, LxError> {
        let mut fields = Vec::new();
        for entry in entries {
            match entry {
                TraitEntry::Spread(base_name) => {
                    let base = self.env.get(base_name).ok_or_else(|| {
                        LxError::runtime(
                            format!("Trait {name}: spread base '{base_name}' not found"),
                            span,
                        )
                    })?;
                    let LxVal::Trait {
                        fields: base_fields,
                        ..
                    } = &base
                    else {
                        return Err(LxError::runtime(
                            format!(
                                "Trait {name}: '{base_name}' is not a Trait, got {}",
                                base.type_name()
                            ),
                            span,
                        ));
                    };
                    for f in base_fields.iter() {
                        if let Some(pos) = fields.iter().position(|pf: &FieldDef| pf.name == f.name)
                        {
                            fields[pos] = f.clone();
                        } else {
                            fields.push(f.clone());
                        }
                    }
                }
                TraitEntry::Field(f) => {
                    let def = self.eval_field_decl(f).await?;
                    if let Some(pos) = fields.iter().position(|pf: &FieldDef| pf.name == def.name) {
                        fields[pos] = def;
                    } else {
                        fields.push(def);
                    }
                }
            }
        }
        Ok(fields)
    }

    async fn eval_field_decl(&mut self, f: &FieldDecl) -> Result<FieldDef, LxError> {
        let default = match &f.default {
            Some(e) => Some(self.eval(e).await?),
            None => None,
        };
        let constraint = f.constraint.as_ref().map(|e| Arc::new(e.clone()));
        Ok(FieldDef {
            name: f.name.clone(),
            type_name: f.type_name.clone(),
            default,
            constraint,
        })
    }

    pub(super) fn eval_trait_union(
        &mut self,
        name: &str,
        variants: &[String],
        span: Span,
    ) -> Result<LxVal, LxError> {
        for v in variants {
            let val = self.env.get(v).ok_or_else(|| {
                LxError::runtime(format!("Trait union {name}: variant '{v}' not found"), span)
            })?;
            if !matches!(val, LxVal::Trait { .. }) {
                return Err(LxError::runtime(
                    format!(
                        "Trait union {name}: variant '{v}' is not a Trait, got {}",
                        val.type_name()
                    ),
                    span,
                ));
            }
        }
        let variant_arcs: Vec<Arc<str>> = variants.iter().map(|v| Arc::from(v.as_str())).collect();
        let val = LxVal::TraitUnion {
            name: Arc::from(name),
            variants: Arc::new(variant_arcs),
        };
        let mut env = self.env.child();
        env.bind(name.to_string(), val);
        self.env = env.into_arc();
        Ok(LxVal::Unit)
    }

    pub(super) fn update_record_field(
        val: &LxVal,
        fields: &[String],
        new_val: LxVal,
        span: Span,
    ) -> Result<LxVal, LxError> {
        match (val, fields) {
            (LxVal::Record(rec), [field]) => {
                let mut new_rec = rec.as_ref().clone();
                new_rec.insert(field.clone(), new_val);
                Ok(LxVal::Record(Arc::new(new_rec)))
            }
            (LxVal::Record(rec), [field, rest @ ..]) => {
                let inner = rec
                    .get(field)
                    .ok_or_else(|| LxError::runtime(format!("field '{field}' not found"), span))?;
                let updated = Self::update_record_field(inner, rest, new_val, span)?;
                let mut new_rec = rec.as_ref().clone();
                new_rec.insert(field.clone(), updated);
                Ok(LxVal::Record(Arc::new(new_rec)))
            }
            (other, _) => Err(LxError::type_err(
                format!("field update requires Record, got {}", other.type_name()),
                span,
            )),
        }
    }
}
