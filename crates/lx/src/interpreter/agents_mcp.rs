use std::sync::Arc;

use crate::ast::{McpOutputType, McpToolDecl};
use crate::error::LxError;
use crate::span::Span;
use crate::value::{BuiltinKind, FieldDef, McpOutputDef, McpToolDef, Value};

use super::Interpreter;

impl Interpreter {
    pub(super) async fn eval_mcp_decl(
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
                    Some(e) => Some(self.eval(e).await?),
                    None => None,
                };
                input.push(FieldDef {
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

    pub(super) fn resolve_mcp_output(&self, out: &McpOutputType) -> McpOutputDef {
        match out {
            McpOutputType::Named(n) => {
                if let Some(Value::Trait { fields, .. }) = self.env.get(n) {
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
                    .map(|f| FieldDef {
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

    pub(super) async fn apply_trait_fields(
        &mut self,
        name: &str,
        fields: &Arc<Vec<FieldDef>>,
        arg: &Value,
        _span: Span,
    ) -> Result<Value, LxError> {
        let Value::Record(rec) = arg else {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
                "Trait {name}: expected Record, got {}",
                arg.type_name()
            ))))));
        };
        let mut result = rec.as_ref().clone();
        for field in fields.iter() {
            match rec.get(&field.name) {
                Some(val) => {
                    if field.type_name != "Any" && val.type_name() != field.type_name {
                        return Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
                            "Trait {name}: field '{}' expected {}, got {}",
                            field.name,
                            field.type_name,
                            val.type_name()
                        ))))));
                    }
                }
                None => {
                    if let Some(ref default) = field.default {
                        result.insert(field.name.clone(), default.clone());
                    } else {
                        return Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
                            "Trait {name}: missing required field '{}'",
                            field.name
                        ))))));
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
                let ok = self.eval(constraint_expr).await?;
                self.env = saved;
                match ok.as_bool() {
                    Some(true) => {}
                    _ => {
                        return Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
                            "Trait {name}: field '{}' constraint violated",
                            field.name
                        ))))));
                    }
                }
            }
        }
        Ok(Value::Record(Arc::new(result)))
    }

    pub(super) async fn apply_trait_union(
        &mut self,
        name: &str,
        variants: &Arc<Vec<Arc<str>>>,
        arg: &Value,
        span: Span,
    ) -> Result<Value, LxError> {
        let Value::Record(rec) = arg else {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
                "Trait union {name}: expected Record, got {}",
                arg.type_name()
            ))))));
        };
        for variant_name in variants.iter() {
            let Some(proto) = self.env.get(variant_name.as_ref()) else {
                continue;
            };
            let Value::Trait {
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
        Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
            "Trait union {name}: no variant matched. Tried: {}",
            variant_names.join(", ")
        ))))))
    }

    fn try_match_variant(
        &mut self,
        fields: &Arc<Vec<FieldDef>>,
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
                kind: BuiltinKind::Sync(crate::stdlib::mcp::typed_call),
                applied: vec![client.clone(), Value::Str(Arc::from(tool.name.as_str()))],
            });
            result.insert(tool.name.clone(), wrapper);
        }
        Ok(Value::Record(Arc::new(result)))
    }
}
