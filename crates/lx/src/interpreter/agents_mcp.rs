use std::sync::Arc;

use crate::ast::{McpOutputType, McpToolDecl};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{McpOutputDef, McpToolDef, ProtoFieldDef, Value};

use super::Interpreter;

impl Interpreter {
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
                    constraint: None,
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
                        constraint: None,
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
        for field in fields.iter() {
            if let Some(ref constraint_expr) = field.constraint {
                let val = result.get(&field.name).cloned().unwrap_or(Value::Unit);
                let saved = Arc::clone(&self.env);
                let mut scope = self.env.child();
                scope.bind(field.name.clone(), val);
                self.env = scope.into_arc();
                let ok = self.eval(constraint_expr)?;
                self.env = saved;
                match ok.as_bool() {
                    Some(true) => {}
                    _ => {
                        return Err(LxError::runtime(
                            format!(
                                "Protocol {name}: field '{}' constraint violated",
                                field.name
                            ),
                            span,
                        ));
                    }
                }
            }
        }
        Ok(Value::Record(Arc::new(result)))
    }

    pub(super) fn apply_protocol_union(
        &mut self,
        name: &str,
        variants: &Arc<Vec<Arc<str>>>,
        arg: &Value,
        span: Span,
    ) -> Result<Value, LxError> {
        let Value::Record(rec) = arg else {
            return Err(LxError::runtime(
                format!(
                    "Protocol union {name}: expected Record, got {}",
                    arg.type_name()
                ),
                span,
            ));
        };
        for variant_name in variants.iter() {
            let proto = self.env.get(variant_name.as_ref()).ok_or_else(|| {
                LxError::runtime(
                    format!("Protocol union {name}: variant '{variant_name}' not in scope"),
                    span,
                )
            })?;
            let Value::Protocol {
                fields: ref proto_fields,
                ..
            } = proto
            else {
                continue;
            };
            if self.try_match_variant(proto_fields, rec, span).is_ok() {
                let mut result = rec.as_ref().clone();
                result.insert(
                    "_variant".to_string(),
                    Value::Str(Arc::from(variant_name.as_ref())),
                );
                for field in proto_fields.iter() {
                    if !rec.contains_key(&field.name)
                        && let Some(ref default) = field.default
                    {
                        result.insert(field.name.clone(), default.clone());
                    }
                }
                return Ok(Value::Record(Arc::new(result)));
            }
        }
        let variant_names: Vec<&str> = variants.iter().map(|v| v.as_ref()).collect();
        Err(LxError::runtime(
            format!(
                "Protocol union {name}: no variant matched. Tried: {}",
                variant_names.join(", ")
            ),
            span,
        ))
    }

    fn try_match_variant(
        &mut self,
        fields: &Arc<Vec<ProtoFieldDef>>,
        rec: &Arc<indexmap::IndexMap<String, Value>>,
        span: Span,
    ) -> Result<(), LxError> {
        for field in fields.iter() {
            match rec.get(&field.name) {
                Some(val) => {
                    if field.type_name != "Any" && val.type_name() != field.type_name {
                        return Err(LxError::runtime(
                            format!(
                                "field '{}': expected {}, got {} `{}`",
                                field.name,
                                field.type_name,
                                val.type_name(),
                                val.short_display()
                            ),
                            span,
                        ));
                    }
                }
                None => {
                    if field.default.is_none() {
                        return Err(LxError::runtime(
                            format!("missing required field '{}'", field.name),
                            span,
                        ));
                    }
                }
            }
        }
        Ok(())
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
}
