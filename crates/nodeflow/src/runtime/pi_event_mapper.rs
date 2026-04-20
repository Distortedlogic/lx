use super::registry::DesktopRuntimeRegistry;
use super::types::{DesktopRuntimeEvent, DesktopRuntimeEventKind, DesktopToolActivity, DesktopToolStatus, now_ts, result_preview, text_payload};

pub fn handle_stdout_value(registry: &DesktopRuntimeRegistry, agent_id: &str, value: &serde_json::Value) {
  let kind = value.get("type").and_then(serde_json::Value::as_str).unwrap_or("unknown");
  match kind {
    "response" => handle_response(registry, agent_id, value),
    "agent_start" => {
      registry.append_event(DesktopRuntimeEvent::new(agent_id.to_string(), DesktopRuntimeEventKind::AgentSpawn, text_payload("system", "Agent started")))
    },
    "agent_end" => {
      registry.append_event(DesktopRuntimeEvent::new(agent_id.to_string(), DesktopRuntimeEventKind::AgentStop, text_payload("system", "Agent completed")))
    },
    "message_update" => handle_message_update(registry, agent_id, value),
    "message_end" => handle_message_end(registry, agent_id, value),
    "tool_execution_start" => handle_tool_start(registry, agent_id, value),
    "tool_execution_update" => handle_tool_update(registry, agent_id, value),
    "tool_execution_end" => handle_tool_end(registry, agent_id, value),
    "queue_update" => {
      let steering = value.get("steering").cloned().unwrap_or_default();
      let follow_up = value.get("followUp").cloned().unwrap_or_default();
      registry.append_event(DesktopRuntimeEvent::new(
        agent_id.to_string(),
        DesktopRuntimeEventKind::ControlState,
        serde_json::json!({ "text": format!("Queue updated: steering={}, follow_up={}", steering, follow_up) }),
      ));
    },
    other => registry.append_event(DesktopRuntimeEvent::new(
      agent_id.to_string(),
      DesktopRuntimeEventKind::RuntimeEmit,
      serde_json::json!({ "text": format!("Pi event: {other}"), "payload": value }),
    )),
  }
}

fn handle_response(registry: &DesktopRuntimeRegistry, agent_id: &str, value: &serde_json::Value) {
  let command = value.get("command").and_then(serde_json::Value::as_str).unwrap_or("unknown");
  let success = value.get("success").and_then(serde_json::Value::as_bool).unwrap_or(false);
  if command == "get_state" && success {
    if let Some(data) = value.get("data") {
      registry.update_agent(agent_id, |agent| {
        agent.session_id = data.get("sessionId").and_then(serde_json::Value::as_str).unwrap_or(&agent.id).to_string();
        agent.model =
          data.get("model").and_then(|model| model.get("id").or_else(|| model.get("name"))).and_then(serde_json::Value::as_str).map(ToOwned::to_owned);
      });
    }
    return;
  }
  if !success {
    let message = value.get("error").and_then(serde_json::Value::as_str).unwrap_or("Pi command failed");
    registry.append_event(DesktopRuntimeEvent::new(agent_id.to_string(), DesktopRuntimeEventKind::BackendError, text_payload("system", message)));
  }
}

fn handle_message_update(registry: &DesktopRuntimeRegistry, agent_id: &str, value: &serde_json::Value) {
  let event = value.get("assistantMessageEvent").cloned().unwrap_or_default();
  let event_type = event.get("type").and_then(serde_json::Value::as_str).unwrap_or("unknown");
  match event_type {
    "text_delta" => registry.append_event(DesktopRuntimeEvent::new(
      agent_id.to_string(),
      DesktopRuntimeEventKind::MessageDelta,
      serde_json::json!({
        "role": "assistant",
        "message_id": message_id(value),
        "delta": event.get("delta").and_then(serde_json::Value::as_str).unwrap_or_default()
      }),
    )),
    "error" => registry.append_event(DesktopRuntimeEvent::new(
      agent_id.to_string(),
      DesktopRuntimeEventKind::BackendError,
      text_payload("system", event.get("reason").and_then(serde_json::Value::as_str).unwrap_or("assistant stream error")),
    )),
    _ => {},
  }
}

fn handle_message_end(registry: &DesktopRuntimeRegistry, agent_id: &str, value: &serde_json::Value) {
  let Some(message) = value.get("message") else {
    return;
  };
  let role = message.get("role").and_then(serde_json::Value::as_str).unwrap_or("assistant");
  let text = extract_message_text(message);
  if text.is_empty() {
    return;
  }
  registry.append_event(DesktopRuntimeEvent::new(
    agent_id.to_string(),
    DesktopRuntimeEventKind::MessageComplete,
    serde_json::json!({ "role": role, "message_id": message_id(value), "text": text }),
  ));
}

