use std::sync::Arc;
use std::sync::mpsc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use parking_lot::Mutex;

use crate::ast::{ProtocolEntry, ProtocolField, SExpr};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{ProtoFieldDef, Value};

use super::Interpreter;

fn inject_deadline(msg: Value) -> Value {
    let Some(ms) = crate::stdlib::deadline::current_remaining_ms() else {
        return msg;
    };
    let Value::Record(fields) = msg else {
        return msg;
    };
    let mut fields = (*fields).clone();
    fields.insert("_deadline_ms".into(), Value::Int(BigInt::from(ms)));
    Value::Record(Arc::new(fields))
}

fn extract_pid(agent: &Value, span: Span) -> Result<u32, LxError> {
    let Value::Record(fields) = agent else {
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
    fn get_agent_handler(&self, target: &Value, span: Span) -> Result<Value, LxError> {
        match target {
            Value::Record(fields) => fields
                .get("handler")
                .cloned()
                .ok_or_else(|| LxError::runtime("agent has no 'handler' field", span)),
            other => Err(LxError::type_err(
                format!(
                    "~> target must be an agent (Record with handler), got {}",
                    other.type_name()
                ),
                span,
            )),
        }
    }

    pub async fn call(&mut self, func: Value, arg: Value) -> Result<Value, LxError> {
        self.apply_func(func, arg, Span::default()).await
    }

    pub(super) async fn eval_agent_send(
        &mut self,
        target_expr: &SExpr,
        msg_expr: &SExpr,
        span: Span,
    ) -> Result<Value, LxError> {
        let target = self.eval(target_expr).await?;
        let msg = inject_deadline(self.eval(msg_expr).await?);
        if matches!(target, Value::Record(ref f) if f.contains_key("__pid")) {
            let pid = extract_pid(&target, span)?;
            crate::stdlib::agent::send_subprocess(pid, &msg, span)?;
            return Ok(Value::Unit);
        }
        let handler = self.get_agent_handler(&target, span)?;
        self.apply_func(handler, msg, span).await?;
        Ok(Value::Unit)
    }

    pub(super) async fn eval_agent_ask(
        &mut self,
        target_expr: &SExpr,
        msg_expr: &SExpr,
        span: Span,
    ) -> Result<Value, LxError> {
        let target = self.eval(target_expr).await?;
        let msg = inject_deadline(self.eval(msg_expr).await?);
        if matches!(target, Value::Record(ref f) if f.contains_key("__pid")) {
            let pid = extract_pid(&target, span)?;
            return crate::stdlib::agent::ask_subprocess(pid, &msg, span);
        }
        let handler = self.get_agent_handler(&target, span)?;
        self.apply_func(handler, msg, span).await
    }

    pub(super) async fn eval_stream_ask(
        &mut self,
        target_expr: &SExpr,
        msg_expr: &SExpr,
        span: Span,
    ) -> Result<Value, LxError> {
        let target = self.eval(target_expr).await?;
        let msg = inject_deadline(self.eval(msg_expr).await?);
        if matches!(target, Value::Record(ref f) if f.contains_key("__pid")) {
            let pid = extract_pid(&target, span)?;
            return crate::stdlib::agent_stream::stream_ask_subprocess(pid, &msg, span);
        }
        let handler = self.get_agent_handler(&target, span)?;
        let result = self.apply_func(handler, msg, span).await?;
        let items = match result {
            Value::List(l) => (*l).clone(),
            Value::Ok(v) => match *v {
                Value::List(l) => (*l).clone(),
                other => vec![Value::Ok(Box::new(other))],
            },
            other => vec![other],
        };
        let (tx, rx) = mpsc::channel();
        for item in items {
            let _ = tx.send(item);
        }
        drop(tx);
        Ok(Value::Stream {
            rx: Arc::new(Mutex::new(rx)),
            cancel_tx: Arc::new(Mutex::new(None)),
        })
    }

    pub(super) async fn eval_protocol_def(
        &mut self,
        name: &str,
        entries: &[ProtocolEntry],
        span: Span,
    ) -> Result<Value, LxError> {
        let mut proto_fields = Vec::new();
        for entry in entries {
            match entry {
                ProtocolEntry::Spread(base_name) => {
                    let base = self.env.get(base_name).ok_or_else(|| {
                        LxError::runtime(
                            format!("Protocol {name}: spread base '{base_name}' not found"),
                            span,
                        )
                    })?;
                    let Value::Trait { fields, .. } = &base else {
                        return Err(LxError::runtime(
                            format!(
                                "Protocol {name}: '{base_name}' is not a Protocol, got {}",
                                base.type_name()
                            ),
                            span,
                        ));
                    };
                    for f in fields.iter() {
                        if let Some(pos) = proto_fields
                            .iter()
                            .position(|pf: &ProtoFieldDef| pf.name == f.name)
                        {
                            proto_fields[pos] = f.clone();
                        } else {
                            proto_fields.push(f.clone());
                        }
                    }
                }
                ProtocolEntry::Field(f) => {
                    let def = self.eval_proto_field(f).await?;
                    if let Some(pos) = proto_fields
                        .iter()
                        .position(|pf: &ProtoFieldDef| pf.name == def.name)
                    {
                        proto_fields[pos] = def;
                    } else {
                        proto_fields.push(def);
                    }
                }
            }
        }
        let val = Value::Trait {
            name: Arc::from(name),
            fields: Arc::new(proto_fields),
            methods: Arc::new(Vec::new()),
            defaults: Arc::new(IndexMap::new()),
            requires: Arc::new(Vec::new()),
            description: None,
            tags: Arc::new(Vec::new()),
        };
        let mut env = self.env.child();
        env.bind(name.to_string(), val);
        self.env = env.into_arc();
        Ok(Value::Unit)
    }

    async fn eval_proto_field(&mut self, f: &ProtocolField) -> Result<ProtoFieldDef, LxError> {
        let default = match &f.default {
            Some(e) => Some(self.eval(e).await?),
            None => None,
        };
        let constraint = f.constraint.as_ref().map(|e| Arc::new(e.clone()));
        Ok(ProtoFieldDef {
            name: f.name.clone(),
            type_name: f.type_name.clone(),
            default,
            constraint,
        })
    }

    pub(super) fn eval_protocol_union(
        &mut self,
        name: &str,
        variants: &[String],
        span: Span,
    ) -> Result<Value, LxError> {
        for v in variants {
            let val = self.env.get(v).ok_or_else(|| {
                LxError::runtime(
                    format!("Protocol union {name}: variant '{v}' not found"),
                    span,
                )
            })?;
            if !matches!(val, Value::Trait { .. }) {
                return Err(LxError::runtime(
                    format!(
                        "Protocol union {name}: variant '{v}' is not a Protocol, got {}",
                        val.type_name()
                    ),
                    span,
                ));
            }
        }
        let variant_arcs: Vec<Arc<str>> = variants.iter().map(|v| Arc::from(v.as_str())).collect();
        let val = Value::ProtocolUnion {
            name: Arc::from(name),
            variants: Arc::new(variant_arcs),
        };
        let mut env = self.env.child();
        env.bind(name.to_string(), val);
        self.env = env.into_arc();
        Ok(Value::Unit)
    }

    pub(super) fn update_record_field(
        val: &Value,
        fields: &[String],
        new_val: Value,
        span: Span,
    ) -> Result<Value, LxError> {
        match (val, fields) {
            (Value::Record(rec), [field]) => {
                let mut new_rec = rec.as_ref().clone();
                new_rec.insert(field.clone(), new_val);
                Ok(Value::Record(Arc::new(new_rec)))
            }
            (Value::Record(rec), [field, rest @ ..]) => {
                let inner = rec
                    .get(field)
                    .ok_or_else(|| LxError::runtime(format!("field '{field}' not found"), span))?;
                let updated = Self::update_record_field(inner, rest, new_val, span)?;
                let mut new_rec = rec.as_ref().clone();
                new_rec.insert(field.clone(), updated);
                Ok(Value::Record(Arc::new(new_rec)))
            }
            (other, _) => Err(LxError::type_err(
                format!("field update requires Record, got {}", other.type_name()),
                span,
            )),
        }
    }
}
