pub fn parse_goal_input(raw: &str) -> (String, Option<String>) {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return (String::new(), None);
  }
  let mut lines = trimmed.lines();
  let title = lines.next().unwrap_or("").trim().to_string();
  let description: String = lines.collect::<Vec<_>>().join("\n").trim().to_string();
  if description.is_empty() { (title, None) } else { (title, Some(description)) }
}

pub fn build_project_payload(goal_id: Option<&str>) -> serde_json::Value {
  let mut payload = serde_json::json!({
      "name": "Onboarding",
      "status": "in_progress"
  });
  if let Some(gid) = goal_id {
    payload["goalIds"] = serde_json::json!([gid]);
  }
  payload
}

pub fn build_issue_payload(title: &str, description: &str, assignee_agent_id: &str, project_id: &str, goal_id: Option<&str>) -> serde_json::Value {
  let mut payload = serde_json::json!({
      "title": title.trim(),
      "assigneeAgentId": assignee_agent_id,
      "projectId": project_id,
      "status": "todo"
  });
  let desc = description.trim();
  if !desc.is_empty() {
    payload["description"] = serde_json::json!(desc);
  }
  if let Some(gid) = goal_id {
    payload["goalId"] = serde_json::json!(gid);
  }
  payload
}
