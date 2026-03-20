use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::{McpOutputDef, FieldDef, Value};

use super::{extract_text, get_tool_def, mcp_rpc};

pub(crate) fn typed_call(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let tool_name = args[1]
        .as_str()
        .ok_or_else(|| LxError::runtime("mcp typed call: invalid tool name", span))?;
    let mcp_id: u64 = match &args[0] {
        Value::Record(r) => r
            .get("__mcp_id")
            .and_then(|v| v.as_int())
            .and_then(|n| n.try_into().ok())
            .ok_or_else(|| LxError::runtime("mcp typed call: invalid client", span))?,
        _ => {
            return Err(LxError::runtime(
                "mcp typed call: expected client record",
                span,
            ));
        }
    };
    let tool_def = get_tool_def(mcp_id, tool_name, span)?;
    let validated = match validate_input(&args[2], &tool_def.input, tool_name) {
        Ok(v) => v,
        Err(msg) => return Ok(Value::Err(Box::new(Value::Str(Arc::from(msg.as_str()))))),
    };
    let call_args = json_conv::lx_to_json(&validated, span)?;
    let params = serde_json::json!({"name": tool_name, "arguments": call_args});
    let result = mcp_rpc::with_proc(&args[0], "tools/call", &params, span)?;
    if result.get("isError") == Some(&serde_json::Value::Bool(true)) {
        let msg = extract_text(&result);
        return Ok(Value::Err(Box::new(crate::stdlib::agent_errors::upstream(
            tool_name, 0, &msg,
        ))));
    }
    let raw = extract_typed_result(&result, &tool_def.output);
    match validate_output(&raw, &tool_def.output, tool_name) {
        Ok(v) => Ok(Value::Ok(Box::new(v))),
        Err(msg) => Ok(Value::Err(Box::new(Value::Str(Arc::from(msg.as_str()))))),
    }
}

fn validate_input(
    input: &Value,
    fields: &[FieldDef],
    tool_name: &str,
) -> Result<Value, String> {
    let empty_rec;
    let rec = match input {
        Value::Record(r) => r,
        Value::Unit => {
            empty_rec = Arc::new(IndexMap::new());
            &empty_rec
        }
        _ => {
            return Err(format!(
                "MCP tool '{tool_name}': expected Record args, got {}",
                input.type_name()
            ));
        }
    };
    let mut result = rec.as_ref().clone();
    for field in fields {
        match rec.get(&field.name) {
            Some(val) => {
                if field.type_name != "Any" && val.type_name() != field.type_name {
                    return Err(format!(
                        "MCP tool '{tool_name}': field '{}' expected {}, got {}",
                        field.name,
                        field.type_name,
                        val.type_name()
                    ));
                }
            }
            None => {
                if let Some(ref default) = field.default {
                    result.insert(field.name.clone(), default.clone());
                } else {
                    return Err(format!(
                        "MCP tool '{tool_name}': missing required field '{}'",
                        field.name
                    ));
                }
            }
        }
    }
    Ok(Value::Record(Arc::new(result)))
}

fn extract_typed_result(result: &serde_json::Value, output: &McpOutputDef) -> Value {
    if matches!(output, McpOutputDef::Simple(t) if t == "Str") {
        let text = extract_text(result);
        return Value::Str(Arc::from(text.as_str()));
    }
    let text = extract_text(result);
    if !text.is_empty() {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
            return json_conv::json_to_lx(parsed);
        }
        return Value::Str(Arc::from(text.as_str()));
    }
    json_conv::json_to_lx(result.clone())
}

fn validate_output(
    val: &Value,
    output_def: &McpOutputDef,
    tool_name: &str,
) -> Result<Value, String> {
    match output_def {
        McpOutputDef::Simple(type_name) => {
            if type_name != "Any" && val.type_name() != type_name.as_str() {
                return Err(format!(
                    "MCP tool '{tool_name}': expected output {type_name}, got {}",
                    val.type_name()
                ));
            }
            Ok(val.clone())
        }
        McpOutputDef::Record(fields) => {
            let Value::Record(rec) = val else {
                return Err(format!(
                    "MCP tool '{tool_name}': expected Record output, got {}",
                    val.type_name()
                ));
            };
            for field in fields {
                match rec.get(&field.name) {
                    Some(fv) => {
                        if field.type_name != "Any" && fv.type_name() != field.type_name {
                            return Err(format!(
                                "MCP tool '{tool_name}': output field '{}' expected {}, got {}",
                                field.name,
                                field.type_name,
                                fv.type_name()
                            ));
                        }
                    }
                    None => {
                        return Err(format!(
                            "MCP tool '{tool_name}': output missing field '{}'",
                            field.name
                        ));
                    }
                }
            }
            Ok(val.clone())
        }
        McpOutputDef::List(inner) => {
            let Value::List(items) = val else {
                return Err(format!(
                    "MCP tool '{tool_name}': expected List output, got {}",
                    val.type_name()
                ));
            };
            for item in items.iter() {
                validate_output(item, inner, tool_name)?;
            }
            Ok(val.clone())
        }
    }
}
