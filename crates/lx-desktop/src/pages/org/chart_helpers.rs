use std::collections::HashMap;

use dioxus::prelude::*;

use crate::contexts::activity_log::ActivityLog;
use crate::pages::routines::types::OrgNode;

pub fn nodes_from_events(log: &ActivityLog) -> Vec<OrgNode> {
  let events = log.events.read();
  let mut nodes_map: HashMap<String, OrgNode> = HashMap::new();

  for event in events.iter() {
    match event.kind.as_str() {
      "agent_start" | "agent_running" | "agent_spawn" => {
        let name = event.message.clone();
        let id = name.to_lowercase().replace(' ', "-");
        let entry = nodes_map.entry(id.clone()).or_insert_with(|| OrgNode {
          id,
          name,
          role: "Agent".into(),
          status: if event.kind == "agent_running" { "running".into() } else { "active".into() },
          reports_to: None,
          connected_to: Vec::new(),
          icon: None,
          adapter: event.adapter.clone(),
        });
        if entry.adapter.is_none() && event.adapter.is_some() {
          entry.adapter.clone_from(&event.adapter);
        }
      },
      "agent_reports_to" => {
        let parts: Vec<&str> = event.message.splitn(2, "->").collect();
        if parts.len() == 2 {
          let child_name = parts[0].trim();
          let parent_name = parts[1].trim();
          let child_id = child_name.to_lowercase().replace(' ', "-");
          let parent_id = parent_name.to_lowercase().replace(' ', "-");
          if let Some(node) = nodes_map.get_mut(&child_id) {
            node.reports_to = Some(parent_id);
          }
        }
      },
      k if k == "tell" || k == "ask" || k.contains("message") => {
        let parts: Vec<&str> = event.message.splitn(2, "->").collect();
        if parts.len() == 2 {
          let from_name = parts[0].trim();
          let to_name = parts[1].trim();
          let from_id = from_name.to_lowercase().replace(' ', "-");
          let to_id = to_name.to_lowercase().replace(' ', "-");
          let label = k.to_string();
          if let Some(node) = nodes_map.get_mut(&from_id)
            && !node.connected_to.iter().any(|(id, _)| id == &to_id)
          {
            node.connected_to.push((to_id, label));
          }
        }
      },
      _ => {},
    }
  }

  nodes_map.into_values().collect()
}

pub fn build_children_map(nodes: &[OrgNode]) -> HashMap<String, Vec<OrgNode>> {
  let mut map: HashMap<String, Vec<OrgNode>> = HashMap::new();
  for node in nodes {
    if let Some(parent_id) = &node.reports_to {
      map.entry(parent_id.clone()).or_default().push(node.clone());
    }
  }
  map
}

pub fn status_dot_color(status: &str) -> &'static str {
  match status {
    "running" => "var(--tertiary)",
    "active" => "var(--success)",
    "paused" | "idle" => "var(--warning)",
    "error" => "var(--error)",
    "terminated" => "var(--outline)",
    _ => "var(--outline)",
  }
}
