#[path = "mcp_http.rs"]
mod mcp_http;
#[path = "mcp_rpc.rs"]
pub(super) mod mcp_rpc;
#[path = "mcp_stdio.rs"]
mod mcp_stdio;
#[path = "mcp_typed.rs"]
mod mcp_typed;

use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv;
use crate::value::{McpToolDef, Value};

static TOOL_REGISTRY: LazyLock<DashMap<u64, Arc<Vec<McpToolDef>>>> = LazyLock::new(DashMap::new);

pub(crate) fn register_tool_defs(mcp_id: u64, defs: &Arc<Vec<McpToolDef>>) {
    TOOL_REGISTRY.insert(mcp_id, Arc::clone(defs));
}

pub(super) fn get_tool_def(mcp_id: u64, name: &str, span: Span) -> Result<McpToolDef, LxError> {
    let tools = TOOL_REGISTRY
        .get(&mcp_id)
        .ok_or_else(|| LxError::runtime("mcp typed call: no tool defs for client", span))?;
    tools
        .iter()
        .find(|t| t.name == name)
        .cloned()
        .ok_or_else(|| LxError::runtime(format!("mcp typed call: no def for tool '{name}'"), span))
}

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("connect".into(), mk("mcp.connect", 1, mcp_rpc::connect));
    m.insert("close".into(), mk("mcp.close", 1, mcp_rpc::close));
    m.insert("list_tools".into(), mk("mcp.list_tools", 1, bi_list_tools));
    m.insert("call".into(), mk("mcp.call", 3, bi_call));
    m.insert(
        "list_resources".into(),
        mk("mcp.list_resources", 1, bi_list_resources),
    );
    m.insert(
        "read_resource".into(),
        mk("mcp.read_resource", 2, bi_read_resource),
    );
    m.insert(
        "list_prompts".into(),
        mk("mcp.list_prompts", 1, bi_list_prompts),
    );
    m.insert("get_prompt".into(), mk("mcp.get_prompt", 3, bi_get_prompt));
    m
}

fn bi_list_tools(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let empty = serde_json::json!({});
    let result = mcp_rpc::with_proc(&args[0], "tools/list", &empty, span)?;
    let tools = result
        .get("tools")
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));
    Ok(Value::Ok(Box::new(json_conv::json_to_lx(tools))))
}

pub(super) fn extract_text(result: &serde_json::Value) -> String {
    result
        .get("content")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    if c.get("type").and_then(|t| t.as_str()) == Some("text") {
                        c.get("text").and_then(|t| t.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

fn bi_call(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let tool = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("mcp.call: tool name must be Str", span))?;
    let call_args = json_conv::lx_to_json(&args[2], span)?;
    let params = serde_json::json!({"name": tool, "arguments": call_args});
    let result = mcp_rpc::with_proc(&args[0], "tools/call", &params, span)?;
    if result.get("isError") == Some(&serde_json::Value::Bool(true)) {
        let msg = extract_text(&result);
        return Ok(Value::Err(Box::new(super::agent_errors::upstream(
            tool, 0, &msg,
        ))));
    }
    let text = extract_text(&result);
    if !text.is_empty() {
        return Ok(Value::Ok(Box::new(Value::Str(Arc::from(text.as_str())))));
    }
    Ok(Value::Ok(Box::new(json_conv::json_to_lx(result))))
}

fn bi_list_resources(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let empty = serde_json::json!({});
    let result = mcp_rpc::with_proc(&args[0], "resources/list", &empty, span)?;
    let resources = result
        .get("resources")
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));
    Ok(Value::Ok(Box::new(json_conv::json_to_lx(resources))))
}

fn bi_read_resource(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let uri = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("mcp.read_resource: uri must be Str", span))?;
    let params = serde_json::json!({"uri": uri});
    let result = mcp_rpc::with_proc(&args[0], "resources/read", &params, span)?;
    Ok(Value::Ok(Box::new(json_conv::json_to_lx(result))))
}

fn bi_list_prompts(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let empty = serde_json::json!({});
    let result = mcp_rpc::with_proc(&args[0], "prompts/list", &empty, span)?;
    let prompts = result
        .get("prompts")
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));
    Ok(Value::Ok(Box::new(json_conv::json_to_lx(prompts))))
}

fn bi_get_prompt(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("mcp.get_prompt: name must be Str", span))?;
    let prompt_args = json_conv::lx_to_json(&args[2], span)?;
    let params = serde_json::json!({"name": name, "arguments": prompt_args});
    let result = mcp_rpc::with_proc(&args[0], "prompts/get", &params, span)?;
    Ok(Value::Ok(Box::new(json_conv::json_to_lx(result))))
}

pub(crate) use mcp_typed::typed_call;
