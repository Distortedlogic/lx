pub fn issue_status_icon_class(status: &str) -> &'static str {
  match status {
    "backlog" => "text-gray-400 border-gray-400",
    "todo" => "text-blue-400 border-blue-400",
    "in_progress" => "text-yellow-400 border-yellow-400",
    "in_review" => "text-violet-400 border-violet-400",
    "done" => "text-green-400 border-green-400",
    "cancelled" => "text-neutral-500 border-neutral-500",
    "blocked" => "text-red-400 border-red-400",
    _ => "text-gray-400 border-gray-400",
  }
}

pub fn status_badge_class(status: &str) -> &'static str {
  match status {
    "active" => "bg-green-900/50 text-green-300",
    "running" => "bg-cyan-900/50 text-cyan-300",
    "paused" => "bg-orange-900/50 text-orange-300",
    "idle" => "bg-yellow-900/50 text-yellow-300",
    "failed" | "error" | "terminated" => "bg-red-900/50 text-red-300",
    "succeeded" | "done" | "achieved" | "completed" | "approved" => "bg-green-900/50 text-green-300",
    "pending" | "pending_approval" | "revision_requested" => "bg-amber-900/50 text-amber-300",
    "timed_out" => "bg-orange-900/50 text-orange-300",
    "todo" => "bg-blue-900/50 text-blue-300",
    "in_progress" => "bg-yellow-900/50 text-yellow-300",
    "in_review" => "bg-violet-900/50 text-violet-300",
    "blocked" | "rejected" => "bg-red-900/50 text-red-300",
    "backlog" | "cancelled" | "archived" | "planned" => "bg-gray-800 text-gray-400",
    _ => "bg-gray-800 text-gray-400",
  }
}

pub fn priority_color_class(priority: &str) -> &'static str {
  match priority {
    "critical" => "text-red-400",
    "high" => "text-orange-400",
    "medium" => "text-yellow-400",
    "low" => "text-blue-400",
    _ => "text-yellow-400",
  }
}

pub fn priority_label(priority: &str) -> &'static str {
  match priority {
    "critical" => "Critical",
    "high" => "High",
    "medium" => "Medium",
    "low" => "Low",
    _ => "Medium",
  }
}

pub fn status_label(status: &str) -> String {
  status
    .split('_')
    .map(|word| {
      let mut chars = word.chars();
      match chars.next() {
        None => String::new(),
        Some(c) => {
          let upper: String = c.to_uppercase().collect();
          upper + &chars.as_str().to_lowercase()
        },
      }
    })
    .collect::<Vec<_>>()
    .join(" ")
}
