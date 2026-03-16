use std::sync::Arc;

use crate::ast::{McpOutputType, McpToolDecl, ProtocolField, SExpr};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{McpOutputDef, McpToolDef, ProtoFieldDef, Value};

use super::Interpreter;

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

    pub fn call(&mut self, func: Value, arg: Value) -> Result<Value, LxError> {
        self.apply_func(func, arg, Span::default())
    }

    pub(super) fn eval_agent_send(
        &mut self,
        target_expr: &SExpr,
        msg_expr: &SExpr,
        span: Span,
    ) -> Result<Value, LxError> {
        let target = self.eval(target_expr)?;
        let msg = self.eval(msg_expr)?;
        if let Value::Record(ref fields) = target
            && let Some(pid_val) = fields.get("__pid")
        {
            let pid: u32 = pid_val
                .as_int()
                .and_then(|n| n.try_into().ok())
                .ok_or_else(|| LxError::runtime("agent: invalid __pid", span))?;
            crate::stdlib::agent::send_subprocess(pid, &msg, span)?;
            return Ok(Value::Unit);
        }
        let handler = self.get_agent_handler(&target, span)?;
        self.apply_func(handler, msg, span)?;
        Ok(Value::Unit)
    }

    pub(super) fn eval_agent_ask(
        &mut self,
        target_expr: &SExpr,
        msg_expr: &SExpr,
        span: Span,
    ) -> Result<Value, LxError> {
        let target = self.eval(target_expr)?;
        let msg = self.eval(msg_expr)?;
        if let Value::Record(ref fields) = target
            && let Some(pid_val) = fields.get("__pid")
        {
            let pid: u32 = pid_val
                .as_int()
                .and_then(|n| n.try_into().ok())
                .ok_or_else(|| LxError::runtime("agent: invalid __pid", span))?;
            return crate::stdlib::agent::ask_subprocess(pid, &msg, span);
        }
        let handler = self.get_agent_handler(&target, span)?;
        self.apply_func(handler, msg, span)
    }

    pub(super) fn eval_protocol_def(
        &mut self,
        name: &str,
        fields: &[ProtocolField],
        span: Span,
    ) -> Result<Value, LxError> {
        let mut proto_fields = Vec::new();
        for f in fields {
            let default = match &f.default {
                Some(e) => Some(self.eval(e)?),
                None => None,
            };
            proto_fields.push(ProtoFieldDef {
                name: f.name.clone(),
                type_name: f.type_name.clone(),
                default,
            });
        }
        let val = Value::Protocol {
            name: Arc::from(name),
            fields: Arc::new(proto_fields),
        };
        let mut env = self.env.child();
        env.bind(name.to_string(), val);
        self.env = env.into_arc();
        let _ = span;
        Ok(Value::Unit)
    }

    pub(super) fn eval_mcp_decl(
        &mut self,
        name: &str,
        tools: &[McpToolDecl],
        _span: Span,
    ) -> Result<Value, LxError> {
        let mut tool_defs = Vec::new();
        for t in tools {
            let mut input = Vec::new();
            for f in &t.input {
                let default = match &f.default {
                    Some(e) => Some(self.eval(e)?),
                    None => None,
                };
                input.push(ProtoFieldDef {
                    name: f.name.clone(),
                    type_name: f.type_name.clone(),
                    default,
                });
            }
            let output = self.resolve_mcp_output(&t.output);
            tool_defs.push(McpToolDef {
                name: t.name.clone(),
                input,
                output,
            });
        }
        let val = Value::McpDecl {
            name: Arc::from(name),
            tools: Arc::new(tool_defs),
        };
        let mut env = self.env.child();
        env.bind(name.to_string(), val);
        self.env = env.into_arc();
        Ok(Value::Unit)
    }

    fn resolve_mcp_output(&self, out: &McpOutputType) -> McpOutputDef {
        match out {
            McpOutputType::Named(n) => {
                if let Some(Value::Protocol { fields, .. }) = self.env.get(n) {
                    McpOutputDef::Record((*fields).clone())
                } else {
                    McpOutputDef::Simple(n.clone())
                }
            }
            McpOutputType::List(inner) => {
                McpOutputDef::List(Box::new(self.resolve_mcp_output(inner)))
            }
            McpOutputType::Record(fields) => {
                let defs = fields
                    .iter()
                    .map(|f| ProtoFieldDef {
                        name: f.name.clone(),
                        type_name: f.type_name.clone(),
                        default: None,
                    })
                    .collect();
                McpOutputDef::Record(defs)
            }
        }
    }

    pub(super) fn apply_protocol(
        &mut self,
        name: &str,
        fields: &Arc<Vec<ProtoFieldDef>>,
        arg: &Value,
        span: Span,
    ) -> Result<Value, LxError> {
        let Value::Record(rec) = arg else {
            return Err(LxError::runtime(
                format!("Protocol {name}: expected Record, got {}", arg.type_name()),
                span,
            ));
        };
        let mut result = rec.as_ref().clone();
        for field in fields.iter() {
            match rec.get(&field.name) {
                Some(val) => {
                    if field.type_name != "Any" && val.type_name() != field.type_name {
                        return Err(LxError::runtime(
                            format!(
                                "Protocol {name}: field '{}' expected {}, got {}",
                                field.name,
                                field.type_name,
                                val.type_name()
                            ),
                            span,
                        ));
                    }
                }
                None => {
                    if let Some(ref default) = field.default {
                        result.insert(field.name.clone(), default.clone());
                    } else {
                        return Err(LxError::runtime(
                            format!("Protocol {name}: missing required field '{}'", field.name),
                            span,
                        ));
                    }
                }
            }
        }
        Ok(Value::Record(Arc::new(result)))
    }

    pub(super) fn apply_mcp_decl(
        &mut self,
        name: &str,
        tools: &Arc<Vec<McpToolDef>>,
        client: &Value,
        span: Span,
    ) -> Result<Value, LxError> {
        let Value::Record(rec) = client else {
            return Err(LxError::type_err(
                format!(
                    "MCP {name}: expected connection Record, got {}",
                    client.type_name()
                ),
                span,
            ));
        };
        let mcp_id: u64 = rec
            .get("__mcp_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| {
                LxError::runtime(
                    format!("MCP {name}: client record must have __mcp_id field"),
                    span,
                )
            })?;
        crate::stdlib::mcp::register_tool_defs(mcp_id, tools);
        let mut result = rec.as_ref().clone();
        for tool in tools.iter() {
            let wrapper = Value::BuiltinFunc(crate::value::BuiltinFunc {
                name: "mcp.typed_call",
                arity: 3,
                func: crate::stdlib::mcp::typed_call,
                applied: vec![client.clone(), Value::Str(Arc::from(tool.name.as_str()))],
            });
            result.insert(tool.name.clone(), wrapper);
        }
        Ok(Value::Record(Arc::new(result)))
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