fn handle_tool_start(registry: &DesktopRuntimeRegistry, agent_id: &str, value: &serde_json::Value) {
  let call_id = value.get("toolCallId").and_then(serde_json::Value::as_str).unwrap_or("tool-call").to_string();
  let tool_name = value.get("toolName").and_then(serde_json::Value::as_str).unwrap_or("tool").to_string();
  let args = value.get("args").cloned().unwrap_or_default();
  registry.upsert_tool(DesktopToolActivity::running(agent_id.to_string(), call_id.clone(), tool_name.clone(), args.clone()));
  registry.append_event(DesktopRuntimeEvent::new(
    agent_id.to_string(),
    DesktopRuntimeEventKind::ToolCall,
    serde_json::json!({ "tool_name": tool_name, "call_id": call_id, "args": args }),
  ));
}

fn handle_tool_update(registry: &DesktopRuntimeRegistry, agent_id: &str, value: &serde_json::Value) {
  let call_id = value.get("toolCallId").and_then(serde_json::Value::as_str).unwrap_or("tool-call").to_string();
  let partial = value.get("partialResult").cloned().unwrap_or_default();
  if let Some(mut tool) = registry.tools_for_agent(agent_id).into_iter().find(|tool| tool.call_id == call_id) {
    if let Some(preview) = result_preview(&partial) {
      tool.result_preview = Some(preview);
    }
    registry.upsert_tool(tool);
  }
}

fn handle_tool_end(registry: &DesktopRuntimeRegistry, agent_id: &str, value: &serde_json::Value) {
  let call_id = value.get("toolCallId").and_then(serde_json::Value::as_str).unwrap_or("tool-call").to_string();
  let tool_name = value.get("toolName").and_then(serde_json::Value::as_str).unwrap_or("tool").to_string();
  let result = value.get("result").cloned().unwrap_or_default();
  let is_error = value.get("isError").and_then(serde_json::Value::as_bool).unwrap_or(false);
  let existing = registry.tools_for_agent(agent_id).into_iter().find(|tool| tool.call_id == call_id);
  let preview = result_preview(&result).or_else(|| existing.as_ref().and_then(|tool| tool.result_preview.clone()));
  registry.upsert_tool(DesktopToolActivity {
    call_id: call_id.clone(),
    agent_id: agent_id.to_string(),
    tool_name: existing.as_ref().map(|tool| tool.tool_name.clone()).unwrap_or(tool_name.clone()),
    args: existing.as_ref().map(|tool| tool.args.clone()).unwrap_or(serde_json::Value::Null),
    status: if is_error { DesktopToolStatus::Error } else { DesktopToolStatus::Completed },
    result_preview: preview.clone(),
    is_error,
  });
  registry.append_event(DesktopRuntimeEvent::new(
    agent_id.to_string(),
    if is_error { DesktopRuntimeEventKind::ToolError } else { DesktopRuntimeEventKind::ToolResult },
    serde_json::json!({
      "tool_name": tool_name,
      "call_id": call_id,
      "text": preview.unwrap_or_else(|| format!("Tool finished at {}", now_ts()))
    }),
  ));
}

fn message_id(value: &serde_json::Value) -> String {
  value.get("message").and_then(|message| message.get("id")).and_then(serde_json::Value::as_str).unwrap_or("assistant-message").to_string()
}

fn extract_message_text(message: &serde_json::Value) -> String {
  message
    .get("content")
    .and_then(serde_json::Value::as_array)
    .into_iter()
    .flatten()
    .filter_map(|item| item.get("text").and_then(serde_json::Value::as_str))
    .collect::<Vec<_>>()
    .join("")
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::runtime::types::{DesktopAgentLaunchSpec, DesktopAgentRuntime};

  #[test]
  fn tool_completion_preserves_start_arguments() {
    let registry = DesktopRuntimeRegistry::new();
    let agent = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("Agent", "task", "prompt"));
    let agent_id = agent.id.clone();
    registry.register_agent(agent);

    handle_stdout_value(
      &registry,
      &agent_id,
      &serde_json::json!({
        "type": "tool_execution_start",
        "toolCallId": "call-1",
        "toolName": "bash",
        "args": { "command": "pwd" }
      }),
    );
    handle_stdout_value(
      &registry,
      &agent_id,
      &serde_json::json!({
        "type": "tool_execution_end",
        "toolCallId": "call-1",
        "toolName": "bash",
        "result": { "text": "ok" },
        "isError": false
      }),
    );

    let tools = registry.tools_for_agent(&agent_id);
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].args, serde_json::json!({ "command": "pwd" }));
    assert_eq!(tools[0].status, DesktopToolStatus::Completed);
  }

  #[test]
  fn missing_partial_results_do_not_render_null_previews() {
    let registry = DesktopRuntimeRegistry::new();
    let agent = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("Agent", "task", "prompt"));
    let agent_id = agent.id.clone();
    registry.register_agent(agent);

    handle_stdout_value(
      &registry,
      &agent_id,
      &serde_json::json!({
        "type": "tool_execution_start",
        "toolCallId": "call-1",
        "toolName": "bash",
        "args": { "command": "pwd" }
      }),
    );
    handle_stdout_value(
      &registry,
      &agent_id,
      &serde_json::json!({
        "type": "tool_execution_update",
        "toolCallId": "call-1"
      }),
    );

    let tools = registry.tools_for_agent(&agent_id);
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].result_preview, None);
  }
}
